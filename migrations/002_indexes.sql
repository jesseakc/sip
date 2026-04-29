-- SIP MVP Indexes

-- Organizations
CREATE INDEX idx_org_slug ON organizations (slug);

-- Locations
CREATE INDEX idx_loc_org ON locations (organization_id);
CREATE INDEX idx_loc_parent ON locations (parent_id);

-- Asset types
CREATE INDEX idx_atype_org ON asset_types (organization_id);
CREATE INDEX idx_atype_system ON asset_types (is_system) WHERE is_system = true;

-- Manufacturers
CREATE INDEX idx_mfg_org ON manufacturers (organization_id);

-- Asset models
CREATE INDEX idx_model_mfg ON asset_models (manufacturer_id);
CREATE INDEX idx_model_type ON asset_models (asset_type_id);
CREATE INDEX idx_model_org ON asset_models (organization_id);

-- Assets
CREATE INDEX idx_asset_org_status ON assets (organization_id, status);
CREATE INDEX idx_asset_org_loc ON assets (organization_id, location_id);
CREATE INDEX idx_asset_org_type ON assets (organization_id, asset_type_id);
CREATE INDEX idx_asset_tags ON assets USING GIN (tags);
CREATE INDEX idx_asset_attrs ON assets USING GIN (attributes jsonb_path_ops);
CREATE INDEX idx_asset_parent ON assets (parent_id);
CREATE INDEX idx_asset_serial ON assets (serial_number);

-- Users
CREATE INDEX idx_user_org ON users (organization_id);
CREATE INDEX idx_user_email ON users (organization_id, email);

-- Teams
CREATE INDEX idx_team_org ON teams (organization_id);
CREATE INDEX idx_team_lead ON teams (lead_id);

-- Work orders
CREATE INDEX idx_wo_org_asset_status ON work_orders (organization_id, asset_id, status);
CREATE INDEX idx_wo_org_status_scheduled ON work_orders (organization_id, status, scheduled_start)
    WHERE status IN ('OPEN','ASSIGNED','IN_PROGRESS');
CREATE INDEX idx_wo_display ON work_orders (organization_id, display_number);
CREATE INDEX idx_wo_org_created ON work_orders (organization_id, created_at DESC);

-- Work order status history
CREATE INDEX idx_wosh_wo ON work_order_status_history (work_order_id, created_at DESC);
CREATE INDEX idx_wosh_org_status ON work_order_status_history (organization_id, to_status, created_at DESC);

-- Work order assignments
CREATE INDEX idx_woa_org_wo ON work_order_assignments (organization_id, work_order_id);
CREATE INDEX idx_woa_assignee ON work_order_assignments (organization_id, assignee_type, assignee_id, status);
CREATE INDEX idx_woa_role ON work_order_assignments (organization_id, assignee_type, assignee_id, role, status);

-- Schedules
CREATE INDEX idx_sched_org_next ON schedules (organization_id, next_due)
    WHERE enabled = true;

-- Inspections
CREATE INDEX idx_insp_wo ON inspections (work_order_id);

-- Checklist items
CREATE INDEX idx_check_insp ON inspection_checklist_items (inspection_id);

-- Parts
CREATE INDEX idx_part_org ON parts (organization_id);

-- Part usage
CREATE INDEX idx_pu_wo ON part_usage (work_order_id);
CREATE INDEX idx_pu_part ON part_usage (part_id);

-- Asset parts (BOM)
CREATE INDEX idx_ap_type ON asset_parts (asset_type_id);

-- Documents
CREATE INDEX idx_doc_org_status ON documents (organization_id, processing_status);
CREATE INDEX idx_doc_org_name ON documents (organization_id, name);
CREATE INDEX idx_doc_checksum ON documents (checksum);

-- Document links
CREATE INDEX idx_dl_entity ON document_links (organization_id, entity_type, entity_id);
CREATE INDEX idx_dl_doc ON document_links (organization_id, document_id);

-- Document chunks
CREATE INDEX idx_dc_doc ON document_chunks (document_id, chunk_index);
CREATE INDEX idx_dc_org_doc ON document_chunks (organization_id, document_id);

-- Embedding records: HNSW vector index
CREATE INDEX idx_emb_vector ON embedding_records
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 200);
CREATE INDEX idx_emb_collection ON embedding_records (organization_id, collection, source_type);
CREATE INDEX idx_emb_source ON embedding_records (organization_id, source_type, source_id);

-- Activities
CREATE INDEX idx_act_org_entity ON activities (organization_id, entity_type, entity_id, created_at DESC);
CREATE INDEX idx_act_org_actor ON activities (organization_id, actor_type, actor_id, created_at DESC);
CREATE INDEX idx_act_request ON activities (request_id);

-- AI conversations
CREATE INDEX idx_aic_org_user ON ai_conversations (organization_id, user_id, created_at DESC);

-- AI messages
CREATE INDEX idx_aim_conv ON ai_messages (conversation_id, created_at DESC);

-- AI retrieval traces
CREATE INDEX idx_art_msg ON ai_retrieval_traces (message_id);

-- Agent identities
CREATE INDEX idx_agent_org ON agent_identities (organization_id);

-- Idempotency keys
CREATE INDEX idx_idk_org ON idempotency_keys (organization_id, key);

-- Outbox
CREATE INDEX idx_outbox_ready ON outbox (status, next_attempt_at, created_at)
    WHERE status IN ('PENDING', 'FAILED');

-- verification_traces
CREATE INDEX IF NOT EXISTS idx_vt_msg ON verification_traces (organization_id, message_id);

-- ai_answer_feedback
CREATE INDEX IF NOT EXISTS idx_aaf_msg ON ai_answer_feedback (organization_id, message_id);

-- FK indexes for query support
CREATE INDEX IF NOT EXISTS idx_assets_model ON assets (model_id);
CREATE INDEX IF NOT EXISTS idx_assets_firmware ON assets (firmware_version);
CREATE INDEX IF NOT EXISTS idx_assets_software ON assets (software_version);
CREATE INDEX IF NOT EXISTS idx_assets_hardware ON assets (hardware_revision);
CREATE INDEX IF NOT EXISTS idx_wo_parent ON work_orders (parent_id);
CREATE INDEX IF NOT EXISTS idx_wo_schedule ON work_orders (schedule_id);
CREATE INDEX IF NOT EXISTS idx_wo_created_by ON work_orders (created_by_id);
CREATE INDEX IF NOT EXISTS idx_wo_due ON work_orders (due_at) WHERE due_at IS NOT NULL;
