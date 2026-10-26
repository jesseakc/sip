# SIP demo environment (isolated stack)

Runs SIP locally against the SMRT test corpus, with the Test Bench available as a
plugin, for a dedicated fictional customer tenant.

**Fully isolated from any other SIP deployment on this host.** It is its own
Compose project (`sip-demo`), with its own container names, its own named volumes
and its own host ports. It never starts, stops, or writes to another project's
containers, volumes or database.

## Prerequisites

* Docker reachable through `sudo`. This host's Docker socket is `root:root` and
  no `docker` group exists, so group-based access is not available and the
  startup command needs an interactive password.
* Python venv at `~/.venvs/sip-data` with `pyarrow`, `pandas`, `argon2-cffi`,
  `psycopg`.

## Layout

```
scripts/demo/
├── gen_env.py         generate this environment's secrets, accounts and demo.toml
├── provision.py       create the demo tenant + role accounts (idempotent, guarded reset)
├── load_fixture.py    import the SMRT corpus through the migration pipeline (idempotent)
└── README.md          this runbook

docker-compose.demo.yml   isolation overlay: own project, names, volumes, ports

~/sip-demo/               all generated secrets, mode 0700 (never in the repo)
├── secrets.env           infra secrets, consumed by --env-file
├── demo.toml             mounted over /app/Sip.toml (sets this stack's CORS origin)
├── accounts.json         demo accounts + argon2id hashes
├── credentials.txt       the private credential handoff sheet
└── state.json            tenant id, job ids, imported counts
```

> **Why `~/sip-demo` and not `~/.sip-demo`?** The Docker CLI here is a snap, and
> snap's `home` interface grants access to `$HOME` but **not to dot-prefixed
> paths**. A hidden runtime directory makes `--env-file` fail with a bare
> `permission denied`, so the runtime directory must not start with a dot.

## Ports

Only three bindings, **all loopback-only** — nothing is reachable from the LAN.

| Service | Host binding | Notes |
|---|---|---|
| Frontend | `127.0.0.1:13000` → 3000 | http://localhost:13000 |
| API | `127.0.0.1:18000` → 8000 | http://localhost:18000 |
| Postgres | `127.0.0.1:55432` → 5432 | loopback, non-default port, needed for passwordless provisioning |

The base compose file publishes 3000/8000/5432/6379/9000 on `0.0.0.0`. The overlay
replaces those bindings rather than adding to them (`!override` replaces an
inherited list; `!reset []` clears one — Compose ≥ 2.24). Appending would have
left the base bindings in place and collided with any already-running stack.

## First run

```bash
cd /home/jesseakc/sip

# 0. Build the loadable fixture from the committed raw corpus, if absent.
#    fixture/ is generated (not committed) because it is ~17 MB.
~/.venvs/sip-data/bin/python data/smrt-maintenance-logs/scripts/build_fixture.py

# 1. Generate secrets, the six demo accounts, and demo.toml.
~/.venvs/sip-data/bin/python scripts/demo/gen_env.py

# 2. Start the isolated stack. Needs an interactive sudo password, so run it from
#    a real terminal. Records the build identity so the running revision can be
#    confirmed later.
printf 'SIP_BUILD_COMMIT=%s\n' "$(git rev-parse --short HEAD)" > /home/jesseakc/sip-demo/build.env
sudo docker compose -p sip-demo \
  --env-file /home/jesseakc/sip-demo/secrets.env \
  --env-file /home/jesseakc/sip-demo/build.env \
  -f docker-compose.yml -f docker-compose.demo.yml \
  up -d --build 2>&1 | tee /tmp/sip-demo-up.log

# 3. Create the demo tenant and its accounts. No sudo needed.
~/.venvs/sip-data/bin/python scripts/demo/provision.py

# 4. Import the corpus (~12,144 records) in dependency order. No sudo needed.
~/.venvs/sip-data/bin/python scripts/demo/load_fixture.py
```

Validate the configuration without starting anything:

```bash
cd /home/jesseakc/sip && docker compose -p sip-demo \
  --env-file /home/jesseakc/sip-demo/secrets.env \
  -f docker-compose.yml -f docker-compose.demo.yml config
```

Steps 3 and 4 need no privileges beyond reading `~/sip-demo`, so they can be
re-run freely.

## Stopping and restarting without losing the dataset

Data lives in this project's own named volumes (`sip-demo-postgres-data`,
`sip-demo-redis-data`, `sip-demo-minio-data`), so a stop/start preserves
everything:

```bash
cd /home/jesseakc/sip

# stop, keeping data
sudo docker compose -p sip-demo \
  --env-file /home/jesseakc/sip-demo/secrets.env \
  -f docker-compose.yml -f docker-compose.demo.yml stop

# start again
sudo docker compose -p sip-demo \
  --env-file /home/jesseakc/sip-demo/secrets.env \
  -f docker-compose.yml -f docker-compose.demo.yml up -d
```

`down` also keeps the volumes. **`down -v` destroys the dataset** and is the only
command that does. Both `up` and `down` here are scoped to `-p sip-demo`, so they
cannot affect another stack — but note there is deliberately **no** bare
`docker compose down` anywhere in this runbook.

Re-running `provision.py` and `load_fixture.py` after a restart is safe: both
detect what already exists and skip it. Re-running `gen_env.py` reuses existing
secrets and passwords, so a restart never invalidates the handoff sheet.

## Re-running the import

```bash
# refuse-safe: requires the tenant slug twice, and verifies the recorded org id
~/.venvs/sip-data/bin/python scripts/demo/provision.py \
  --reset --confirm sip-demo-rail-services

~/.venvs/sip-data/bin/python scripts/demo/provision.py
~/.venvs/sip-data/bin/python scripts/demo/load_fixture.py
```

`--reset` will not proceed unless the tenant found under that slug is the one
recorded in `state.json`, so it cannot be aimed at another tenant. It deletes only
rows scoped to this tenant's organization id.

## What the overlay changes, and why

| Change | Reason |
|---|---|
| `name: sip-demo` + `-p sip-demo` | Separate Compose project. |
| `container_name: sip-demo-*` | The base file hardcodes `sip-postgres`, `sip-api`, etc. Container names are global, so a second project would collide. |
| Explicit volume `name:` | Isolation must not depend on `-p` being passed correctly every time. |
| `!override` ports | Replaces the base bindings instead of appending, so 3000/8000/5432/6379/9000 are not published by this stack. |
| Postgres on `127.0.0.1:55432` | Provisioning needs a new tenant's first administrator inserted directly (see below), and `sudo docker exec` per run is avoidable if the database is reachable on loopback. |
| `demo.toml` mount | Sets CORS to this stack's frontend origin. Expressing a list through an environment variable would depend on figment parsing it out of a string; TOML is unambiguous. |
| `SIP_BUILD_COMMIT` build arg | Baked into the image and surfaced at `GET /api/v1/capabilities` as `build.commit`, so readiness is judged on build identity plus service health rather than on a feature flag. |
| `CARGO_BUILD_JOBS=2` | This host is memory-constrained; fewer parallel `rustc` jobs avoids thrashing swap. |

## Security notes

* **One direct SQL write.** `provision.py` inserts the tenant's *first*
  administrator with SQL, because `POST /users` always scopes a new user to the
  caller's organization and there is no API path for one tenant's administrator to
  create a user inside another tenant. Every other account goes through
  `POST /users`, so SIP hashes the password and enforces `user:manage` itself.
* **Hashes are generated outside SIP.** `gen_env.py` produces argon2id hashes with
  `Argon2::default()`'s exact parameters (Argon2id, v19, m=19456, t=2, p=1). That
  compatibility is asserted by `crates/sip-auth/tests/external_hash_compat.rs`,
  which fails if SIP stops accepting them.
* **Tenant isolation is SIP's own.** The demo tenant is an ordinary tenant, so
  every tenant-scoped table is protected by the same row-level security as any
  other.
* **Secrets stay out of the repository.** Generated secrets live in `~/sip-demo`
  (mode 0700). Nothing generated is written under the working tree.

## Known limitation

Imported work orders record historical part usage *without* decrementing
`parts.quantity_on_hand`: back-filling a repair from 2020 must not drain today's
inventory. The importer inserts part-usage rows directly for that reason.

Rollback archives assets and work orders, but locations, asset types, parts,
manufacturers and models have no archive operation in SIP, so a rollback reports
those as `rollback_skipped` rather than removing them. A partial rollback is
visible in the report instead of being reported as complete.
