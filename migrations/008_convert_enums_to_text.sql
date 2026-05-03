-- Convert all custom ENUM columns to TEXT for sqlx compatibility
-- Must drop dependent indexes/constraints before altering column types

-- Drop partial indexes that reference enum types in WHERE clauses
DROP INDEX IF EXISTS idx_wo_org_status_scheduled;
DROP INDEX IF EXISTS idx_outbox_ready;
DROP INDEX IF EXISTS idx_wo_org_status_scheduled;

-- work_orders
ALTER TABLE work_orders ALTER COLUMN status DROP DEFAULT;
ALTER TABLE work_orders ALTER COLUMN status TYPE text USING status::text;
ALTER TABLE work_orders ALTER COLUMN status SET DEFAULT 'DRAFT';

ALTER TABLE work_orders ALTER COLUMN type DROP DEFAULT;
ALTER TABLE work_orders ALTER COLUMN type TYPE text USING type::text;
ALTER TABLE work_orders ALTER COLUMN type SET DEFAULT 'CORRECTIVE';

ALTER TABLE work_orders ALTER COLUMN priority DROP DEFAULT;
ALTER TABLE work_orders ALTER COLUMN priority TYPE text USING priority::text;
ALTER TABLE work_orders ALTER COLUMN priority SET DEFAULT 'MEDIUM';

ALTER TABLE work_orders ALTER COLUMN source_type DROP DEFAULT;
ALTER TABLE work_orders ALTER COLUMN source_type TYPE text USING source_type::text;
ALTER TABLE work_orders ALTER COLUMN source_type SET DEFAULT 'MANUAL';

-- work_order_status_history
ALTER TABLE work_order_status_history ALTER COLUMN from_status TYPE text USING from_status::text;
ALTER TABLE work_order_status_history ALTER COLUMN to_status TYPE text USING to_status::text;
ALTER TABLE work_order_status_history ALTER COLUMN actor_type TYPE text USING actor_type::text;

-- work_order_assignments
ALTER TABLE work_order_assignments ALTER COLUMN assignee_type TYPE text USING assignee_type::text;
ALTER TABLE work_order_assignments ALTER COLUMN role TYPE text USING role::text;
ALTER TABLE work_order_assignments ALTER COLUMN status DROP DEFAULT;
ALTER TABLE work_order_assignments ALTER COLUMN status TYPE text USING status::text;
ALTER TABLE work_order_assignments ALTER COLUMN status SET DEFAULT 'ASSIGNED';

-- assets
ALTER TABLE assets ALTER COLUMN status DROP DEFAULT;
ALTER TABLE assets ALTER COLUMN status TYPE text USING status::text;
ALTER TABLE assets ALTER COLUMN status SET DEFAULT 'OPERATIONAL';

ALTER TABLE assets ALTER COLUMN criticality DROP DEFAULT;
ALTER TABLE assets ALTER COLUMN criticality TYPE text USING criticality::text;
ALTER TABLE assets ALTER COLUMN criticality SET DEFAULT 'MEDIUM';

-- users
ALTER TABLE users ALTER COLUMN role DROP DEFAULT;
ALTER TABLE users ALTER COLUMN role TYPE text USING role::text;
ALTER TABLE users ALTER COLUMN role SET DEFAULT 'VIEWER';

-- locations
ALTER TABLE locations ALTER COLUMN type TYPE text USING type::text;

-- schedules
ALTER TABLE schedules ALTER COLUMN trigger_type DROP DEFAULT;
ALTER TABLE schedules ALTER COLUMN trigger_type TYPE text USING trigger_type::text;
ALTER TABLE schedules ALTER COLUMN trigger_type SET DEFAULT 'CRON';

-- documents
ALTER TABLE documents ALTER COLUMN type DROP DEFAULT;
ALTER TABLE documents ALTER COLUMN type TYPE text USING type::text;
ALTER TABLE documents ALTER COLUMN type SET DEFAULT 'OTHER';

ALTER TABLE documents ALTER COLUMN source_type DROP DEFAULT;
ALTER TABLE documents ALTER COLUMN source_type TYPE text USING source_type::text;
ALTER TABLE documents ALTER COLUMN source_type SET DEFAULT 'UPLOAD';

ALTER TABLE documents ALTER COLUMN visibility DROP DEFAULT;
ALTER TABLE documents ALTER COLUMN visibility TYPE text USING visibility::text;
ALTER TABLE documents ALTER COLUMN visibility SET DEFAULT 'PRIVATE_TENANT';

ALTER TABLE documents ALTER COLUMN processing_status DROP DEFAULT;
ALTER TABLE documents ALTER COLUMN processing_status TYPE text USING processing_status::text;
ALTER TABLE documents ALTER COLUMN processing_status SET DEFAULT 'PENDING';

-- document_links
ALTER TABLE document_links ALTER COLUMN entity_type TYPE text USING entity_type::text;
ALTER TABLE document_links ALTER COLUMN relationship_type TYPE text USING relationship_type::text;

-- embedding_records
ALTER TABLE embedding_records ALTER COLUMN source_type TYPE text USING source_type::text;

-- outbox_events
ALTER TABLE outbox ALTER COLUMN status DROP DEFAULT;
ALTER TABLE outbox ALTER COLUMN status TYPE text USING status::text;
ALTER TABLE outbox ALTER COLUMN status SET DEFAULT 'PENDING';

-- asset_models
ALTER TABLE asset_models ALTER COLUMN lifecycle_status DROP DEFAULT;
ALTER TABLE asset_models ALTER COLUMN lifecycle_status TYPE text USING lifecycle_status::text;
ALTER TABLE asset_models ALTER COLUMN lifecycle_status SET DEFAULT 'ACTIVE';

-- agent_identities
ALTER TABLE agent_identities ALTER COLUMN type TYPE text USING type::text;

-- ai_retrieval_traces
ALTER TABLE ai_retrieval_traces ALTER COLUMN retriever_type TYPE text USING retriever_type::text;

-- activities
ALTER TABLE activities ALTER COLUMN source TYPE text USING source::text;

-- migration_jobs
ALTER TABLE migration_jobs ALTER COLUMN status DROP DEFAULT;
ALTER TABLE migration_jobs ALTER COLUMN status TYPE text USING status::text;
ALTER TABLE migration_jobs ALTER COLUMN status SET DEFAULT 'DRAFT';

-- migration_runs
ALTER TABLE migration_runs ALTER COLUMN run_type TYPE text USING run_type::text;
ALTER TABLE migration_runs ALTER COLUMN status DROP DEFAULT;
ALTER TABLE migration_runs ALTER COLUMN status TYPE text USING status::text;
ALTER TABLE migration_runs ALTER COLUMN status SET DEFAULT 'RUNNING';

-- Now drop all enum types (CASCADE handles remaining dependencies)
DROP TYPE IF EXISTS work_order_status CASCADE;
DROP TYPE IF EXISTS work_order_type CASCADE;
DROP TYPE IF EXISTS work_order_priority CASCADE;
DROP TYPE IF EXISTS work_order_source_type CASCADE;
DROP TYPE IF EXISTS asset_status CASCADE;
DROP TYPE IF EXISTS asset_criticality CASCADE;
DROP TYPE IF EXISTS user_role CASCADE;
DROP TYPE IF EXISTS location_type CASCADE;
DROP TYPE IF EXISTS schedule_trigger_type CASCADE;
DROP TYPE IF EXISTS document_type CASCADE;
DROP TYPE IF EXISTS document_source_type CASCADE;
DROP TYPE IF EXISTS document_visibility CASCADE;
DROP TYPE IF EXISTS document_processing_status CASCADE;
DROP TYPE IF EXISTS document_link_entity_type CASCADE;
DROP TYPE IF EXISTS document_link_relationship CASCADE;
DROP TYPE IF EXISTS assignee_type CASCADE;
DROP TYPE IF EXISTS assignment_role CASCADE;
DROP TYPE IF EXISTS assignment_status CASCADE;
DROP TYPE IF EXISTS activity_actor_type CASCADE;
DROP TYPE IF EXISTS activity_source CASCADE;
DROP TYPE IF EXISTS agent_type CASCADE;
DROP TYPE IF EXISTS ai_message_role CASCADE;
DROP TYPE IF EXISTS ai_retrieval_type CASCADE;
DROP TYPE IF EXISTS checklist_response_type CASCADE;
DROP TYPE IF EXISTS checklist_result CASCADE;
DROP TYPE IF EXISTS embedding_source_type CASCADE;
DROP TYPE IF EXISTS outbox_status CASCADE;
DROP TYPE IF EXISTS model_lifecycle_status CASCADE;
DROP TYPE IF EXISTS migration_job_status CASCADE;
DROP TYPE IF EXISTS migration_run_type CASCADE;
DROP TYPE IF EXISTS migration_run_status CASCADE;
