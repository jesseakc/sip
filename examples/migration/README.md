# Weekend Testing Package

## What's Included

| File | Records | Import Target |
|------|---------|---------------|
| locations.csv | 5 locations | Location (Site → Building → Floor) |
| assets.csv | 5 assets | Asset (with serial, status, criticality) |
| work_orders.csv | 5 work orders | Work Order (with priority, status, asset ref) |
| assets.json | 5 assets | Asset (JSON format alternative) |

## Test Flow

### Test 1: Import Locations
1. Create migration job (source = csv, object = location)
2. Upload examples/migration/locations.csv
3. Map fields: name → name, location_type → type
4. Validate — should pass with 0 errors
5. Dry run — should show 5 records to create
6. Execute import
7. Verify: Locations page shows 5 new records

### Test 2: Import Assets
1. Create migration job (source = csv, object = asset)
2. Upload examples/migration/assets.csv
3. Map fields: name → name, serial_number → serial_number, status → status
4. Validate — 0 errors expected
5. Dry run
6. Execute import
7. Verify: Assets page shows new records

### Test 3: Import Work Orders
1. Create migration job (source = csv, object = work_order)
2. Upload examples/migration/work_orders.csv
3. Map fields: title → title, description → description, priority → priority
4. Validate
5. Dry run
6. Execute import
7. Verify: Work Orders page shows new records

### Test 4: Rollback
1. Open the completed import job
2. Click "Rollback"
3. Verify imported records are removed from Assets/Work Orders

### Test 5: Idempotency
1. Run the same import job again (re-execute)
2. Verify no duplicate records are created (external_id_map prevents this)
