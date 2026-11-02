-- Convert enum-backed business columns without dropping their data or defaults.
-- Drop partial indexes whose predicates reference the old enum types first.
DROP INDEX IF EXISTS idx_wo_org_status_scheduled;
DROP INDEX IF EXISTS idx_outbox_ready;

DO $$
DECLARE
    col record;
    default_expression text;
    default_value text;
BEGIN
    FOR col IN
        SELECT c.relname AS table_name, a.attname AS column_name, a.attrelid, a.attnum
        FROM pg_attribute a
        JOIN pg_class c ON c.oid = a.attrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        JOIN pg_type t ON t.oid = a.atttypid
        WHERE n.nspname = 'public' AND c.relkind = 'r'
          AND NOT a.attisdropped AND a.attnum > 0
          AND t.typtype = 'e' AND t.typname IN ('work_order_status', 'work_order_type', 'work_order_priority', 'work_order_source_type', 'asset_status', 'asset_criticality', 'user_role', 'location_type', 'schedule_trigger_type', 'document_type', 'document_source_type', 'document_visibility', 'document_processing_status', 'document_link_entity_type', 'document_link_relationship', 'assignee_type', 'assignment_role', 'assignment_status', 'activity_actor_type', 'activity_source', 'agent_type', 'ai_message_role', 'ai_retrieval_type', 'checklist_response_type', 'checklist_result', 'embedding_source_type', 'outbox_status', 'model_lifecycle_status', 'migration_job_status', 'migration_run_type', 'migration_run_status')
        ORDER BY c.relname, a.attnum
    LOOP
        default_expression := NULL;
        default_value := NULL;
        SELECT pg_get_expr(d.adbin, d.adrelid) INTO default_expression
        FROM pg_attrdef d WHERE d.adrelid = col.attrelid AND d.adnum = col.attnum;
        IF default_expression IS NOT NULL THEN
            -- All enum defaults in this schema are enum literals. Preserve their
            -- actual values as text, not casts that still depend on the old type.
            IF default_expression !~ '^''[^'']*''::[a-z_]+$' THEN
                RAISE EXCEPTION 'nonliteral enum default on %.% requires review', col.table_name, col.column_name;
            END IF;
            EXECUTE 'SELECT (' || default_expression || ')::text' INTO default_value;
            EXECUTE format('ALTER TABLE public.%I ALTER COLUMN %I DROP DEFAULT', col.table_name, col.column_name);
        END IF;
        EXECUTE format('ALTER TABLE public.%I ALTER COLUMN %I TYPE text USING %I::text', col.table_name, col.column_name, col.column_name);
        IF default_expression IS NOT NULL THEN
            EXECUTE format('ALTER TABLE public.%I ALTER COLUMN %I SET DEFAULT %L', col.table_name, col.column_name, default_value);
        END IF;
    END LOOP;
END $$;

-- RESTRICT turns a missed dependency into an error instead of deleting columns.
DROP TYPE IF EXISTS work_order_status RESTRICT;
DROP TYPE IF EXISTS work_order_type RESTRICT;
DROP TYPE IF EXISTS work_order_priority RESTRICT;
DROP TYPE IF EXISTS work_order_source_type RESTRICT;
DROP TYPE IF EXISTS asset_status RESTRICT;
DROP TYPE IF EXISTS asset_criticality RESTRICT;
DROP TYPE IF EXISTS user_role RESTRICT;
DROP TYPE IF EXISTS location_type RESTRICT;
DROP TYPE IF EXISTS schedule_trigger_type RESTRICT;
DROP TYPE IF EXISTS document_type RESTRICT;
DROP TYPE IF EXISTS document_source_type RESTRICT;
DROP TYPE IF EXISTS document_visibility RESTRICT;
DROP TYPE IF EXISTS document_processing_status RESTRICT;
DROP TYPE IF EXISTS document_link_entity_type RESTRICT;
DROP TYPE IF EXISTS document_link_relationship RESTRICT;
DROP TYPE IF EXISTS assignee_type RESTRICT;
DROP TYPE IF EXISTS assignment_role RESTRICT;
DROP TYPE IF EXISTS assignment_status RESTRICT;
DROP TYPE IF EXISTS activity_actor_type RESTRICT;
DROP TYPE IF EXISTS activity_source RESTRICT;
DROP TYPE IF EXISTS agent_type RESTRICT;
DROP TYPE IF EXISTS ai_message_role RESTRICT;
DROP TYPE IF EXISTS ai_retrieval_type RESTRICT;
DROP TYPE IF EXISTS checklist_response_type RESTRICT;
DROP TYPE IF EXISTS checklist_result RESTRICT;
DROP TYPE IF EXISTS embedding_source_type RESTRICT;
DROP TYPE IF EXISTS outbox_status RESTRICT;
DROP TYPE IF EXISTS model_lifecycle_status RESTRICT;
DROP TYPE IF EXISTS migration_job_status RESTRICT;
DROP TYPE IF EXISTS migration_run_type RESTRICT;
DROP TYPE IF EXISTS migration_run_status RESTRICT;
