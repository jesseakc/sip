-- SIP MVP Schema Fixes

-- 1. Fix documents.source_type using wrong enum (work_order_source_type does not include 'UPLOAD')
CREATE TYPE document_source_type AS ENUM ('UPLOAD', 'GENERATED', 'IMPORTED', 'SYNCED');

ALTER TABLE documents ALTER COLUMN source_type DROP DEFAULT;
ALTER TABLE documents ALTER COLUMN source_type TYPE document_source_type USING
  CASE source_type::text
    WHEN 'UPLOAD' THEN 'UPLOAD'::document_source_type
    WHEN 'MANUAL' THEN 'UPLOAD'::document_source_type
    WHEN 'SCHEDULE' THEN 'GENERATED'::document_source_type
    WHEN 'AI_AGENT' THEN 'GENERATED'::document_source_type
    WHEN 'API' THEN 'SYNCED'::document_source_type
    WHEN 'IMPORT' THEN 'IMPORTED'::document_source_type
    WHEN 'PLUGIN' THEN 'SYNCED'::document_source_type
    WHEN 'SYSTEM' THEN 'GENERATED'::document_source_type
    ELSE 'UPLOAD'::document_source_type
  END;
ALTER TABLE documents ALTER COLUMN source_type SET DEFAULT 'UPLOAD'::document_source_type;

-- 2. Add organization_id to outbox for proper RLS isolation
ALTER TABLE outbox ADD COLUMN organization_id UUID REFERENCES organizations(id);
-- No existing seed data in outbox, so no UPDATE needed.
ALTER TABLE outbox ALTER COLUMN organization_id SET NOT NULL;

-- 3. Fix outbox RLS policy (aggregate_id is the entity id, not the organization id)
DROP POLICY IF EXISTS outbox_tenant ON outbox;
CREATE POLICY outbox_tenant ON outbox FOR ALL USING (organization_id = current_org_id());

-- 4. Add version column to assets for optimistic locking
ALTER TABLE assets ADD COLUMN version INTEGER NOT NULL DEFAULT 1;
