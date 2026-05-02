#!/bin/bash
set -euo pipefail

BASE="http://localhost:8000/api/v1"
PASS=0
FAIL=0

pass() { echo "PASS"; PASS=$((PASS + 1)); }
fail() { echo "FAIL"; FAIL=$((FAIL + 1)); }
fail_exit() { echo "FAIL"; echo "  $1"; exit 1; }

echo "=== SIP Smoke Test ==="
echo ""

# ── Health (critical) ────────────────────────────────────────────────
echo -n "Health: "
curl -sf "$BASE/health" > /dev/null && pass || fail_exit "API unreachable"

echo -n "Ready: "
curl -sf "$BASE/health/ready" > /dev/null && pass || fail

# ── Login (critical) ─────────────────────────────────────────────────
echo -n "Login: "
LOGIN=$(curl -sf -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@acme.local","password":"password"}')
TOKEN=$(echo "$LOGIN" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
if [ -n "$TOKEN" ]; then
  pass
else
  fail_exit "Could not extract token from login response"
fi

# ── Auth (informational) ─────────────────────────────────────────────
echo -n "Auth/me: "
curl -sf "$BASE/auth/me" -H "Authorization: Bearer $TOKEN" > /dev/null && pass || fail

# ── Core endpoints (informational) ────────────────────────────────────
echo -n "Assets: "
curl -sf "$BASE/assets" -H "Authorization: Bearer $TOKEN" > /dev/null && pass || fail

echo -n "Plugins: "
curl -sf "$BASE/plugins" > /dev/null && pass || fail

echo -n "Navigation: "
NAV=$(curl -sf "$BASE/ui/navigation")
if echo "$NAV" | grep -q "migration-studio"; then
  pass
else
  echo "WARN: Migration Studio not in nav"
  FAIL=$((FAIL + 1))
fi

# ── Migration pipeline (critical) ─────────────────────────────────────
echo ""
echo "--- Migration Pipeline ---"

echo -n "Create job: "
JOB=$(curl -sf -X POST "$BASE/migrations" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Smoke Test","source_system":"csv","source_object_type":"asset"}')
JOB_ID=$(echo "$JOB" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
if [ -n "$JOB_ID" ]; then pass; else fail_exit "Migration job creation failed"; fi

echo -n "Upload records: "
curl -sf -X POST "$BASE/migrations/$JOB_ID/source-records" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"records":[
    {"name":"Smoke Pump","serial_number":"SMOKE-001","status":"operational","criticality":"medium"},
    {"name":"Smoke Motor","serial_number":"SMOKE-002","status":"operational","criticality":"high"}
  ]}' > /dev/null && pass || fail_exit "Source record upload failed"

echo -n "Save mappings: "
curl -sf -X POST "$BASE/migrations/$JOB_ID/mappings" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"mappings":[
    {"target_entity_type":"asset","source_field":"name","target_field":"name","is_required":true},
    {"target_entity_type":"asset","source_field":"serial_number","target_field":"serial_number","is_required":false},
    {"target_entity_type":"asset","source_field":"status","target_field":"status","is_required":false},
    {"target_entity_type":"asset","source_field":"criticality","target_field":"criticality","is_required":false}
  ]}' > /dev/null && pass || fail_exit "Mapping save failed"

echo -n "Validate: "
curl -sf -X POST "$BASE/migrations/$JOB_ID/validate" \
  -H "Authorization: Bearer $TOKEN" > /dev/null && pass || fail_exit "Validation failed"

echo -n "Dry run: "
curl -sf -X POST "$BASE/migrations/$JOB_ID/dry-run" \
  -H "Authorization: Bearer $TOKEN" > /dev/null && pass || fail_exit "Dry run failed"

echo -n "Import: "
IMPORT=$(curl -sf -X POST "$BASE/migrations/$JOB_ID/import" \
  -H "Authorization: Bearer $TOKEN")
if echo "$IMPORT" | grep -q '"data"'; then
  pass
else
  fail_exit "Import failed"
fi

echo -n "Verify import: "
ASSETS=$(curl -sf "$BASE/assets" -H "Authorization: Bearer $TOKEN")
if echo "$ASSETS" | grep -q "Smoke Pump"; then
  pass
else
  fail_exit "Imported assets not found"
fi

echo -n "Rollback: "
curl -sf -X POST "$BASE/migrations/$JOB_ID/rollback" \
  -H "Authorization: Bearer $TOKEN" > /dev/null && pass || fail_exit "Rollback failed"

echo -n "Verify rollback: "
ASSETS_AFTER=$(curl -sf "$BASE/assets" -H "Authorization: Bearer $TOKEN")
if echo "$ASSETS_AFTER" | grep -q "Smoke Pump"; then
  echo "WARN: Asset still visible after rollback"
  FAIL=$((FAIL + 1))
else
  pass
fi

# ── Idempotency test ──────────────────────────────────────────────────
echo -n "Re-import (idempotent): "
curl -sf -X POST "$BASE/migrations/$JOB_ID/import" \
  -H "Authorization: Bearer $TOKEN" > /dev/null && pass || fail

echo ""
echo "=== Results: $PASS passed, $FAIL failed ==="
if [ "$FAIL" -gt 0 ]; then
  echo "SMOKE TEST FAILED"
  exit 1
else
  echo "SMOKE TEST PASSED"
  exit 0
fi
