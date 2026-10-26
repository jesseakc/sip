# Migration Core Framework

The Migration Core Framework is the infrastructure that enables migration plugins to safely import external CMMS/CMMS data (assets, work orders, parts, locations, etc.) into SIP. It provides a structured pipeline from source data ingestion through validation, mapping, import, and rollback.

## Why It Exists

Organizations migrating from legacy CMMS platforms need to bring their operational history with them. The migration framework provides:

- **Structured pipeline**: Source → Mapped → Validated → Imported → Auditable
- **Canonical import contracts**: Standardized DTOs that plugins translate source data into
- **Full traceability**: Source external IDs are mapped to SIP entity IDs
- **Defense-in-depth multi-tenancy**: RLS on all migration tables
- **Rollback support**: Reverse an import via external ID maps
- **Dry-run mode**: Validate without writing to production tables
- **Plugin extension points**: Source connectors, file parsers, field mappers, validators, etc.

## Pipeline Stages

1. **Job Creation** — Define what system you're migrating from and what objects
2. **Upload** — Source records ingested as JSON (via REST API or plugin connector)
3. **Map** — Field mappings define how source fields map to SIP canonical fields
4. **Validate** — Staged records validated, validation issues recorded
5. **Dry Run** — Simulate import without writing final records
6. **Import** — Execute import, creating SIP entities via existing services
7. **Report** — Full audit of what was created, skipped, failed

## Key Concepts

- **Source records**: Raw JSON data from the external system
- **Staged records**: Source records mapped into canonical import DTOs
- **Field mappings**: Source field → target field with optional transforms
- **External ID map**: Source external ID → SIP UUID, preserving traceability
- **Import results**: Per-record result (created, updated, skipped, failed)
- **Checkpoints**: Saved state for resume after failure

---

## Supported object types

`execute_import` used to handle `asset` only, and mapped every other object type
to a "no import handler" skip. A job for work orders therefore *reported success*
while creating nothing, with the records counted as skipped. The importer now
dispatches on the staged record's `target_entity_type` and has a real handler for
each of these:

| Object type | Handler creates | Notes |
|---|---|---|
| `location` | A location, resolving `parent_external_id` | Address and timezone land in location `metadata` (the table has no columns for them) |
| `asset_type` | An asset type with its `json_schema` | `category` defaults to `general` |
| `manufacturer` | A manufacturer | `support_email`/`support_phone` map onto the single `support_url` column |
| `asset_model` | An asset model | Requires both manufacturer and asset type references; fails loudly without them |
| `asset` | An asset, honouring type, model, location, parent, status, criticality and dates | Previously hardcoded: location `None`, model `None`, a *random* asset type id, status `Operational`, criticality `Medium` |
| `part` | A part | `unit_of_measure`→`unit`, `reorder_point`→`quantity_minimum`, `location`→`storage_location`. SIP's `parts` table has no category column, so `category` is folded into the description |
| `work_order` | A work order plus its `parts_used` rows | Requires a resolvable asset |

`customer`, `contact`, `organization` and `document` are defined in the canonical
contract but have no handler; they are recorded as skipped with an explicit
message rather than silently dropped.

## Dependency order

Cross-references resolve through the external-ID map, so referenced objects must
already exist:

```
location → asset_type → manufacturer → asset_model → asset → part → work_order
```

A record whose reference cannot be resolved **fails with a message naming the
missing external ID** instead of importing a partial entity. Assets are the one
exception: an asset with no resolvable `asset_type_external_id` falls back to a
generated type id and logs a warning, which preserves the behaviour of
single-type asset imports.

## Records that are already canonical

A job with **no field mappings** for an entity type treats each source record as
already canonical. This is what the canonical import contract promises — a plugin
that has already translated its source data, or a prepared fixture, hands over
canonical records directly.

Before this, an unmapped job staged `{}` for every record and then rejected each
one for a missing required field, which made the "upload canonical JSON" path
impossible and was the second reason only assets ever imported.

## Historical imports do not mutate live state

Imported work orders are records of work that already happened, so their part
usage is inserted as history rather than routed through
`PartUsageRepository::create_part_usage`. That method decrements
`parts.quantity_on_hand` and **rejects the write when stock is insufficient** —
back-filling a 2020 repair would otherwise drain today's inventory or fail
outright. Historical part usage therefore records what was consumed without
touching current stock.

## Rollback

Rollback archives what SIP can archive:

| Object type | Rollback behaviour |
|---|---|
| `asset` | Archived |
| `work_order` | Archived |
| everything else | **Not rolled back.** Locations, asset types, manufacturers, asset models and parts have no archive or delete operation in the domain or the schema. These are recorded as `rollback_skipped` with a reason, so a partial rollback is visible instead of being reported as complete. |

## Known limitation: no per-record skip in execute_import

`execute_import` processes a staged record only when its status is
`pending_validation`. `validate_job` marks a record `invalid` when it has a
blocking issue, and the importer counts those as skipped. A job can therefore
finish as `CompletedWithWarnings` while creating fewer entities than it staged —
check the report's `records_created` against the source count, which is what the
`/migrations/{job_id}/report` endpoint and the Test Bench plugin's summary are for.

