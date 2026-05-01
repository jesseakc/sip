# Migration Lifecycle

Every migration goes through these states:

```
  Draft ──▶ Uploaded ──▶ Mapped ──▶ Validated ──▶ ReadyForImport
    │          │           │          │               │
    │          │           │          │               ▼
    │          │           │          │          Importing ──▶ Completed
    │          │           │          │               │          │
    │          │           │          │               │          ▼
    │          │           │          │               │     CompletedWithWarnings
    │          │           │          │               │
    ▼          ▼           ▼          ▼               ▼
  Cancelled ◄─────────────────────────────┬────── Failed
                                          │
                                          ▼
                                       RolledBack
```

## State Descriptions

| State | Meaning | Next Actions |
|-------|---------|-------------|
| **Draft** | Job created, no source data yet. | Upload source records. |
| **Uploaded** | Source records ingested. | Map fields or skip to validation. |
| **Mapped** | Field mappings defined. | Validate. |
| **Validated** | Validation complete. May have issues. | Fix issues or proceed to dry run. |
| **ReadyForImport** | All validations pass, ready to import. | Dry run or execute. |
| **Importing** | Import in progress. | Wait for completion. |
| **Completed** | All records imported successfully. | View report. |
| **CompletedWithWarnings** | Import complete but some records had non-blocking errors. | Review skipped/failed records. |
| **Failed** | Import failed due to errors. | Fix issues and re-execute. |
| **Cancelled** | Job cancelled by user. | Cannot be resumed. |
| **RolledBack** | Import was rolled back. Created records have been archived. | Create a new job if needed. |

## Checkpoint Behavior

During import execution, checkpoints are written after each batch. If the server restarts during an import:
1. `MigrationRun` status is `Running` → import was interrupted
2. `MigrationCheckpoint` records show where to resume
3. Re-execute to continue from the last checkpoint

## Resumability

Imports skip records that have already been imported (tracked via `migration_external_id_maps`). You can re-run a failed import safely — already-imported records are skipped, and new/failed records are retried.

## Rollback

Rollback is conservative. It:
1. Finds all SIP entities created by this run (via `migration_external_id_maps`)
2. Archives them (sets `archived = true`, does not delete)
3. For assets: marks them as Retired
4. For work orders: marks them as Cancelled
5. Sets job status to RolledBack

Records that were edited after import are not rolled back (requires confirmation).
