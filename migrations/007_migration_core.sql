-- SIP Migration Core Framework
-- Tables for the migration pipeline: source → staged → canonical import

CREATE TYPE migration_job_status AS ENUM (
    'DRAFT', 'UPLOADED', 'MAPPED', 'VALIDATED', 'READY_FOR_IMPORT',
    'IMPORTING', 'COMPLETED', 'COMPLETED_WITH_WARNINGS', 'FAILED',
    'CANCELLED', 'ROLLED_BACK'
);

CREATE TYPE migration_run_type AS ENUM ('DRY_RUN', 'IMPORT', 'ROLLBACK');

CREATE TYPE migration_run_status AS ENUM ('RUNNING', 'COMPLETED', 'FAILED', 'CANCELLED');

-- Migration jobs track a single import operation
CREATE TABLE migration_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    source_system TEXT NOT NULL,
    source_object_type TEXT NOT NULL,
    status migration_job_status NOT NULL DEFAULT 'DRAFT',
    source_record_count INTEGER NOT NULL DEFAULT 0,
    valid_record_count INTEGER NOT NULL DEFAULT 0,
    imported_record_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    created_by UUID NOT NULL REFERENCES users(id),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Migration runs track a single execution (dry run, import, or rollback)
CREATE TABLE migration_runs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    run_type migration_run_type NOT NULL,
    status migration_run_status NOT NULL DEFAULT 'RUNNING',
    records_processed INTEGER NOT NULL DEFAULT 0,
    records_created INTEGER NOT NULL DEFAULT 0,
    records_updated INTEGER NOT NULL DEFAULT 0,
    records_skipped INTEGER NOT NULL DEFAULT 0,
    records_failed INTEGER NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Raw source records ingested from the external system
CREATE TABLE migration_source_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    batch_id UUID,
    external_id TEXT,
    source_object_type TEXT NOT NULL,
    raw_data JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    row_number INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Staged records — source data normalized into canonical import DTOs
CREATE TABLE migration_staged_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    source_record_id UUID NOT NULL REFERENCES migration_source_records(id) ON DELETE CASCADE,
    target_entity_type TEXT NOT NULL,
    canonical_data JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending_validation',
    validation_errors JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Field mappings: source_field → target_field with optional transform
CREATE TABLE migration_field_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    target_entity_type TEXT NOT NULL,
    source_field TEXT NOT NULL,
    target_field TEXT NOT NULL,
    transform_expression TEXT,
    default_value TEXT,
    is_required BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (job_id, target_entity_type, source_field)
);

-- Import results: one row per staged record imported
CREATE TABLE migration_import_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    run_id UUID NOT NULL REFERENCES migration_runs(id) ON DELETE CASCADE,
    staged_record_id UUID NOT NULL REFERENCES migration_staged_records(id) ON DELETE CASCADE,
    sip_entity_type TEXT NOT NULL,
    sip_entity_id UUID,
    action TEXT NOT NULL,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- External ID map: source external ID → SIP UUID (for traceability)
CREATE TABLE migration_external_id_maps (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    run_id UUID NOT NULL REFERENCES migration_runs(id) ON DELETE CASCADE,
    source_system TEXT NOT NULL,
    source_object_type TEXT NOT NULL,
    source_external_id TEXT NOT NULL,
    sip_entity_type TEXT NOT NULL,
    sip_entity_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (source_system, source_object_type, source_external_id, organization_id)
);

-- Batches: groups of source records ingested together
CREATE TABLE migration_batches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    batch_number INTEGER NOT NULL,
    record_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (job_id, batch_number)
);

-- Validation issues found during the validate phase
CREATE TABLE migration_validation_issues (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    staged_record_id UUID NOT NULL REFERENCES migration_staged_records(id) ON DELETE CASCADE,
    severity TEXT NOT NULL DEFAULT 'error',
    field TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Duplicate candidates detected during import
CREATE TABLE migration_duplicate_candidates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    staged_record_id UUID NOT NULL REFERENCES migration_staged_records(id) ON DELETE CASCADE,
    sip_entity_type TEXT NOT NULL,
    sip_entity_id UUID NOT NULL,
    confidence_score DOUBLE PRECISION NOT NULL DEFAULT 0,
    match_reason TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Checkpoints: save migration state for resume/rollback
CREATE TABLE migration_checkpoints (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    job_id UUID NOT NULL REFERENCES migration_jobs(id) ON DELETE CASCADE,
    run_id UUID NOT NULL REFERENCES migration_runs(id) ON DELETE CASCADE,
    checkpoint_type TEXT NOT NULL,
    state_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Indexes ──

CREATE INDEX idx_mjob_org ON migration_jobs (organization_id);
CREATE INDEX idx_mjob_org_status ON migration_jobs (organization_id, status);

CREATE INDEX idx_mrun_job ON migration_runs (job_id);
CREATE INDEX idx_mrun_org ON migration_runs (organization_id);

CREATE INDEX idx_msr_job ON migration_source_records (job_id);
CREATE INDEX idx_msr_job_status ON migration_source_records (job_id, status);
CREATE INDEX idx_msr_ext_id ON migration_source_records (job_id, external_id) WHERE external_id IS NOT NULL;

CREATE INDEX idx_mstaged_job ON migration_staged_records (job_id);
CREATE INDEX idx_mstaged_job_status ON migration_staged_records (job_id, status);
CREATE INDEX idx_mstaged_src ON migration_staged_records (source_record_id);

CREATE INDEX idx_mfm_job ON migration_field_mappings (job_id);
CREATE INDEX idx_mfm_job_entity ON migration_field_mappings (job_id, target_entity_type);

CREATE INDEX idx_mir_run ON migration_import_results (run_id);
CREATE INDEX idx_mir_job ON migration_import_results (job_id);
CREATE INDEX idx_mir_staged ON migration_import_results (staged_record_id);

CREATE INDEX idx_meim_job ON migration_external_id_maps (job_id);
CREATE INDEX idx_meim_lookup ON migration_external_id_maps (organization_id, source_system, source_object_type, source_external_id);

CREATE INDEX idx_mbatch_job ON migration_batches (job_id);

CREATE INDEX idx_mvi_job ON migration_validation_issues (job_id);
CREATE INDEX idx_mvi_staged ON migration_validation_issues (staged_record_id);

CREATE INDEX idx_mdc_job ON migration_duplicate_candidates (job_id);
CREATE INDEX idx_mdc_staged ON migration_duplicate_candidates (staged_record_id);

CREATE INDEX idx_mcp_run ON migration_checkpoints (run_id);
CREATE INDEX idx_mcp_job ON migration_checkpoints (job_id);

-- FK index on batch_id (source_records → batches)
CREATE INDEX idx_msr_batch ON migration_source_records (batch_id) WHERE batch_id IS NOT NULL;

-- ── Row-Level Security ──

ALTER TABLE migration_jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_source_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_staged_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_field_mappings ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_import_results ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_external_id_maps ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_batches ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_validation_issues ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_duplicate_candidates ENABLE ROW LEVEL SECURITY;
ALTER TABLE migration_checkpoints ENABLE ROW LEVEL SECURITY;

CREATE POLICY mjob_tenant ON migration_jobs FOR ALL USING (organization_id = current_org_id());
CREATE POLICY mrun_tenant ON migration_runs FOR ALL USING (organization_id = current_org_id());
CREATE POLICY msr_tenant ON migration_source_records FOR ALL USING (organization_id = current_org_id());
CREATE POLICY mstaged_tenant ON migration_staged_records FOR ALL USING (organization_id = current_org_id());
CREATE POLICY mfm_tenant ON migration_field_mappings FOR ALL USING (organization_id = current_org_id());
CREATE POLICY mir_tenant ON migration_import_results FOR ALL USING (organization_id = current_org_id());
CREATE POLICY meim_tenant ON migration_external_id_maps FOR ALL USING (organization_id = current_org_id());
CREATE POLICY mbatch_tenant ON migration_batches FOR ALL USING (organization_id = current_org_id());
CREATE POLICY mvi_tenant ON migration_validation_issues FOR ALL USING (organization_id = current_org_id());
CREATE POLICY mdc_tenant ON migration_duplicate_candidates FOR ALL USING (organization_id = current_org_id());
CREATE POLICY mcp_tenant ON migration_checkpoints FOR ALL USING (organization_id = current_org_id());
