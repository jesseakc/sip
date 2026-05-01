# SIP Testing Status — May 2026

> Honest state of the repo before weekend hands-on testing.

## Commands Run in CI (this environment)

```bash
# Backend
cargo fmt --all -- --check        # Reformatting applied
cargo clippy --workspace --all-features -- -D warnings  # PASS, 0 errors
cargo test --workspace --all-features                    # PASS, 103 tests, 0 failures

# Frontend
cd frontend && npx tsc --noEmit   # PASS, 0 errors
cd frontend && npm run build      # PASS, static + dynamic pages

# Docker config
docker compose config             # Not run (no Docker in this env)
docker compose up --build         # Not run (no Docker in this env)
```

## What Builds

- ✅ Full Rust workspace (22+ crates)
- ✅ Frontend (Next.js 15, TypeScript)
- ✅ All 103 tests pass
- ✅ Clippy with `-D warnings` clean

## What Has NOT Been Verified At Runtime

- ❌ Docker Compose startup (no Docker in CI)
- ❌ Database migration execution in Docker
- ❌ Plugin manifest loading at runtime in Docker
- ❌ Migration Studio full import workflow (needs runtime)
- ❌ Auth login/logout with real Postgres + JWT
- ❌ Ollama model pull and AI chat
- ❌ Rollback of imported records
- ❌ Idempotent re-import

## Known Issues

| # | Issue | Severity | Status |
|---|-------|----------|--------|
| 1 | Migration route mismatch between frontend and backend | FIXED | Frontend now calls `/migrations` (not `/migrations/jobs`) |
| 2 | Docker db-migrate used localhost URL inside container | FIXED | Now uses `postgres://sip:sip@postgres:5432/sip` |
| 3 | Ollama AI enabled by default in compose | FIXED | Now `SIP_AI_ENABLED=false`, `SIP_AI_PROVIDER=disabled` |
| 4 | Plugin manifests not copied into Docker runtime image | FIXED | Added `COPY plugins ./plugins` in Dockerfile.api |
| 5 | Migration status enums used Rust Debug formatting | FIXED | Added `#[serde(rename_all = "snake_case")]` to all 3 enums |
| 6 | Asset state machine missing Degraded→Retired transition | FIXED | Added transition, all 103 tests pass |
| 7 | `/auth/me` returned bare JSON without `data` envelope | FIXED | Now returns `{ data: { id, email, name, role, org_id, permissions } }` |

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

### Step 8: Save field mappings
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
# Expected: job status becomes "validated" or "ready_for_import"
```

### Step 10: Dry Run
```bash
curl -X POST http://localhost:8000/api/v1/migrations/${JOB_ID}/dry-run \
  -H "Authorization: Bearer $TOKEN"
# Expected: shows records_to_create: 2
```

### Step 11: Execute Import
```bash
curl -X POST http://localhost:8000/api/v1/migrations/${JOB_ID}/import \
  -H "Authorization: Bearer $TOKEN"
# Expected: job status becomes "completed", imported records appear in Assets
```

### Step 12: Verify records
```bash
curl http://localhost:8000/api/v1/assets?search=Test \
  -H "Authorization: Bearer $TOKEN" | jq '.data | length'
# Expected: 2 (Test Pump and Test Motor)
```

### Step 13: Rollback
```bash
curl -X POST http://localhost:8000/api/v1/migrations/${JOB_ID}/rollback \
  -H "Authorization: Bearer $TOKEN"
# Expected: job status becomes "rolled_back", Test Pump and Test Motor are archived
```

### Step 14: Repeat with CSV file (via UI)
1. Open http://localhost:3000/migration-studio
2. Click "New Migration Job"
3. Name: "CSV Asset Import", Source: CSV, Object: Asset
4. Upload `examples/migration/assets.csv`
5. Click Create
6. Map fields: name→name, serial_number→serial_number, status→status
7. Validate → Dry Run → Execute → Verify

## Current Limitations for This Weekend

- Migration Studio import currently only handles `asset` entity type in the backend execute_import. Work order and location imports go through the staging pipeline but the service doesn't convert them to real entities yet.
- AI chat requires a running Ollama instance (`--profile ollama`). Without it, the AI Chat nav item still appears but API calls will fail gracefully.
- No pagination on migration source records, staged records, issues, or external ID maps.
- Frontend uses a 5-second timeout on navigation fetch; falls back to hardcoded nav if API is unavailable.
- Auth refresh token handling is basic — long-lived sessions may expire without auto-refresh.
