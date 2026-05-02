# Generic CMMS Asset Import Example

## File: assets.csv

Contains 4 asset records designed to test CSV parsing robustness.

| Record | Name | Serial | Special Feature |
|--------|------|--------|-----------------|
| 1 | Chiller Plant C2 | CH-C2-2024 | Clean record with all fields |
| 2 | Generator G2 | GEN-G2-2021 | Clean record |
| 3 | Pump Station 4 | PS-004-2023 | **Description contains a quoted comma**: `"Cooling water pump, 50HP"` — tests CSV parser's quoted field handling |
| 4 | HVAC Rooftop Unit 2 | HVAC-R2-2022 | Has `notes` field with semicolons |

## Field Mapping

Map these source columns to SIP canonical fields:

| Source Field | SIP Target | Required | Notes |
|-------------|-----------|----------|-------|
| `name` | `name` | Yes | Asset name |
| `serial_number` | `serial_number` | No | Unique identifier |
| `description` | `description` | No | May contain quoted commas |
| `status` | `status` | No | operational / maintenance / degraded |
| `criticality` | `criticality` | No | critical / high / medium / low |
| `purchase_date` | `purchase_date` | No | ISO date format |
| `warranty_expiry` | `warranty_expiry` | No | ISO date format |
| `tags` | `tags` | No | Comma-separated tags |
| `notes` | `notes` | No | Free-text notes |

## Expected Results

| Stage | Count | Notes |
|-------|-------|-------|
| Source records | 4 | |
| Validated records | 4 | All records should pass validation |
| Dry run: records to create | 4 | |
| Import: records created | 4 | |
| Import: records updated | 0 | First import creates, doesn't update |
| Import: records skipped | 0 | |
| Import: errors | 0 | |

## Testing the Quoted Comma

Record 3 (Pump Station 4) has: `"Cooling water pump, 50HP"` as its description.
When properly parsed, this should appear as `Cooling water pump, 50HP` (without the quotes, comma preserved).
If the CSV parser is broken, the comma inside the quotes will split the record incorrectly and the description will be truncated.

## Rollback

After import, rollback should archive all 4 assets. They'll no longer appear in the Assets list.
Re-import should recreate them with the same external IDs (idempotent).
