# Security & Tenant Isolation

## API-Level Security

All migration endpoints go through the same JWT-based auth as the rest of SIP. The `auth_middleware` extracts the tenant context (`organization_id`, `user_id`, permissions) from the bearer token.

### Permission Enforcement

| Action | Required Permission | Default Roles |
|--------|--------------------|---------------|
| List jobs | `migration:read` | Admin, Manager |
| Create job | `migration:create` | Admin |
| Upload source records | `migration:create` | Admin |
| Map fields | `migration:map` | Admin, Manager |
| Run validation | `migration:validate` | Admin, Manager |
| Dry run | `migration:dry_run` | Admin, Manager |
| Execute import | `migration:execute` | Admin |
| Rollback | `migration:rollback` | Admin |
| Cancel | `migration:delete` | Admin |
| View raw source | `migration:view_raw_source` | Admin |

### Tenant Isolation

Every migration table has `organization_id` and Row-Level Security:

```sql
ALTER TABLE migration_jobs ENABLE ROW LEVEL SECURITY;
CREATE POLICY migration_jobs_org_isolation ON migration_jobs
  USING (organization_id = current_org_id());
```

The `PgMigrationRepository` enforces this at the query level too:
```rust
sqlx::query("... WHERE organization_id = $1 ...")
    .bind(Uuid::from(ctx.organization_id))
```

### Defense in Depth

- **Query Level**: Every SQL query includes `WHERE organization_id = $1`
- **RLS Level**: PostgreSQL enforces tenant isolation at the row level
- **Service Level**: `ctx.require_permission()` checks before any action
- **API Level**: JWT middleware extracts tenant context from every request

## File Upload Safety

- File size limits are enforced at the API level
- CSV files are parsed in the browser, not stored on disk
- Raw source data is stored as JSONB in `migration_source_records` (controlled by `migration:view_raw_source` permission)
- Sensitive fields can be redacted before storage

## External ID Mapping

The `migration_external_id_maps` table preserves the link between source system IDs and SIP entity IDs. This is critical for:
- Traceability
- Idempotent re-imports
- Rollback
- Audit compliance

## Audit Trail

Every migration action creates an `Activity` record:
- `migration.job.created`
- `migration.source.uploaded`
- `migration.mapping.updated`
- `migration.validation.completed`
- `migration.import.started`
- `migration.import.completed`
- `migration.import.failed`
- `migration.rollback.completed`

The actor type records whether the action was performed by a `human` user or a `plugin`.
