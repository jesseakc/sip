# Migration integrity recovery

The corrected 008 preserves activities.actor_type, ai_messages.role, and checklist response_type/result. The corrected 011 handles both RESTRICT and foreign-key errors and measures progress over a complete dependency pass. Both files differ from the initial public snapshot.

## Fresh installations

Run the normal SQLx migration sequence. No checksum reconciliation is necessary. Application demo users and tenant records are removed by 011; shared system catalogs remain.

## Existing installations

Back up the database, stop writers, and inspect `_sqlx_migrations` before upgrading. Keep the backup until application and migration checks pass.

- If 008 has not run, the corrected migration preserves its dependent columns and values.
- If the original 008 ran, it deleted four columns and their contents. Restore their definitions and authoritative values from a backup. Adding empty columns or invented default values does not recover the data. If no backup exists, stop and make an explicit data-recovery decision before upgrading. This PR does not fabricate lost values.
- If 011 has not run, inspect the fixed demo organization `00000000-0000-0000-0000-000000000001`. Migration 011 deletes its records. If you used that organization for real work, do not run cleanup against it until those records are preserved or migrated to a different organization. This PR retains the existing cleanup scope, it does not infer which new records are disposable.
- After the data and schema inspection, run `psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -f scripts/reconcile-reviewed-migrations.sql`. It accepts only the exact old or corrected checksums for 008 and 011, aborts on a failed migration record, and refuses to reconcile a post-008 schema with missing columns. It is an explicit operator action, never an application startup side effect.
- Run `sqlx migrate run` and the regression checks against the restored database. Existing missing column values must be audited separately; column presence alone does not prove data recovery.

The PostgreSQL regression tests use a disposable database only. Do not run fixture setup on a real database.
