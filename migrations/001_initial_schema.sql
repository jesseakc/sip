-- SIP MVP Initial Schema
-- PRD-v2 compliant: Core Entities, Tenant Scoping, pgvector, ltree

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "vector";
CREATE EXTENSION IF NOT EXISTS "ltree";
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Enums
CREATE TYPE asset_status AS ENUM ('OPERATIONAL', 'DEGRADED', 'DOWN', 'MAINTENANCE', 'RETIRED');
CREATE TYPE asset_criticality AS ENUM ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL');
CREATE TYPE location_type AS ENUM ('SITE', 'BUILDING', 'FLOOR', 'ROOM', 'AREA', 'OTHER');
CREATE TYPE work_order_status AS ENUM ('DRAFT', 'OPEN', 'ASSIGNED', 'ACCEPTED', 'IN_PROGRESS', 'ON_HOLD', 'COMPLETED', 'REVIEWED', 'CLOSED', 'CANCELLED');
CREATE TYPE work_order_type AS ENUM ('PREVENTIVE', 'CORRECTIVE', 'INSPECTION', 'EMERGENCY');
CREATE TYPE work_order_priority AS ENUM ('LOW', 'MEDIUM', 'HIGH', 'CRITICAL');
CREATE TYPE work_order_source_type AS ENUM ('MANUAL', 'SCHEDULE', 'AI_AGENT', 'API', 'IMPORT', 'PLUGIN', 'SYSTEM');
CREATE TYPE assignment_role AS ENUM ('PRIMARY', 'SECONDARY', 'OBSERVER', 'APPROVER', 'DISPATCHED_TECH', 'REMOTE_SUPPORT');
CREATE TYPE assignment_status AS ENUM ('ASSIGNED', 'ACCEPTED', 'DECLINED', 'REMOVED', 'COMPLETED');
CREATE TYPE assignee_type AS ENUM ('USER', 'TEAM', 'VENDOR', 'AI_AGENT');
CREATE TYPE schedule_trigger_type AS ENUM ('CRON', 'METER');
CREATE TYPE checklist_response_type AS ENUM ('PASS_FAIL', 'NUMERIC', 'TEXT', 'PHOTO');
CREATE TYPE checklist_result AS ENUM ('PASS', 'FAIL', 'N_A');
CREATE TYPE document_type AS ENUM ('MANUAL', 'PROCEDURE', 'DIAGRAM', 'WARRANTY', 'CERTIFICATE', 'PHOTO', 'OTHER');
CREATE TYPE document_visibility AS ENUM ('PRIVATE_TENANT', 'SHARED_VENDOR', 'PUBLIC', 'SYSTEM_DEFAULT');
CREATE TYPE document_processing_status AS ENUM ('PENDING', 'EXTRACTING', 'EXTRACTED', 'CHUNKING', 'EMBEDDING', 'INDEXED', 'FAILED');
CREATE TYPE document_link_relationship AS ENUM ('MANUAL_FOR', 'PHOTO_OF', 'WARRANTY_FOR', 'PROCEDURE_FOR', 'EVIDENCE_FOR', 'ATTACHMENT');
CREATE TYPE document_link_entity_type AS ENUM ('ASSET', 'WORK_ORDER', 'ASSET_MODEL', 'MANUFACTURER', 'PART', 'INSPECTION');
CREATE TYPE embedding_source_type AS ENUM ('ASSET', 'WORK_ORDER', 'DOCUMENT_CHUNK', 'INSPECTION_FINDING');
CREATE TYPE activity_actor_type AS ENUM ('HUMAN', 'AI_AGENT', 'SYSTEM', 'PLUGIN');
CREATE TYPE activity_source AS ENUM ('API', 'UI', 'AI', 'AUTOMATION', 'PLUGIN', 'SYSTEM');
CREATE TYPE ai_message_role AS ENUM ('USER', 'ASSISTANT', 'SYSTEM', 'TOOL');
CREATE TYPE ai_retrieval_type AS ENUM ('RELATIONAL', 'VECTOR', 'TOOL');
CREATE TYPE agent_type AS ENUM ('KNOWLEDGE', 'SCHEDULING', 'DIAGNOSTIC', 'DISPATCH', 'COMPLIANCE', 'INVENTORY', 'CUSTOM');
CREATE TYPE user_role AS ENUM ('ADMIN', 'MANAGER', 'TECHNICIAN', 'VIEWER', 'VENDOR', 'AUDITOR');
CREATE TYPE outbox_status AS ENUM ('PENDING', 'PROCESSING', 'PUBLISHED', 'FAILED', 'DEAD_LETTER');
CREATE TYPE model_lifecycle_status AS ENUM ('ACTIVE', 'DEPRECATED', 'END_OF_SUPPORT', 'RETIRED');

-- Core tenant tables
CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ
);

CREATE TABLE organization_settings (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    timezone TEXT NOT NULL DEFAULT 'UTC',
    default_currency TEXT NOT NULL DEFAULT 'USD',
    unit_system TEXT NOT NULL DEFAULT 'METRIC',
    ai_enabled BOOLEAN NOT NULL DEFAULT true,
    ai_provider_config JSONB,
    ai_prompt_retention_policy TEXT NOT NULL DEFAULT 'METADATA_ONLY',
    ai_message_retention_days INTEGER,
    ai_read_only_mode BOOLEAN NOT NULL DEFAULT true,
    ai_mutations_require_approval BOOLEAN NOT NULL DEFAULT true,
    ai_max_autonomy_level TEXT NOT NULL DEFAULT 'READ_ONLY',
    ai_allowed_tools TEXT[] NOT NULL DEFAULT '{}',
    ai_blocked_tools TEXT[] NOT NULL DEFAULT '{}',
    document_retention_days INTEGER,
    work_order_display_prefix TEXT NOT NULL DEFAULT 'WO',
    feature_flags JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE locations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES locations(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    type location_type NOT NULL DEFAULT 'OTHER',
    geo GEOGRAPHY(POINT, 4326),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE asset_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    description TEXT,
    schema JSONB NOT NULL DEFAULT '{}',
    default_pm_schedules JSONB[] NOT NULL DEFAULT '{}',
    default_inspection_template JSONB,
    icon TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE manufacturers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    website TEXT,
    support_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE asset_models (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    manufacturer_id UUID NOT NULL REFERENCES manufacturers(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    model_number TEXT NOT NULL,
    revision TEXT,
    lifecycle_status model_lifecycle_status NOT NULL DEFAULT 'ACTIVE',
    asset_type_id UUID NOT NULL REFERENCES asset_types(id) ON DELETE RESTRICT,
    documentation_url TEXT,
    default_attributes JSONB NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE assets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    location_id UUID REFERENCES locations(id) ON DELETE SET NULL,
    parent_id UUID REFERENCES assets(id) ON DELETE RESTRICT,
    asset_type_id UUID NOT NULL REFERENCES asset_types(id) ON DELETE RESTRICT,
    model_id UUID REFERENCES asset_models(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    description TEXT,
    serial_number TEXT,
    firmware_version TEXT,
    software_version TEXT,
    hardware_revision TEXT,
    status asset_status NOT NULL DEFAULT 'OPERATIONAL',
    criticality asset_criticality NOT NULL DEFAULT 'MEDIUM',
    installed_date DATE,
    warranty_expiry DATE,
    attributes JSONB NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ,
    archived_by_id UUID,
    archive_reason TEXT
);

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email TEXT NOT NULL,
    name TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'VIEWER',
    skills TEXT[] NOT NULL DEFAULT '{}',
    certifications JSONB[] NOT NULL DEFAULT '{}',
    working_hours JSONB,
    password_hash TEXT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, email)
);

CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    lead_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE team_members (
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (team_id, user_id)
);

CREATE TABLE work_orders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE RESTRICT,
    parent_id UUID REFERENCES work_orders(id) ON DELETE RESTRICT,
    schedule_id UUID,
    type work_order_type NOT NULL DEFAULT 'CORRECTIVE',
    priority work_order_priority NOT NULL DEFAULT 'MEDIUM',
    status work_order_status NOT NULL DEFAULT 'DRAFT',
    title TEXT NOT NULL,
    display_number TEXT NOT NULL,
    description TEXT NOT NULL,
    scheduled_start TIMESTAMPTZ,
    scheduled_end TIMESTAMPTZ,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    due_at TIMESTAMPTZ,
    estimated_hours NUMERIC(5,2),
    actual_hours NUMERIC(5,2),
    resolution_notes TEXT,
    failure_code TEXT,
    root_cause TEXT,
    created_by_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    source_type work_order_source_type NOT NULL DEFAULT 'MANUAL',
    source_system TEXT,
    external_id TEXT,
    external_url TEXT,
    reopened_count INTEGER NOT NULL DEFAULT 0,
    last_reopened_at TIMESTAMPTZ,
    last_reopened_by_id UUID REFERENCES users(id) ON DELETE SET NULL,
    version INTEGER NOT NULL DEFAULT 1,
    archived_at TIMESTAMPTZ,
    archived_by_id UUID REFERENCES users(id) ON DELETE SET NULL,
    archive_reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, display_number)
);

CREATE TABLE work_order_status_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    work_order_id UUID NOT NULL REFERENCES work_orders(id) ON DELETE CASCADE,
    from_status work_order_status,
    to_status work_order_status NOT NULL,
    changed_by_id UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_type activity_actor_type NOT NULL DEFAULT 'HUMAN',
    agent_identity_id UUID,
    plugin_id UUID,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE work_order_assignments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    work_order_id UUID NOT NULL REFERENCES work_orders(id) ON DELETE CASCADE,
    assignee_type assignee_type NOT NULL,
    assignee_id UUID NOT NULL,
    role assignment_role NOT NULL DEFAULT 'PRIMARY',
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    accepted_at TIMESTAMPTZ,
    removed_at TIMESTAMPTZ,
    status assignment_status NOT NULL DEFAULT 'ASSIGNED'
);

CREATE TABLE schedules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    trigger_type schedule_trigger_type NOT NULL DEFAULT 'CRON',
    trigger_config JSONB NOT NULL DEFAULT '{}',
    work_order_template JSONB NOT NULL DEFAULT '{}',
    next_due TIMESTAMPTZ,
    last_triggered TIMESTAMPTZ,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ,
    archived_by_id UUID REFERENCES users(id) ON DELETE SET NULL,
    archive_reason TEXT
);

-- Fix: schedule FK in work_orders added after schedules exist, or we add it in a separate pass.
-- We'll add the FK now that schedules exists (it's above in the script).
ALTER TABLE work_orders ADD CONSTRAINT fk_work_orders_schedule
    FOREIGN KEY (schedule_id) REFERENCES schedules(id) ON DELETE SET NULL;

CREATE TABLE inspections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_order_id UUID NOT NULL UNIQUE REFERENCES work_orders(id) ON DELETE CASCADE,
    template_name TEXT
);

CREATE TABLE inspection_checklist_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    inspection_id UUID NOT NULL REFERENCES inspections(id) ON DELETE CASCADE,
    ordinal INTEGER NOT NULL DEFAULT 0,
    question TEXT NOT NULL,
    response_type checklist_response_type NOT NULL DEFAULT 'PASS_FAIL',
    expected_value TEXT,
    actual_value TEXT,
    result checklist_result,
    finding TEXT,
    photo_url TEXT
);

CREATE TABLE parts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    part_number TEXT,
    description TEXT,
    quantity_on_hand NUMERIC(10,2) NOT NULL DEFAULT 0,
    quantity_minimum NUMERIC(10,2),
    unit TEXT NOT NULL,
    unit_cost NUMERIC(10,4),
    storage_location TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE part_usage (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    work_order_id UUID NOT NULL REFERENCES work_orders(id) ON DELETE CASCADE,
    part_id UUID NOT NULL REFERENCES parts(id) ON DELETE RESTRICT,
    quantity NUMERIC(10,2) NOT NULL CHECK (quantity > 0),
    used_by_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT
);

CREATE TABLE asset_parts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_type_id UUID NOT NULL REFERENCES asset_types(id) ON DELETE CASCADE,
    part_id UUID NOT NULL REFERENCES parts(id) ON DELETE CASCADE,
    default_quantity NUMERIC(10,2) NOT NULL DEFAULT 1,
    UNIQUE (asset_type_id, part_id)
);

CREATE TABLE documents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    type document_type NOT NULL DEFAULT 'OTHER',
    mime_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    document_version TEXT,
    checksum TEXT NOT NULL,
    source_type work_order_source_type NOT NULL DEFAULT 'MANUAL',
    source_system TEXT,
    external_id TEXT,
    external_url TEXT,
    storage_path TEXT NOT NULL,
    visibility document_visibility NOT NULL DEFAULT 'PRIVATE_TENANT',
    processing_status document_processing_status NOT NULL DEFAULT 'PENDING',
    processing_error TEXT,
    extracted_text_path TEXT,
    text_content TEXT,
    effective_date DATE,
    expiration_date DATE,
    supersedes_document_id UUID REFERENCES documents(id) ON DELETE SET NULL,
    version INTEGER NOT NULL DEFAULT 1,
    archived_at TIMESTAMPTZ,
    archived_by_id UUID REFERENCES users(id) ON DELETE SET NULL,
    archive_reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    uploaded_by_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE document_links (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    entity_type document_link_entity_type NOT NULL,
    entity_id UUID NOT NULL,
    relationship_type document_link_relationship NOT NULL DEFAULT 'ATTACHMENT',
    created_by_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE document_chunks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    token_count INTEGER,
    page_number INTEGER,
    section_title TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (document_id, chunk_index)
);

CREATE TABLE embedding_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    source_type embedding_source_type NOT NULL,
    source_id UUID NOT NULL,
    collection TEXT NOT NULL,
    content TEXT NOT NULL,
    embedding vector(1024) NOT NULL,
    embedding_model TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_type activity_actor_type NOT NULL DEFAULT 'HUMAN',
    agent_identity_id UUID,
    plugin_id UUID,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    action TEXT NOT NULL,
    changes JSONB,
    request_id TEXT,
    correlation_id TEXT,
    source activity_source NOT NULL DEFAULT 'API',
    reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE ai_conversations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_identity_id UUID,
    title TEXT,
    entity_type TEXT,
    entity_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    archived_at TIMESTAMPTZ
);

CREATE TABLE ai_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    conversation_id UUID NOT NULL REFERENCES ai_conversations(id) ON DELETE CASCADE,
    role ai_message_role NOT NULL,
    content TEXT,
    content_retained BOOLEAN NOT NULL DEFAULT false,
    retention_policy_at_creation TEXT NOT NULL DEFAULT 'METADATA_ONLY',
    structured_response JSONB,
    sources JSONB,
    token_input_count INTEGER,
    token_output_count INTEGER,
    model TEXT,
    provider TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE ai_retrieval_traces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    message_id UUID NOT NULL REFERENCES ai_messages(id) ON DELETE CASCADE,
    retriever_type ai_retrieval_type NOT NULL,
    source_type TEXT NOT NULL,
    source_id UUID NOT NULL,
    score NUMERIC(5,4),
    included_in_context BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE agent_identities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    type agent_type NOT NULL DEFAULT 'KNOWLEDGE',
    provider TEXT,
    model TEXT,
    permissions JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- FK: ai_conversations.agent_identity_id → agent_identities
ALTER TABLE ai_conversations ADD CONSTRAINT fk_aic_agent
    FOREIGN KEY (agent_identity_id) REFERENCES agent_identities(id);

CREATE TABLE idempotency_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    key TEXT NOT NULL,
    request_hash TEXT NOT NULL,
    response_status INTEGER NOT NULL,
    response_body JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE (organization_id, key)
);

CREATE TABLE outbox (
    id BIGSERIAL PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type VARCHAR(200) NOT NULL,
    payload JSONB NOT NULL,
    status outbox_status NOT NULL DEFAULT 'PENDING',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    next_attempt_at TIMESTAMPTZ,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
