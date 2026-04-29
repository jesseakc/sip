-- SIP MVP Row-Level Security Policies

-- Helper function to get current org ID from session variable
CREATE OR REPLACE FUNCTION current_org_id() RETURNS UUID AS $$
BEGIN
    RETURN NULLIF(current_setting('app.current_organization_id', true), '')::UUID;
EXCEPTION WHEN OTHERS THEN
    RETURN NULL;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Apply RLS to tenant-owned tables
ALTER TABLE locations ENABLE ROW LEVEL SECURITY;
ALTER TABLE assets ENABLE ROW LEVEL SECURITY;
ALTER TABLE asset_types ENABLE ROW LEVEL SECURITY;
ALTER TABLE manufacturers ENABLE ROW LEVEL SECURITY;
ALTER TABLE asset_models ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE teams ENABLE ROW LEVEL SECURITY;
ALTER TABLE team_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_order_status_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE work_order_assignments ENABLE ROW LEVEL SECURITY;
ALTER TABLE schedules ENABLE ROW LEVEL SECURITY;
ALTER TABLE inspections ENABLE ROW LEVEL SECURITY;
ALTER TABLE inspection_checklist_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE parts ENABLE ROW LEVEL SECURITY;
ALTER TABLE part_usage ENABLE ROW LEVEL SECURITY;
ALTER TABLE asset_parts ENABLE ROW LEVEL SECURITY;
ALTER TABLE documents ENABLE ROW LEVEL SECURITY;
ALTER TABLE document_links ENABLE ROW LEVEL SECURITY;
ALTER TABLE document_chunks ENABLE ROW LEVEL SECURITY;
ALTER TABLE embedding_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE activities ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai_conversations ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai_messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai_retrieval_traces ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_identities ENABLE ROW LEVEL SECURITY;
ALTER TABLE idempotency_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE organization_settings ENABLE ROW LEVEL SECURITY;

-- Note: organizations table is NOT RLS-protected in the same way;
-- access is governed by application logic (users belong to one org).

-- Helper policy: match org_id or allow system-level null org_id rows for shared reference data
CREATE POLICY loc_tenant ON locations FOR ALL USING (organization_id = current_org_id());
CREATE POLICY asset_tenant ON assets FOR ALL USING (organization_id = current_org_id());
CREATE POLICY atype_tenant ON asset_types FOR ALL USING (organization_id = current_org_id() OR is_system = true);
CREATE POLICY mfg_tenant ON manufacturers FOR ALL USING (organization_id = current_org_id() OR organization_id IS NULL);
CREATE POLICY model_tenant ON asset_models FOR ALL USING (organization_id = current_org_id() OR organization_id IS NULL);
CREATE POLICY usr_tenant ON users FOR ALL USING (organization_id = current_org_id());
CREATE POLICY team_tenant ON teams FOR ALL USING (organization_id = current_org_id());
CREATE POLICY tm_tenant ON team_members FOR ALL USING (team_id IN (SELECT id FROM teams WHERE organization_id = current_org_id()));
CREATE POLICY wo_tenant ON work_orders FOR ALL USING (organization_id = current_org_id());
CREATE POLICY wosh_tenant ON work_order_status_history FOR ALL USING (organization_id = current_org_id());
CREATE POLICY woa_tenant ON work_order_assignments FOR ALL USING (organization_id = current_org_id());
CREATE POLICY sched_tenant ON schedules FOR ALL USING (organization_id = current_org_id());
CREATE POLICY insp_tenant ON inspections FOR ALL USING (work_order_id IN (SELECT id FROM work_orders WHERE organization_id = current_org_id()));
CREATE POLICY ici_tenant ON inspection_checklist_items FOR ALL USING (inspection_id IN (SELECT id FROM inspections WHERE work_order_id IN (SELECT id FROM work_orders WHERE organization_id = current_org_id())));
CREATE POLICY part_tenant ON parts FOR ALL USING (organization_id = current_org_id());
CREATE POLICY pu_tenant ON part_usage FOR ALL USING (work_order_id IN (SELECT id FROM work_orders WHERE organization_id = current_org_id()));
CREATE POLICY ap_tenant ON asset_parts FOR ALL USING (asset_type_id IN (SELECT id FROM asset_types WHERE organization_id = current_org_id() OR is_system = true));
CREATE POLICY doc_tenant ON documents FOR ALL USING (organization_id = current_org_id());
CREATE POLICY dl_tenant ON document_links FOR ALL USING (organization_id = current_org_id());
CREATE POLICY dc_tenant ON document_chunks FOR ALL USING (organization_id = current_org_id());
CREATE POLICY emb_tenant ON embedding_records FOR ALL USING (organization_id = current_org_id());
CREATE POLICY act_tenant ON activities FOR ALL USING (organization_id = current_org_id());
CREATE POLICY aic_tenant ON ai_conversations FOR ALL USING (organization_id = current_org_id());
CREATE POLICY aim_tenant ON ai_messages FOR ALL USING (organization_id = current_org_id());
CREATE POLICY art_tenant ON ai_retrieval_traces FOR ALL USING (organization_id = current_org_id());
CREATE POLICY agent_tenant ON agent_identities FOR ALL USING (organization_id = current_org_id() OR organization_id IS NULL);
CREATE POLICY idk_tenant ON idempotency_keys FOR ALL USING (organization_id = current_org_id());
CREATE POLICY outbox_tenant ON outbox FOR ALL USING (aggregate_id IN (SELECT id FROM organizations WHERE id = current_org_id()));
CREATE POLICY orgset_tenant ON organization_settings FOR ALL USING (organization_id = current_org_id());
