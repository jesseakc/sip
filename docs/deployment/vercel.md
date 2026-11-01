# Deploying SIP to Vercel

**The frontend deploys to Vercel. The Rust API does not.** Vercel runs Next.js and
short-lived functions; SIP's API is a long-lived Axum server (SSE streaming in
`routes/ai.rs`, in-process background work via `tokio::spawn` in
`services.rs`) that also serves plugin bundles from `plugins/*/dist` on disk. There
is no way to run that as-is on Vercel without rewriting it as per-request
handlers, so this guide puts the frontend on Vercel and the API on a container
host.

| Component | Where | Why |
|---|---|---|
| `frontend/` (Next.js 15) | **Vercel** | Static + SSR pages, no build-time API calls |
| `sip-api` (Rust, Axum) | Container host (Fly.io, Railway, Render…) | Long-lived process, SSE, background jobs |
| PostgreSQL 16 | Neon / Supabase / RDS | Needs `uuid-ossp`, `vector`, `ltree`, `pg_trgm` |
| Redis | Upstash (`rediss://`) | Add cache/rate-limit backend |
| Object storage | S3 / Cloudflare R2 | MinIO is not available on either platform |

## 1. Vercel project

Import the GitHub repo and set:

| Setting | Value |
|---|---|
| **Root Directory** | `frontend` |
| Framework Preset | Next.js (auto-detected) |
| Install Command | `npm ci` |
| Build Command | `npm run build` |
| Output Directory | *(leave default)* |

`frontend/vercel.json` pins the build commands and adds baseline security
headers. Root Directory is a project setting, not a `vercel.json` key: if you
leave it at the repo root, the build fails because there is no root `package.json`.

### Environment variable

| Variable | Value | Notes |
|---|---|---|
| `NEXT_PUBLIC_API_URL` | `https://api.example.com/api/v1` | **Must include `/api/v1`** |

Set it for **Production and Preview** separately. This value is inlined into the
client bundle at build time, so changing it requires a redeploy: it is not read
at runtime.

`/api/v1` is not optional. `lib/api.ts` appends paths such as `/auth/login` to it
verbatim, so a bare origin posts to `https://api.example.com/auth/login` and every
login returns 401.

## 2. The API

Build the existing image: `docker/Dockerfile.api` is already the production
build, and it bakes `SIP_BUILD_COMMIT` in so you can prove which revision is live.

```bash
docker build -f docker/Dockerfile.api \
  --build-arg SIP_BUILD_COMMIT=$(git rev-parse --short HEAD) \
  -t sip-api .
```

Fly.io is the shortest path:

```bash
fly launch --no-deploy --image-label sip-api
fly secrets set \
  SIP_ENV=production \
  SIP_AUTH__JWT_SECRET="$(openssl rand -base64 48)" \
  SIP_DATABASE__URL='postgres://USER:PASS@HOST:5432/sip?sslmode=require' \
  SIP_REDIS__URL='rediss://default:PASS@HOST:6379' \
  SIP_SECURITY__CORS_ALLOWED_ORIGINS='["https://your-app.vercel.app"]' \
  SIP_SECURITY__REQUIRE_HTTPS=true \
  SIP_OBJECT_STORAGE__ENDPOINT='https://ACCOUNT.r2.cloudflarestorage.com' \
  SIP_OBJECT_STORAGE__ACCESS_KEY='...' \
  SIP_OBJECT_STORAGE__SECRET_KEY='...' \
  SIP_OBJECT_STORAGE__BUCKET=sip-documents
fly deploy
```

### Configuration keys

Nested keys use `SIP_` + `__` (`SIP_SERVER__PORT` → `server.port`), which is how
figment loads them. Flat names such as `SIP_CORS_ALLOWED_ORIGINS` appear in some
warning messages but **do not** map: use the nested form.

| Variable | Required | Notes |
|---|---|---|
| `SIP_ENV` | yes | `production` enables strict validation |
| `SIP_AUTH__JWT_SECRET` | yes | **≥32 chars or startup validation fails** |
| `SIP_DATABASE__URL` | yes | `?sslmode=require` for managed Postgres |
| `SIP_REDIS__URL` | yes | `rediss://` for Upstash |
| `SIP_SECURITY__CORS_ALLOWED_ORIGINS` | yes | JSON array of exact origins |
| `SIP_SECURITY__REQUIRE_HTTPS` | recommended | logs a production warning if unset |
| `SIP_SECURITY__RATE_LIMIT_RPM` | recommended | warns at ≥1000 |
| `SIP_OBJECT_STORAGE__*` | for documents | endpoint/keys/bucket |
| `SIP_FEATURES__PLUGIN_SYSTEM_ENABLED` | yes | otherwise no plugins are hosted |

Set `SIP_SERVER__PORT` to whatever the platform injects (Fly uses `8080`;
pass its `$PORT` through if the host supplies one).

> Leaving `SIP_SECURITY__CORS_ALLOWED_ORIGINS` empty makes the API fall back to
> **permissive CORS** and only log a warning. Set it explicitly.

## 3. Database

Create the database, then run migrations **before** starting the API: `sip-api`
does not migrate at boot, so a fresh database means every read 500s with
"relation does not exist".

```bash
sqlx migrate run --database-url 'postgres://...'
# or, with the image:
docker run --rm -e DATABASE_URL='postgres://...' sip-api \
  sqlx migrate run
```

The migrations require `uuid-ossp`, `vector`, `ltree` and `pg_trgm`. `postgis` is
present but commented out and not required. Confirm each extension is available on
your provider before migrating.

Then create the first administrator with `POST /api/v1/auth/bootstrap`. Set
`SIP_SECURITY__BOOTSTRAP_TOKEN` in the API host's environment and send the same
value in the `X-Bootstrap-Token` header. The request takes `organization_name`,
`name`, `email`, and a password of at least 12 characters. Use HTTPS for a hosted
deployment. See the [Quick Start](../../QUICKSTART.md#create-your-first-administrator)
for the request format.

The endpoint is disabled without a configured token and refuses setup after
users exist. Remove the token from the host configuration and restart the API
after setup. Fresh installations remove the old demo tenant through migration
`011`; there is no shared demo login.

## 4. CORS and preview deployments

SIP matches origins **exactly**: there is no wildcard support. Consequences:

- add `https://your-app.vercel.app` (and any custom domain) to
  `SIP_SECURITY__CORS_ALLOWED_ORIGINS`;
- **every preview deployment gets a random hostname, so previews will be blocked
  by CORS** unless you add that hostname too or point previews at the same API
  and accept the block.

If working previews matter, the change is small but touches security code: accept
a `*.vercel.app` style suffix entry via `AllowOrigin::predicate`. Ask before
assuming it exists: it does not today.

Login uses a bearer token in `localStorage`, not cookies, so cross-site
deployment (frontend on `vercel.app`, API on `api.example.com`) works without
`SameSite` problems.

## 5. Plugins on Vercel

No extra work. Plugin bundles are served by the **API**
(`/api/v1/ui/plugins/<id>/app/`), not by Vercel, so `plugins/*/dist` must ship in
the API image: it is tracked in git for exactly that reason. The shell frames the
API origin, and `buildMountUrl` resolves relative mounts against
`NEXT_PUBLIC_API_URL`, so the iframe works cross-origin.

## 6. Verifying a deployment

```bash
# build identity, not just "it responds"
curl -s https://api.example.com/api/v1/capabilities | jq '.data.build.commit'
curl -s https://api.example.com/api/v1/health/ready
curl -s https://api.example.com/api/v1/ui/plugins | jq '.data[].id'
```

`build.commit` must match the commit you deployed. A `demo_mode` flag or a 200 on
`/health` does not identify the running revision.

`demo_mode` must stay **off** in production: it enables test-only affordances.
It is disabled by default and by production validation.

## 7. Not supported by this layout

- **The API on Vercel.** Needs the container host described above.
- **Vercel Postgres as-is without checking extensions.** `vector`/`ltree` must
  exist before migrating.
- **MinIO.** Use S3 or R2.
- **Preview deploys without per-preview CORS entries.** See §4.
