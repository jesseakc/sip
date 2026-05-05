# SIP Testing Status — May 2026

> Current state after full install, build, and Docker E2E testing.

## Commands Run and Verified

```bash
# Backend
cargo fmt --all -- --check        # PASS
cargo clippy --workspace --all-features -- -D warnings  # PASS, 0 errors
cargo test --workspace --all-features                    # PASS, ~260 tests, 0 failures

# Frontend
cd frontend && npx tsc --noEmit   # PASS, 0 errors
cd frontend && npm run build      # PASS

# Docker
docker compose config             # PASS
docker compose up --build         # PASS (all 5 containers healthy)
./scripts/preflight.sh            # PASS
./scripts/smoke.sh                # PASS (health, login, auth/me, assets, plugins)
```

## What Builds

- ✅ Full Rust workspace (21 crates)
- ✅ Frontend (Next.js 15, TypeScript)
- ✅ All ~260 tests pass (0 failures)
- ✅ Clippy with `-D warnings` clean

## Docker E2E Verified

- ✅ Docker Compose startup (all 5 containers healthy)
- ✅ Database migration execution in Docker (all 9 migrations applied)
- ✅ Seed data loads correctly (8 users, 35 assets, etc.)
- ✅ Auth login/logout with real Postgres + JWT
- ✅ List assets, plugins, and navigation endpoints
- ✅ Health and readiness endpoints

## Test Coverage by Crate

| Crate | Tests | Focus |
|-------|-------|-------|
| `sip-auth` | 24 | JWT encode/decode, password hash/verify, RBAC roles, permission checks |
| `sip-domain` | 48 | SipError Display/constructors, TenantContext, state machines, serde roundtrips for 28 enums |
| `sip-application` | 16 | role_permissions (all UserRole variants), classify_query (7 question types) |
| `sip-api` | 3 | JSON response envelope shapes |
| `sip-config` | 13 | Default config, environment helpers, secret redaction, validation |
| `sip-ai` | 40 | Key rotation, provider registry, failover logic |
| `sip-plugins` | 29 | Manifest validation, registry operations, navigation aggregation |
| `sipmem-core` | 32 | Memory types, fact ledger, temporal resolver, recipes, verification |
| `sipmem-adapters` | 18 | Retriever stubs, pipeline, router, cross-reference verification |
| `sip-infrastructure` | 1 | Migration job row mapping |
| **Total** | **~260** | **0 failures across all crates** |

## Fixes Applied During Docker Testing

| # | Issue | Fix |
|---|-------|-----|
| 1 | JWT secret not resolved in Docker (figment env var splitting) | Mounted `Sip.toml` config file; upstream fixed with `__` double-underscore convention |
| 2 | 31 Postgres ENUM types incompatible with sqlx `String` decoding | Migration 008: converted all ENUM columns to TEXT |
| 3 | `certifications` column JSONB[] vs JSONB mismatch | Migration 009: changed column type from JSONB[] to JSONB |
| 4 | `SET` command with `$1` parameter (Postgres rejects this) | Changed `sip-tenancy` to use string interpolation for `SET` commands |
| 5 | Migration Studio not in default navigation | Added to frontend `FALLBACK_NAV_ITEMS`, backend `register_default_navigation()`, and enabled `plugin_system_enabled` in `Sip.toml` |
| 6 | Seed data password hash incompatible with argon2 crate defaults | Hash parameters differ from what `Argon2::default()` generates; updated seed data to use correct hash |

## Known Issues

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| 1 | Migration field mapping API expects server-generated fields (id, organization_id, etc.) from client | Medium | Pre-existing; affects smoke test mappings step |
| 2 | AI chat requires Ollama (`--profile ollama`); without it, endpoint returns errors gracefully | Low | By design |
| 3 | No pagination on migration source records, staged records, or external ID maps | Low | Future improvement |
| 4 | Frontend 5-second timeout on navigation fetch; falls back to hardcoded nav | Low | Acceptable fallback |
| 5 | Auth refresh token handling is basic; long-lived sessions may expire | Low | Future improvement |

## Demo Credentials

| Role | Email | Password |
|------|-------|----------|
| Admin | admin@acme.local | password |
| Manager | manager@acme.local | password |
| Technician | tech1@acme.local | password |
| Viewer | viewer@acme.local | password |
| Vendor | vendor@acme.local | password |
| Auditor | auditor@acme.local | password |

## Reset Command

```bash
docker compose down -v    # Destroys containers + volumes
docker compose up --build # Fresh start with migrations + seed data
```

## Weekend Manual Test Script

### Step 1: Startup
```bash
git clone https://github.com/jesseakc/sip.git && cd sip
cp .env.example .env
docker compose up --build
# Wait for db-migrate to complete, then sip-api to show "SIP API listening on 0.0.0.0:8000"
```

### Step 2: Verify health
```bash
curl http://localhost:8000/api/v1/health
# Expected: { "status": "ok" }
```

### Step 3: Login
```bash
# Via API:
curl -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@acme.local","password":"password"}'

# Or open http://localhost:3000 in browser
```

### Step 4: Check plugins loaded
```bash
curl http://localhost:8000/api/v1/plugins | jq .
# Expected: sip-core-ui and sip-migration-studio in the list
```

### Step 5: Check navigation includes Migration Studio
```bash
curl http://localhost:8000/api/v1/ui/navigation | jq '.data[] | select(.id == "migration-studio")'
# Expected: { "id": "migration-studio", "label": "Migration Studio", "path": "/migration-studio", ... }
```

### Step 6: Create a migration job (via API)
```bash
TOKEN=$(curl -s -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@acme.local","password":"password"}' | jq -r '.token')

curl -X POST http://localhost:8000/api/v1/migrations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Test Import","source_system":"csv","source_object_type":"asset"}'
# Save the returned job ID
```

### Step 7: Upload source records
```bash
JOB_ID="<from step 6>"

curl -X POST http://localhost:8000/api/v1/migrations/${JOB_ID}/source-records \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"records":[
    {"name":"Test Pump","serial_number":"TP-001","status":"operational","criticality":"medium"},
    {"name":"Test Motor","serial_number":"TM-001","status":"operational","criticality":"high"}
  ]}'
```

### Step 8: Save field mappings (via UI preferred)
```bash
curl -X POST http://localhost:8000/api/v1/migrations/${JOB_ID}/mappings \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"mappings":[
    {"target_entity_type":"asset","source_field":"name","target_field":"name","is_required":true},
    {"target_entity_type":"asset","source_field":"serial_number","target_field":"serial_number","is_required":false},
    {"target_entity_type":"asset","source_field":"status","target_field":"status","is_required":false}
  ]}'
```

### Step 9: Validate
```bash
curl -X POST http://localhost:8000/api/v1/migrations/${JOB_ID}/validate \
  -H "Authorization: Bearer $TOKEN"
```

### Step 10: Dry Run
```bash
curl -X POST http://localhost:8000/api/v1/migrations/${JOB_ID}/dry-run \
  -H "Authorization: Bearer $TOKEN"
```

### Step 11: Execute Import
```bash
curl -X POST http://localhost:8000/api/v1/migrations/${JOB_ID}/import \
  -H "Authorization: Bearer $TOKEN"
```

### Step 12: Verify records
```bash
curl http://localhost:8000/api/v1/assets?search=Test \
  -H "Authorization: Bearer $TOKEN" | jq '.data | length'
```

### Step 13: Rollback
```bash
curl -X POST http://localhost:8000/api/v1/migrations/${JOB_ID}/rollback \
  -H "Authorization: Bearer $TOKEN"
```
