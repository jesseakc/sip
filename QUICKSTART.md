# Quick Start

Run SIP locally, create your own administrator, and start adding service records.
For hosted deployments, use the [deployment guide](docs/deployment/vercel.md).

## Before you start

You need Git, Docker, Docker Compose v2, and enough memory to build the Rust API
and Next.js frontend. Local AI needs additional memory. The setup-token example
below also uses OpenSSL.

Fresh installations have no demo accounts or customer records. Migration
[`011_remove_demo_seed_data.sql`](migrations/011_remove_demo_seed_data.sql)
removes the old Acme tenant. The old shared demo credentials no longer apply.

## Start SIP

```bash
git clone https://github.com/jesseakc/sip.git
cd sip
cp .env.example .env
docker compose up -d --build
```

If you already cloned the repository, run the remaining commands from its root.
The first build can take several minutes. Compose starts PostgreSQL, Redis,
MinIO, the API, and the frontend, and runs database migrations before the API.
AI is disabled by default.

Check startup:

```bash
docker compose ps
docker compose logs db-migrate sip-api
curl --fail http://localhost:8000/api/v1/health/ready
```

The web interface is at [localhost:3000](http://localhost:3000). Complete the
administrator setup below before signing in.

## Create your first administrator

On an empty database, SIP provides a setup endpoint protected by a one-time
token. The token is an operator secret, separate from the administrator password.

Generate a token in your terminal, then apply the supplied local setup overlay:

```bash
export SIP_SETUP_TOKEN="$(openssl rand -hex 32)"
docker compose -f docker-compose.yml \
  -f docs/examples/first-run.compose.yml up -d sip-api
```

This passes the token to the API as `SIP_SECURITY__BOOTSTRAP_TOKEN`. The base
Compose file does not pass it through automatically. Simply adding
`SIP_BOOTSTRAP_TOKEN` to `.env` does not configure this endpoint.

Wait for the API to be ready again:

```bash
curl --fail http://localhost:8000/api/v1/health/ready
```

In the same terminal, submit your details. Replace the example values below,
including the password, before running it. Use a unique password of at least
12 characters.

```bash
curl --fail-with-body http://localhost:8000/api/v1/auth/bootstrap \
  -H "Content-Type: application/json" \
  -H "X-Bootstrap-Token: $SIP_SETUP_TOKEN" \
  --data-binary @- <<'JSON'
{
  "organization_name": "Your Service Team",
  "name": "Your Name",
  "email": "you@example.com",
  "password": "replace-with-your-own-unique-password"
}
JSON
```

A successful response includes the new organization and user IDs. Bootstrap
refuses further setup once users exist. Remove the temporary token from the
running API by recreating it with the base configuration:

```bash
unset SIP_SETUP_TOKEN
docker compose up -d --force-recreate sip-api
```

Open [localhost:3000](http://localhost:3000) and sign in with the account you just
created. There is no shared default login.

## Add your data

Create equipment records through the interface or use **Migration Studio** to
import CSV or JSON. Start with the [migration overview](docs/migrations/overview.md)
for supported object types and dependency order. Create referenced locations,
asset types, and models before records that depend on them.

The [Migration Studio guide](plugins/sip-migration-studio/README.md) explains
field mapping, validation, dry runs, import reports, and rollback. Review the
report after each import; rollback support varies by object type.

Optional dataset tools are documented in
[`data/smrt-maintenance-logs/README.md`](data/smrt-maintenance-logs/README.md).
External datasets are loaded separately from installation.

## Enable AI later

SIP's maintenance features can run with AI disabled. When you are ready, choose
a provider and configure it using the [configuration guide](docs/development.md#configuration).
Starting the Ollama profile alone does not enable AI in the API.

## Local service addresses

- [Web interface](http://localhost:3000)
- [API health](http://localhost:8000/api/v1/health)
- [API readiness](http://localhost:8000/api/v1/health/ready)
- [MinIO console](http://localhost:9001)

The base Compose configuration uses development credentials and publishes local
service ports. Use the [deployment guide](docs/deployment/vercel.md) when preparing
a hosted environment.

## Troubleshooting

**A port is already in use.** Check `docker compose ps` and identify what owns the
port before changing its mapping or stopping that service.

**The API cannot connect to the database.** Check `docker compose logs postgres db-migrate sip-api`.
The migration job must finish successfully before the API starts.

**Bootstrap returns 403.** Check that the setup overlay was applied, the API has
restarted, and the request uses the same token exported in your terminal.

**Bootstrap returns 409.** Users already exist. Sign in with an existing account;
bootstrap cannot create another first administrator.

**Login returns 401.** Use the account you created. Old Acme demo credentials
are no longer valid. For an expired session, sign out and sign in again.

**AI is unavailable.** Check `/api/v1/capabilities` and the API logs. Provider
configuration, the AI-enabled setting, and the chat feature flag must agree.

## Stop SIP

```bash
docker compose down
```

This keeps the named data volumes. To intentionally erase this local installation,
including its accounts, imported records, and stored files, run
`docker compose down -v`. The next start requires administrator setup again.

[All documentation](docs/README.md)
