# Weekend Testing Guide

## Prerequisites
- Docker and Docker Compose v2+
- 8 GB RAM free (no AI model needed for migration testing)

## 1. Fresh Install

```bash
git clone https://github.com/jesseakc/sip.git
cd sip
cp .env.example .env
```

## 2. Start (AI Disabled — Fastest Path)

```bash
docker compose down -v      # clean slate
docker compose up --build   # builds + starts everything
```

Wait for `db-migrate` to complete, then `sip-api` to print `SIP API listening on 0.0.0.0:8000`.

## 3. Verify Health

```bash
curl http://localhost:8000/api/v1/health
# Expected: {"status":"ok"}
```

Open http://localhost:3000 — SIP frontend should load.

## 4. Login

| Role | Email | Password |
|------|-------|----------|
| Admin | admin@acme.local | password |
| Manager | manager@acme.local | password |

## 5. Run Preflight

```bash
./scripts/preflight.sh
```

## 6. Run Smoke Test

```bash
./scripts/smoke.sh
```

This tests the full API pipeline: health → login → create migration → upload records → map → validate → dry run → import → verify → rollback.

## 7. Manual Migration Test (via UI)

### 7.1 Open Migration Studio
Click **Migration Studio** in the sidebar.

### 7.2 Create Migration Job
- Click **New Migration Job**
- Name: `Weekend Test`
- Source System: `CSV File`
- Object Type: `Asset / Equipment`
- Upload: `plugins/sip-migration-studio/examples/generic-cmms/assets.csv`
- Click **Create Migration Job**

### 7.3 Check Source Records
You should see the detected fields: `name`, `serial_number`, `description`, `status`, `criticality`, `purchase_date`, `warranty_expiry`, `tags`, `notes`.

### 7.4 Map Fields
Go to the **Mapping** tab. Map at minimum:
- `name` → `name`
- `serial_number` → `serial_number`
- `status` → `status`
- `description` → `description`

Click **Save & Validate**.

### 7.5 Validate
Go to **Validation** tab, click **Validate**.
Expected: 0 errors (or warnings about unused source fields — that's fine).

### 7.6 Dry Run
Go to **Dry Run** tab, click **Run Dry Run**.
Expected: `source_record_count = 4`, `valid_record_count = 4`.

### 7.7 Execute Import
Go to **Report** tab, click **Execute Import**.
Expected: `imported_record_count = 4`, status becomes `completed`.

### 7.8 Verify Imported Assets
Go to **Assets** in the sidebar. The imported assets should appear:
- Chiller Plant C2
- Generator G2
- Pump Station 4 (quoted comma: `"Cooling water pump, 50HP"`)
- HVAC Rooftop Unit 2

### 7.9 Rollback
On the migration job page, click **Rollback**.
Expected: status becomes `rolled_back`. Imported assets are archived (no longer visible in Assets list).

### 7.10 Test Idempotency
Re-execute the same import: click **Execute Import** again.
Expected: the same 4 records are re-imported (external_id_map prevents duplicates).
Verify assets appear again. Then rollback again.

## 8. API-Only Migration Test

```bash
TOKEN=$(curl -s -X POST http://localhost:8000/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@acme.local","password":"password"}' | jq -r '.token')

# Create job
JOB=$(curl -s -X POST http://localhost:8000/api/v1/migrations \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"API Test","source_system":"csv","source_object_type":"asset"}')
JOB_ID=$(echo $JOB | jq -r '.data.id')

# Upload records
curl -s -X POST http://localhost:8000/api/v1/migrations/$JOB_ID/source-records \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"records":[{"name":"API Pump","serial_number":"API-001"},{"name":"API Motor","serial_number":"API-002"}]}'

# Map fields
curl -s -X POST http://localhost:8000/api/v1/migrations/$JOB_ID/mappings \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"mappings":[{"target_entity_type":"asset","source_field":"name","target_field":"name","is_required":true}]}'

# Validate
curl -s -X POST http://localhost:8000/api/v1/migrations/$JOB_ID/validate \
  -H "Authorization: Bearer $TOKEN"

# Dry run
curl -s -X POST http://localhost:8000/api/v1/migrations/$JOB_ID/dry-run \
  -H "Authorization: Bearer $TOKEN"

# Import
curl -s -X POST http://localhost:8000/api/v1/migrations/$JOB_ID/import \
  -H "Authorization: Bearer $TOKEN" | jq .

# Verify
curl -s http://localhost:8000/api/v1/assets \
  -H "Authorization: Bearer $TOKEN" | jq '.data | length'

# Rollback
curl -s -X POST http://localhost:8000/api/v1/migrations/$JOB_ID/rollback \
  -H "Authorization: Bearer $TOKEN"
```

## Known Limitations

- Migration import currently creates `asset` entities only. Work order and location imports go through staging but aren't converted to real entities yet.
- No pagination on migration endpoints. Large datasets (>100 records) may be slow.
- CSV parsing handles quoted commas and escaped quotes but not multiline fields.
- AI chat requires `--profile ollama` and a pulled model. Not needed for migration testing.
- Auth token refresh is basic — sessions may expire after 15 minutes of inactivity.
