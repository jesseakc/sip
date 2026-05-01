# External ID Mapping

## Purpose

When importing data from external CMMS/CMMS platforms, each source record has an identity in its origin system (e.g., a Maximo asset ID `MX-1001`). SIP assigns its own UUIDs to all entities. The `migration_external_id_maps` table preserves the mapping between source external IDs and SIP entity IDs.

## Why It Matters

- **Traceability**: Back-reference any SIP entity to its source system record
- **Incremental imports**: Re-import a source record and update the existing SIP entity instead of creating a duplicate
- **Rollback**: Identify all SIP entities created by a specific migration run
- **Audit compliance**: Know exactly which source record produced each SIP entity

## Schema

```sql
CREATE TABLE migration_external_id_maps (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL,
    job_id UUID NOT NULL,
    run_id UUID NOT NULL,
    source_system TEXT NOT NULL,
    source_object_type TEXT NOT NULL,
    source_external_id TEXT NOT NULL,
    sip_entity_type TEXT NOT NULL,
    sip_entity_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    UNIQUE (source_system, source_object_type, source_external_id, organization_id)
);
```

## Unique Constraint

The combination `(source_system, source_object_type, source_external_id, organization_id)` is unique. This means:
- A single source external ID maps to exactly one SIP entity within an organization
- Re-importing the same source record will update the existing map (upsert)

## How It's Used

### During Import

For each successfully imported staged record:
1. SIP creates the entity through the normal service layer
2. The resulting SIP entity ID is recorded in `migration_external_id_maps`
3. The mapping includes the source system, object type, and external ID

### During Rollback

To rollback a migration:
1. Query `migration_external_id_maps` for all entries from the migration job
2. For each entry, archive/delete the corresponding SIP entity
3. The external ID maps remain as an audit record

### During Duplicate Detection

Before importing, check `migration_external_id_maps` for existing mappings:
1. If the source external ID already exists → the record was already imported
2. The importer can choose to update the existing entity or skip it

## API

```http
GET /api/v1/migrations/{job_id}/external-id-maps
```

Returns all external ID mappings for a migration job.
