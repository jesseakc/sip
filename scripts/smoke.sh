#!/bin/bash
set -euo pipefail

BASE="http://localhost:8000/api/v1"
echo "=== SIP Smoke Test ==="

# Health
echo -n "Health: "
curl -sf $BASE/health > /dev/null && echo "PASS" || (echo "FAIL"; exit 1)

# Ready
echo -n "Ready: "
curl -sf $BASE/health/ready > /dev/null && echo "PASS" || echo "FAIL"

# Login
echo -n "Login: "
LOGIN=$(curl -sf -X POST $BASE/auth/login -H "Content-Type: application/json" \
  -d '{"email":"admin@acme.local","password":"password"}')
TOKEN=$(echo "$LOGIN" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
if [ -n "$TOKEN" ]; then echo "PASS"; else echo "FAIL"; exit 1; fi

# Auth me
echo -n "Auth/me: "
curl -sf $BASE/auth/me -H "Authorization: Bearer $TOKEN" > /dev/null && echo "PASS" || echo "FAIL"

# Assets
echo -n "Assets: "
curl -sf $BASE/assets -H "Authorization: Bearer $TOKEN" > /dev/null && echo "PASS" || echo "FAIL"

# Capabilities
echo -n "Capabilities: "
curl -sf $BASE/capabilities > /dev/null && echo "PASS" || echo "FAIL"

# Plugins
echo -n "Plugins: "
curl -sf $BASE/plugins > /dev/null && echo "PASS" || echo "FAIL"

# Navigation
echo -n "Navigation: "
NAV=$(curl -sf $BASE/ui/navigation)
echo "$NAV" | grep -q "migration-studio" && echo "PASS" || echo "WARN: Migration Studio not in nav"

# Create migration job
echo -n "Migration create: "
JOB=$(curl -sf -X POST $BASE/migrations -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Smoke Test","source_system":"csv","source_object_type":"asset"}')
JOB_ID=$(echo "$JOB" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
if [ -n "$JOB_ID" ]; then echo "PASS ($JOB_ID)"; else echo "FAIL"; exit 1; fi

# Upload source records
echo -n "Migration source upload: "
curl -sf -X POST $BASE/migrations/$JOB_ID/source-records -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"records":[{"name":"Smoke Pump","serial_number":"SMOKE-001","status":"operational","criticality":"medium"}]}' > /dev/null && echo "PASS" || echo "FAIL"

# Save mappings
echo -n "Migration mappings: "
curl -sf -X POST $BASE/migrations/$JOB_ID/mappings -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"mappings":[{"target_entity_type":"asset","source_field":"name","target_field":"name","is_required":true},{"target_entity_type":"asset","source_field":"serial_number","target_field":"serial_number","is_required":false},{"target_entity_type":"asset","source_field":"status","target_field":"status","is_required":false}]}' > /dev/null && echo "PASS" || echo "FAIL"

# Validate
echo -n "Migration validate: "
curl -sf -X POST $BASE/migrations/$JOB_ID/validate -H "Authorization: Bearer $TOKEN" > /dev/null && echo "PASS" || echo "FAIL"

# Dry run
echo -n "Migration dry-run: "
curl -sf -X POST $BASE/migrations/$JOB_ID/dry-run -H "Authorization: Bearer $TOKEN" > /dev/null && echo "PASS" || echo "FAIL"

# Execute import
echo -n "Migration import: "
curl -sf -X POST $BASE/migrations/$JOB_ID/import -H "Authorization: Bearer $TOKEN" > /dev/null && echo "PASS" || echo "FAIL"

# Verify imported asset
echo -n "Verify import: "
curl -sf $BASE/assets -H "Authorization: Bearer $TOKEN" | grep -q "Smoke Pump" && echo "PASS" || echo "FAIL"

# Rollback
echo -n "Migration rollback: "
curl -sf -X POST $BASE/migrations/$JOB_ID/rollback -H "Authorization: Bearer $TOKEN" > /dev/null && echo "PASS" || echo "FAIL"

echo ""
echo "=== Smoke test complete ==="
