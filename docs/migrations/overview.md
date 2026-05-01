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
