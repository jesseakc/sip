# Migration Studio Architecture

## Design Principles

1. **Core owns truth, plugin owns translation.** The SIP Migration Core Framework (`sip-domain`, `sip-application`, `sip-infrastructure`) owns the data model, validation rules, and import pipeline. The Migration Studio plugin owns the UI, file parsing, field mapping suggestions, and user experience.

2. **API-first.** The plugin's frontend communicates exclusively through SIP's public REST API. No direct database access. No bypassing of core validation.

3. **Declarative mapping.** Field mappings are stored as records in `migration_field_mappings`, making them reproducible, auditable, and reusable.

4. **Resumable imports.** Checkpoints and batch processing allow large imports to be paused and resumed.

## Component Architecture

```
┌──────────────────────────────────────────────────────┐
│              Migration Studio (Plugin)                │
│                                                      │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────┐  │
│  │ File Parser │  │ Field Mapper │  │ CSV/JSON   │  │
│  │ (CSV/JSON)  │  │ (UI wizard)  │  │ Upload UI  │  │
│  └──────┬──────┘  └──────┬───────┘  └─────┬──────┘  │
│         │                │                 │         │
│         └────────────────┼─────────────────┘         │
│                          │                           │
│              ┌───────────▼──────────┐               │
│              │   SIP REST API       │               │
│              │   /api/v1/migrations │               │
│              └───────────┬──────────┘               │
└──────────────────────────┼──────────────────────────┘
                           │
┌──────────────────────────┼──────────────────────────┐
│              SIP Core    │                          │
│                          │                          │
│  ┌───────────────────────▼────────────────────┐     │
│  │         MigrationService                   │     │
│  │  - create_job / list_jobs / get_job        │     │
│  │  - add_source_records                      │     │
│  │  - save_field_mappings                     │     │
│  │  - validate_job (maps→canonical, checks)   │     │
│  │  - dry_run (simulate, no mutation)         │     │
│  │  - execute_import (creates real records)   │     │
│  │  - rollback_job (archives created records) │     │
│  └───────────────────┬────────────────────────┘     │
│                      │                              │
│  ┌───────────────────▼────────────────────────┐     │
│  │     PgMigrationRepository                  │     │
│  │  - migration_jobs / _source_records        │     │
│  │  - migration_staged_records                │     │
│  │  - migration_field_mappings                │     │
│  │  - migration_import_results                │     │
│  │  - migration_external_id_maps              │     │
│  │  - migration_validation_issues             │     │
│  │  - migration_checkpoints                   │     │
│  └───────────────────┬────────────────────────┘     │
│                      │                              │
│              ┌───────▼────────┐                     │
│              │  PostgreSQL    │                     │
│              │  (migration_*) │                     │
│              └────────────────┘                     │
└─────────────────────────────────────────────────────┘
```

## Data Flow

### 1. Upload Phase
```
User → Upload CSV/JSON → Plugin parses locally → POST /migrations/jobs → POST /jobs/:id/source-records
```
The plugin parses the file in the browser (no server-side file storage needed). Records are sent as JSON arrays to the API.

### 2. Mapping Phase
```
User maps fields → POST /jobs/:id/field-mappings → MigrationService saves mappings → Job status → Mapped
```

### 3. Validation Phase
```
POST /jobs/:id/validate → MigrationService.validate_job() 
  → applies mappings to each source record 
  → creates staged records (canonical_data JSON)
  → runs validation rules (required fields, enum checks, duplicates)
  → creates validation_issues
  → Job status → Validated or ReadyForImport
```

### 4. Import Phase
```
POST /jobs/:id/execute → MigrationService.execute_import()
  → Creates MigrationRun (type=Import, status=Running)
  → For each staged record:
     → Look up external_id in migration_external_id_maps → skip if already imported
     → Create SIP entity via AssetService/DocumentService/etc.
     → On success: insert migration_external_id_map, migration_import_result
     → On failure: log error, continue to next record
  → Run status → Completed (or CompletedWithWarnings, or Failed)
```

### 5. External ID Preservation
Every imported record creates a row in `migration_external_id_maps`:
```
source_system = "Salesforce"
source_object_type = "case"
source_external_id = "SF-50011001"
sip_entity_type = "work_order"
sip_entity_id = <UUID>
```
This preserves bidirectional traceability forever and enables idempotent re-imports.

## Extension Points

The plugin consumes and provides these migration extension points:
- `migration.file_parser` — Parse CSV/JSON into source records
- `migration.field_mapper` — Suggest and save field mappings
- `migration.validator` — Validate staged records
- `migration.duplicate_resolver` — Handle duplicate candidates
- `migration.post_import_hook` — Post-import actions (notifications, enrichment)
- `migration.report_generator` — Generate formatted reports
