-- SIPmem Tables: Enhanced AI retrieval traces, verification traces, and answer feedback
-- Adds missing columns and new tables for SIPmem MVP

-- 1. Add missing columns to ai_retrieval_traces
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS retrieval_plan_id UUID;
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS retrieval_step_id UUID;
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS verification_trace_id UUID;
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS source_scope VARCHAR(50) DEFAULT 'PRIVATE_TENANT';
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS rank INTEGER;
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS verified_by_sql BOOLEAN DEFAULT FALSE;

-- 2. Add columns for simplified query/strategy/duration fields used by current app code
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS query TEXT;
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS strategy VARCHAR(50);
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS records_queried INTEGER DEFAULT 0;
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS records_returned INTEGER DEFAULT 0;
ALTER TABLE ai_retrieval_traces ADD COLUMN IF NOT EXISTS duration_ms INTEGER DEFAULT 0;

-- 3. Add missing columns to embedding_records
ALTER TABLE embedding_records ADD COLUMN IF NOT EXISTS document_chunk_id UUID REFERENCES document_chunks(id);
ALTER TABLE embedding_records ADD COLUMN IF NOT EXISTS model_name VARCHAR(100);
ALTER TABLE embedding_records ADD COLUMN IF NOT EXISTS dimensions INTEGER;
ALTER TABLE embedding_records ADD COLUMN IF NOT EXISTS source_scope VARCHAR(50) DEFAULT 'PRIVATE_TENANT';

-- 4. Create verification_traces table
CREATE TABLE IF NOT EXISTS verification_traces (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    conversation_id UUID REFERENCES ai_conversations(id),
    message_id UUID REFERENCES ai_messages(id),
    verification_status VARCHAR(50) NOT NULL DEFAULT 'NOT_CHECKED',
    checked_claim_count INTEGER DEFAULT 0,
    verified_claim_count INTEGER DEFAULT 0,
    unsupported_claim_count INTEGER DEFAULT 0,
    contradicted_claim_count INTEGER DEFAULT 0,
    verifier_type VARCHAR(50) DEFAULT 'RULE_BASED',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 5. Create ai_answer_feedback table
CREATE TABLE IF NOT EXISTS ai_answer_feedback (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    conversation_id UUID REFERENCES ai_conversations(id),
    message_id UUID NOT NULL REFERENCES ai_messages(id),
    user_id UUID NOT NULL REFERENCES users(id),
    rating VARCHAR(20) NOT NULL,
    comment TEXT,
    corrected_answer TEXT,
    corrected_sources JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 6. Enable RLS on new and existing tables
ALTER TABLE ai_retrieval_traces ENABLE ROW LEVEL SECURITY;
ALTER TABLE verification_traces ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai_answer_feedback ENABLE ROW LEVEL SECURITY;

-- 7. Create RLS policies (skip if exist)
DROP POLICY IF EXISTS art_tenant ON ai_retrieval_traces;
DROP POLICY IF EXISTS ait_org_isolation ON ai_retrieval_traces;
CREATE POLICY ait_org_isolation ON ai_retrieval_traces FOR ALL USING (organization_id = current_org_id());

DROP POLICY IF EXISTS vt_org_isolation ON verification_traces;
CREATE POLICY vt_org_isolation ON verification_traces FOR ALL USING (organization_id = current_org_id());

DROP POLICY IF EXISTS aaf_org_isolation ON ai_answer_feedback;
CREATE POLICY aaf_org_isolation ON ai_answer_feedback FOR ALL USING (organization_id = current_org_id());

-- 8. Add new enum values to ai_retrieval_type
DO $$
BEGIN
    BEGIN ALTER TYPE ai_retrieval_type ADD VALUE 'SQL'; EXCEPTION WHEN duplicate_object THEN NULL; END;
    BEGIN ALTER TYPE ai_retrieval_type ADD VALUE 'RAG'; EXCEPTION WHEN duplicate_object THEN NULL; END;
    BEGIN ALTER TYPE ai_retrieval_type ADD VALUE 'GRAPH'; EXCEPTION WHEN duplicate_object THEN NULL; END;
    BEGIN ALTER TYPE ai_retrieval_type ADD VALUE 'TEMPORAL'; EXCEPTION WHEN duplicate_object THEN NULL; END;
END $$;

-- Indexes for verification_traces and ai_answer_feedback
CREATE INDEX IF NOT EXISTS idx_vt_msg ON verification_traces (organization_id, message_id);
CREATE INDEX IF NOT EXISTS idx_aaf_msg ON ai_answer_feedback (organization_id, message_id);
