-- Run manually, after a backup and schema/data inspection. See docs/deployment/migration-recovery.md.
-- This refuses to acknowledge 008 until its dropped columns have been restored.
BEGIN;
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM _sqlx_migrations WHERE NOT success) THEN
        RAISE EXCEPTION 'failed migration history requires inspection before reconciliation';
    END IF;
    IF EXISTS (SELECT 1 FROM _sqlx_migrations WHERE version >= 8 AND success) THEN
        IF (SELECT count(*) FROM information_schema.columns
            WHERE table_schema = 'public' AND
              ((table_name='activities' AND column_name='actor_type') OR
               (table_name='ai_messages' AND column_name='role') OR
               (table_name='inspection_checklist_items' AND column_name IN ('response_type','result')))) <> 4 THEN
            RAISE EXCEPTION 'migration 008 dropped business columns: restore their definitions and values from backup before reconciling checksums';
        END IF;
    END IF;
    IF EXISTS (SELECT 1 FROM _sqlx_migrations WHERE version = 8 AND checksum NOT IN (decode('2e47be4401cfc5fa33cb6556d969ec04608fe3f09662dbc1477785b22368aad77087a741b043417a3e29c27b10effed7','hex'), decode('6ef4cc7b41a2f8147de9ad0c77440f76beb8894c3588b90f00042c7fe58818016465da1a62a821f632e477e2402e9a30','hex'))) THEN
        RAISE EXCEPTION 'migration 8 has an unknown checksum; stop and inspect this installation';
    END IF;
    UPDATE _sqlx_migrations SET checksum = decode('6ef4cc7b41a2f8147de9ad0c77440f76beb8894c3588b90f00042c7fe58818016465da1a62a821f632e477e2402e9a30','hex')
    WHERE version = 8 AND success AND checksum = decode('2e47be4401cfc5fa33cb6556d969ec04608fe3f09662dbc1477785b22368aad77087a741b043417a3e29c27b10effed7','hex');
    IF EXISTS (SELECT 1 FROM _sqlx_migrations WHERE version = 11 AND checksum NOT IN (decode('c02ef6470d6ed08730484cc7a441535f62684a5f1690286273213fb243097919eace312a2d2fbeec983efea7413a0dc9','hex'), decode('436f955cb9b983bf9e2d63db87f2ae3f6b2d8f089ee1915c94450755d95bade4a34a8d031c72788a193fda949cd74645','hex'))) THEN
        RAISE EXCEPTION 'migration 11 has an unknown checksum; stop and inspect this installation';
    END IF;
    UPDATE _sqlx_migrations SET checksum = decode('436f955cb9b983bf9e2d63db87f2ae3f6b2d8f089ee1915c94450755d95bade4a34a8d031c72788a193fda949cd74645','hex')
    WHERE version = 11 AND success AND checksum = decode('c02ef6470d6ed08730484cc7a441535f62684a5f1690286273213fb243097919eace312a2d2fbeec983efea7413a0dc9','hex');
END $$;
COMMIT;
