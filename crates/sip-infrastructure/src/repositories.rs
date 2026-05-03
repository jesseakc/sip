use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::prelude::*;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use sip_domain::{
    entity::activity::{Activity, ActivitySource},
    entity::ai_conversation::{AIConversation, AIMessage, AIRetrievalTrace, RetrieverType},
    entity::asset::{Asset, AssetStatus, Criticality},
    entity::asset_model::{AssetModel, LifecycleStatus},
    entity::asset_type::AssetType,
    entity::document::{Document, DocumentSourceType, DocumentType, ProcessingStatus, Visibility},
    entity::embedding_record::{EmbeddingRecord, EmbeddingSourceType},
    entity::inspection::{ChecklistResult, Inspection, InspectionChecklistItem},
    entity::location::{Location, LocationType},
    entity::manufacturer::Manufacturer,
    entity::migration::{
        MigrationBatch, MigrationCheckpoint, MigrationDuplicateCandidate, MigrationExternalIdMap,
        MigrationFieldMapping, MigrationImportResult, MigrationJob, MigrationJobStatus,
        MigrationRun, MigrationSourceRecord, MigrationStagedRecord, MigrationValidationIssue,
    },
    entity::organization::Organization,
    entity::part::Part,
    entity::part::PartUsage,
    entity::schedule::{Schedule, ScheduleTriggerType},
    entity::team::Team,
    entity::user::{User, UserRole},
    entity::work_order::{
        ActorType, AssigneeType, AssignmentRole, AssignmentStatus, WorkOrder, WorkOrderAssignment,
        WorkOrderPriority, WorkOrderSourceType, WorkOrderStatus, WorkOrderStatusHistory,
        WorkOrderType,
    },
    error::SipError,
    id::{
        AIConversationId, AIMessageId, AIRetrievalTraceId, ActivityId, AssetId, AssetModelId,
        AssetTypeId, DocumentId, EmbeddingRecordId, InspectionId, LocationId, ManufacturerId,
        MigrationBatchId, MigrationCheckpointId, MigrationDuplicateCandidateId,
        MigrationExternalIdMapId, MigrationFieldMappingId, MigrationImportResultId, MigrationJobId,
        MigrationRunId, MigrationSourceRecordId, MigrationStagedRecordId,
        MigrationValidationIssueId, OrganizationId, PartId, PartUsageId, ScheduleId, TeamId,
        UserId, WorkOrderAssignmentId, WorkOrderId, WorkOrderStatusHistoryId,
    },
    repository::{
        AIConversationRepository, ActivityRepository, AssetRepository, DocumentRepository,
        EmbeddingRecordRepository, InspectionRepository, PartRepository, PartUsageRepository,
        ScheduleRepository, WorkOrderAssignmentRepository, WorkOrderRepository,
        WorkOrderStatusHistoryRepository,
    },
    tenant::TenantContext,
};
use sip_tenancy::{begin_tx_with_rls, set_rls_org_pool};

#[derive(Debug, Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

#[derive(FromRow, Clone)]
struct UserRow {
    id: Uuid,
    organization_id: Uuid,
    email: String,
    name: String,
    role: String,
    skills: Vec<String>,
    certifications: Option<serde_json::Value>,
    working_hours: Option<serde_json::Value>,
    is_active: bool,
    password_hash: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_email(&self, email: &str) -> Result<Option<(User, String)>, SipError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, organization_id, email, name, role, skills, to_jsonb(certifications) as certifications, working_hours, is_active, password_hash, created_at, updated_at FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        Ok(row.map(|r| (map_user_row(r.clone()), r.password_hash)))
    }

    pub async fn get(&self, ctx: &TenantContext, id: UserId) -> Result<Option<User>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, organization_id, email, name, role, skills, to_jsonb(certifications) as certifications, working_hours, is_active, password_hash, created_at, updated_at FROM users WHERE id = $1"
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_user_row))
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<User>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, UserRow>(
            "SELECT id, organization_id, email, name, role, skills, to_jsonb(certifications) as certifications, working_hours, is_active, password_hash, created_at, updated_at FROM users ORDER BY name LIMIT 200"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_user_row).collect())
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        user: &User,
        password_hash: &str,
    ) -> Result<User, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, UserRow>(
            "INSERT INTO users (id, organization_id, email, name, role, skills, to_jsonb(certifications) as certifications, working_hours, is_active, password_hash, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING *"
        )
        .bind(Uuid::from(user.id))
        .bind(Uuid::from(user.organization_id))
        .bind(&user.email)
        .bind(&user.name)
        .bind(format!("{:?}", user.role).to_uppercase())
        .bind(&user.skills)
        .bind(&user.certifications)
        .bind(&user.working_hours)
        .bind(user.is_active)
        .bind(password_hash)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_user_row(row))
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: UserId,
        name: Option<String>,
        role: Option<UserRole>,
        is_active: Option<bool>,
    ) -> Result<User, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        if let Some(ref n) = name {
            sqlx::query("UPDATE users SET name = $1, updated_at = NOW() WHERE id = $2")
                .bind(n)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(ref r) = role {
            sqlx::query("UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2")
                .bind(format!("{:?}", r).to_uppercase())
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(active) = is_active {
            sqlx::query("UPDATE users SET is_active = $1, updated_at = NOW() WHERE id = $2")
                .bind(active)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        self.get(ctx, id)
            .await?
            .ok_or(SipError::Validation("User not found after update".into()))
    }
}

fn map_user_row(r: UserRow) -> User {
    User {
        id: UserId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        email: r.email,
        name: r.name,
        role: match r.role.as_str() {
            "ADMIN" => UserRole::Admin,
            "MANAGER" => UserRole::Manager,
            "TECHNICIAN" => UserRole::Technician,
            "VIEWER" => UserRole::Viewer,
            "VENDOR" => UserRole::Vendor,
            _ => UserRole::Auditor,
        },
        skills: r.skills,
        certifications: r.certifications.map(|v| match v {
            serde_json::Value::Array(arr) => arr,
            _ => vec![],
        }),
        working_hours: r.working_hours,
        is_active: r.is_active,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(Debug, Clone)]
pub struct PgAssetRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct AssetRow {
    id: Uuid,
    organization_id: Uuid,
    location_id: Option<Uuid>,
    parent_id: Option<Uuid>,
    asset_type_id: Uuid,
    model_id: Option<Uuid>,
    name: String,
    description: Option<String>,
    serial_number: Option<String>,
    firmware_version: Option<String>,
    software_version: Option<String>,
    hardware_revision: Option<String>,
    status: String,
    version: i32,
    criticality: String,
    installed_date: Option<chrono::NaiveDate>,
    warranty_expiry: Option<chrono::NaiveDate>,
    attributes: Option<serde_json::Value>,
    tags: Vec<String>,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl PgAssetRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetRepository for PgAssetRepository {
    async fn create_asset(&self, ctx: &TenantContext, asset: &Asset) -> Result<Asset, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, AssetRow>(
            "INSERT INTO assets (id, organization_id, location_id, parent_id, asset_type_id, model_id, name, description, serial_number, firmware_version, software_version, hardware_revision, status, version, criticality, installed_date, warranty_expiry, attributes, tags, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
             RETURNING *"
        )
        .bind(Uuid::from(asset.id))
        .bind(Uuid::from(asset.organization_id))
        .bind(asset.location_id.map(Uuid::from))
        .bind(asset.parent_id.map(Uuid::from))
        .bind(Uuid::from(asset.asset_type_id))
        .bind(asset.model_id.map(Uuid::from))
        .bind(&asset.name)
        .bind(&asset.description)
        .bind(&asset.serial_number)
        .bind(&asset.firmware_version)
        .bind(&asset.software_version)
        .bind(&asset.hardware_revision)
        .bind(format!("{:?}", asset.status).to_uppercase())
        .bind(asset.version)
        .bind(format!("{:?}", asset.criticality).to_uppercase())
        .bind(asset.installed_date)
        .bind(asset.warranty_expiry)
        .bind(&asset.attributes)
        .bind(&asset.tags)
        .bind(&asset.metadata)
        .bind(asset.created_at)
        .bind(asset.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_asset_row(row))
    }

    async fn get_asset(&self, ctx: &TenantContext, id: AssetId) -> Result<Option<Asset>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, AssetRow>("SELECT * FROM assets WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_asset_row))
    }

    async fn update_asset(
        &self,
        ctx: &TenantContext,
        id: AssetId,
        expected_version: i32,
        patch: serde_json::Value,
    ) -> Result<Asset, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        // Simplified: update only status for MVP transition support
        if let Some(status) = patch.get("status").and_then(|v| v.as_str()) {
            let result = sqlx::query("UPDATE assets SET status = $1, version = version + 1, updated_at = NOW() WHERE id = $2 AND version = $3")
                .bind(status.to_uppercase())
                .bind(Uuid::from(id))
                .bind(expected_version)
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
            if result.rows_affected() == 0 {
                let current = self.get_asset(ctx, id).await?;
                return Err(SipError::VersionConflict {
                    current_version: current.map(|a| a.version).unwrap_or(0),
                    submitted_version: expected_version,
                });
            }
        }
        let row = sqlx::query_as::<_, AssetRow>("SELECT * FROM assets WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_asset_row(row))
    }

    async fn list_assets(&self, ctx: &TenantContext) -> Result<Vec<Asset>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, AssetRow>(
            "SELECT * FROM assets ORDER BY created_at DESC LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_asset_row).collect())
    }

    async fn archive_asset(&self, ctx: &TenantContext, id: AssetId) -> Result<Asset, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query("UPDATE assets SET status = 'RETIRED', updated_at = NOW() WHERE id = $1")
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, AssetRow>("SELECT * FROM assets WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_asset_row(row))
    }

    async fn list_children(
        &self,
        ctx: &TenantContext,
        parent_id: AssetId,
    ) -> Result<Vec<Asset>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, AssetRow>(
            "SELECT * FROM assets WHERE parent_id = $1 ORDER BY name LIMIT 200",
        )
        .bind(Uuid::from(parent_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_asset_row).collect())
    }
}

fn map_asset_row(r: AssetRow) -> Asset {
    Asset {
        id: AssetId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        location_id: r.location_id.map(LocationId::from),
        parent_id: r.parent_id.map(AssetId::from),
        asset_type_id: sip_domain::id::AssetTypeId::from(r.asset_type_id),
        model_id: r.model_id.map(sip_domain::id::AssetModelId::from),
        name: r.name,
        description: r.description,
        serial_number: r.serial_number,
        firmware_version: r.firmware_version,
        software_version: r.software_version,
        hardware_revision: r.hardware_revision,
        status: match r.status.as_str() {
            "OPERATIONAL" => AssetStatus::Operational,
            "DEGRADED" => AssetStatus::Degraded,
            "DOWN" => AssetStatus::Down,
            "MAINTENANCE" => AssetStatus::Maintenance,
            _ => AssetStatus::Retired,
        },
        version: r.version,
        criticality: match r.criticality.as_str() {
            "LOW" => Criticality::Low,
            "HIGH" => Criticality::High,
            "CRITICAL" => Criticality::Critical,
            _ => Criticality::Medium,
        },
        installed_date: r.installed_date,
        warranty_expiry: r.warranty_expiry,
        attributes: r.attributes,
        tags: r.tags,
        metadata: r.metadata,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(Debug, Clone)]
pub struct PgWorkOrderRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct WorkOrderRow {
    id: Uuid,
    organization_id: Uuid,
    asset_id: Uuid,
    parent_id: Option<Uuid>,
    schedule_id: Option<Uuid>,
    work_order_type: String,
    priority: String,
    status: String,
    title: String,
    display_number: String,
    description: String,
    scheduled_start: Option<chrono::DateTime<Utc>>,
    scheduled_end: Option<chrono::DateTime<Utc>>,
    actual_start: Option<chrono::DateTime<Utc>>,
    actual_end: Option<chrono::DateTime<Utc>>,
    due_at: Option<chrono::DateTime<Utc>>,
    estimated_hours: Option<f64>,
    actual_hours: Option<f64>,
    resolution_notes: Option<String>,
    failure_code: Option<String>,
    root_cause: Option<String>,
    created_by_id: Uuid,
    source_type: String,
    source_system: Option<String>,
    external_id: Option<String>,
    external_url: Option<String>,
    reopened_count: i32,
    last_reopened_at: Option<chrono::DateTime<Utc>>,
    last_reopened_by_id: Option<Uuid>,
    version: i32,
    archived_at: Option<chrono::DateTime<Utc>>,
    archived_by_id: Option<Uuid>,
    archive_reason: Option<String>,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl PgWorkOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkOrderRepository for PgWorkOrderRepository {
    async fn create_work_order(
        &self,
        ctx: &TenantContext,
        wo: &WorkOrder,
    ) -> Result<WorkOrder, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, WorkOrderRow>(
            "INSERT INTO work_orders (id, organization_id, asset_id, parent_id, schedule_id, type, priority, status, title, display_number, description, scheduled_start, scheduled_end, actual_start, actual_end, due_at, estimated_hours, actual_hours, resolution_notes, failure_code, root_cause, created_by_id, source_type, source_system, external_id, external_url, reopened_count, last_reopened_at, last_reopened_by_id, version, archived_at, archived_by_id, archive_reason, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36)
             RETURNING *"
        )
        .bind(Uuid::from(wo.id))
        .bind(Uuid::from(wo.organization_id))
        .bind(Uuid::from(wo.asset_id))
        .bind(wo.parent_id.map(Uuid::from))
        .bind(wo.schedule_id.map(Uuid::from))
        .bind(format!("{:?}", wo.work_order_type).to_uppercase())
        .bind(format!("{:?}", wo.priority).to_uppercase())
        .bind(format!("{:?}", wo.status).to_uppercase())
        .bind(&wo.title)
        .bind(&wo.display_number)
        .bind(&wo.description)
        .bind(wo.scheduled_start)
        .bind(wo.scheduled_end)
        .bind(wo.actual_start)
        .bind(wo.actual_end)
        .bind(wo.due_at)
        .bind(wo.estimated_hours.map(|d| d.to_f64().unwrap_or_default()))
        .bind(wo.actual_hours.map(|d| d.to_f64().unwrap_or_default()))
        .bind(&wo.resolution_notes)
        .bind(&wo.failure_code)
        .bind(&wo.root_cause)
        .bind(Uuid::from(wo.created_by_id))
        .bind(format!("{:?}", wo.source_type).to_uppercase())
        .bind(&wo.source_system)
        .bind(&wo.external_id)
        .bind(&wo.external_url)
        .bind(wo.reopened_count)
        .bind(wo.last_reopened_at)
        .bind(wo.last_reopened_by_id.map(Uuid::from))
        .bind(wo.version)
        .bind(wo.archived_at)
        .bind(wo.archived_by_id.map(Uuid::from))
        .bind(&wo.archive_reason)
        .bind(&wo.metadata)
        .bind(wo.created_at)
        .bind(wo.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_work_order_row(row))
    }

    async fn get_work_order(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
    ) -> Result<Option<WorkOrder>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, WorkOrderRow>("SELECT * FROM work_orders WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_work_order_row))
    }

    async fn update_work_order(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        expected_version: i32,
        patch: serde_json::Value,
    ) -> Result<WorkOrder, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let status = patch.get("status").and_then(|v| v.as_str());
        let notes = patch.get("resolution_notes").and_then(|v| v.as_str());
        let reopened_count = patch
            .get("reopened_count")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        let last_reopened_at: Option<chrono::DateTime<Utc>> =
            patch.get("last_reopened_at").and_then(|v| {
                v.as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
            });
        let last_reopened_by_id = patch
            .get("last_reopened_by_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        let mut actual_start: Option<chrono::DateTime<Utc>> = None;
        let mut actual_end: Option<chrono::DateTime<Utc>> = None;
        if let Some(s) = status {
            if s.eq_ignore_ascii_case("IN_PROGRESS") {
                actual_start = Some(Utc::now());
            }
            if s.eq_ignore_ascii_case("COMPLETED") {
                actual_end = Some(Utc::now());
            }
        }

        let result = sqlx::query(
            "UPDATE work_orders SET status = COALESCE($1, status), actual_start = COALESCE($2, actual_start), actual_end = COALESCE($3, actual_end), resolution_notes = COALESCE($4, resolution_notes), reopened_count = COALESCE($5, reopened_count), last_reopened_at = COALESCE($6, last_reopened_at), last_reopened_by_id = COALESCE($7, last_reopened_by_id), version = version + 1, updated_at = NOW() WHERE id = $8 AND version = $9"
        )
        .bind(status.map(|s| s.to_uppercase()))
        .bind(actual_start)
        .bind(actual_end)
        .bind(notes)
        .bind(reopened_count)
        .bind(last_reopened_at)
        .bind(last_reopened_by_id)
        .bind(Uuid::from(id))
        .bind(expected_version)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        if result.rows_affected() == 0 {
            let current = self.get_work_order(ctx, id).await?;
            return Err(SipError::VersionConflict {
                current_version: current.map(|wo| wo.version).unwrap_or(0),
                submitted_version: expected_version,
            });
        }

        let row = sqlx::query_as::<_, WorkOrderRow>("SELECT * FROM work_orders WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_work_order_row(row))
    }

    async fn list_work_orders(&self, ctx: &TenantContext) -> Result<Vec<WorkOrder>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, WorkOrderRow>(
            "SELECT * FROM work_orders ORDER BY created_at DESC LIMIT 200",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_work_order_row).collect())
    }

    async fn get_max_display_number_for_year(
        &self,
        ctx: &TenantContext,
        year: i32,
    ) -> Result<Option<String>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT display_number FROM work_orders WHERE display_number LIKE $1 ORDER BY display_number DESC LIMIT 1"
        )
        .bind(format!("WO-{}-%", year))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(|r| r.0))
    }

    async fn archive_work_order(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        archived_by_id: UserId,
    ) -> Result<WorkOrder, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query("UPDATE work_orders SET archived_at = NOW(), archived_by_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(Uuid::from(archived_by_id))
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, WorkOrderRow>("SELECT * FROM work_orders WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_work_order_row(row))
    }

    async fn list_work_orders_by_asset(
        &self,
        ctx: &TenantContext,
        asset_id: AssetId,
    ) -> Result<Vec<WorkOrder>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, WorkOrderRow>(
            "SELECT * FROM work_orders WHERE asset_id = $1 ORDER BY created_at DESC LIMIT 200",
        )
        .bind(Uuid::from(asset_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_work_order_row).collect())
    }
}

fn map_work_order_row(r: WorkOrderRow) -> WorkOrder {
    WorkOrder {
        id: WorkOrderId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        asset_id: AssetId::from(r.asset_id),
        parent_id: r.parent_id.map(WorkOrderId::from),
        schedule_id: r.schedule_id.map(sip_domain::id::ScheduleId::from),
        work_order_type: match r.work_order_type.as_str() {
            "PREVENTIVE" => WorkOrderType::Preventive,
            "INSPECTION" => WorkOrderType::Inspection,
            "EMERGENCY" => WorkOrderType::Emergency,
            _ => WorkOrderType::Corrective,
        },
        priority: match r.priority.as_str() {
            "LOW" => WorkOrderPriority::Low,
            "HIGH" => WorkOrderPriority::High,
            "CRITICAL" => WorkOrderPriority::Critical,
            _ => WorkOrderPriority::Medium,
        },
        status: match r.status.as_str() {
            "DRAFT" => WorkOrderStatus::Draft,
            "OPEN" => WorkOrderStatus::Open,
            "ASSIGNED" => WorkOrderStatus::Assigned,
            "ACCEPTED" => WorkOrderStatus::Accepted,
            "IN_PROGRESS" => WorkOrderStatus::InProgress,
            "ON_HOLD" => WorkOrderStatus::OnHold,
            "COMPLETED" => WorkOrderStatus::Completed,
            "REVIEWED" => WorkOrderStatus::Reviewed,
            "CLOSED" => WorkOrderStatus::Closed,
            _ => WorkOrderStatus::Cancelled,
        },
        title: r.title,
        display_number: r.display_number,
        description: r.description,
        scheduled_start: r.scheduled_start,
        scheduled_end: r.scheduled_end,
        actual_start: r.actual_start,
        actual_end: r.actual_end,
        due_at: r.due_at,
        estimated_hours: r.estimated_hours.and_then(rust_decimal::Decimal::from_f64),
        actual_hours: r.actual_hours.and_then(rust_decimal::Decimal::from_f64),
        resolution_notes: r.resolution_notes,
        failure_code: r.failure_code,
        root_cause: r.root_cause,
        created_by_id: UserId::from(r.created_by_id),
        source_type: match r.source_type.as_str() {
            "SCHEDULE" => WorkOrderSourceType::Schedule,
            "AI_AGENT" => WorkOrderSourceType::AiAgent,
            "API" => WorkOrderSourceType::Api,
            "IMPORT" => WorkOrderSourceType::Import,
            "PLUGIN" => WorkOrderSourceType::Plugin,
            "SYSTEM" => WorkOrderSourceType::System,
            _ => WorkOrderSourceType::Manual,
        },
        source_system: r.source_system,
        external_id: r.external_id,
        external_url: r.external_url,
        reopened_count: r.reopened_count,
        last_reopened_at: r.last_reopened_at,
        last_reopened_by_id: r.last_reopened_by_id.map(UserId::from),
        version: r.version,
        archived_at: r.archived_at,
        archived_by_id: r.archived_by_id.map(UserId::from),
        archive_reason: r.archive_reason,
        metadata: r.metadata,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

// Repositories are already pub struct definitions above

#[derive(Debug, Clone)]
pub struct PgOrganizationRepository {
    pool: PgPool,
}

#[derive(FromRow, Clone)]
struct OrganizationRow {
    id: Uuid,
    name: String,
    slug: String,
    timezone: Option<String>,
    default_currency: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    archived_at: Option<chrono::DateTime<Utc>>,
}

impl PgOrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(
        &self,
        ctx: &TenantContext,
        id: OrganizationId,
    ) -> Result<Option<Organization>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, OrganizationRow>(
            "SELECT o.id, o.name, o.slug, o.created_at, o.updated_at, o.archived_at,
                    COALESCE(s.timezone, 'UTC') as timezone,
                    COALESCE(s.default_currency, 'USD') as default_currency
             FROM organizations o
             LEFT JOIN organization_settings s ON s.organization_id = o.id
             WHERE o.id = $1",
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_organization_row))
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        name: &str,
        slug: &str,
        timezone: Option<&str>,
        default_currency: Option<&str>,
    ) -> Result<Organization, SipError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query(
            "INSERT INTO organizations (id, name, slug, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        if let Some(tz) = timezone {
            let _ = sqlx::query(
                "INSERT INTO organization_settings (organization_id, timezone) VALUES ($1, $2) ON CONFLICT (organization_id) DO UPDATE SET timezone = $2"
            )
            .bind(id)
            .bind(tz)
            .execute(&self.pool)
            .await;
        }
        if let Some(curr) = default_currency {
            let _ = sqlx::query(
                "INSERT INTO organization_settings (organization_id, default_currency) VALUES ($1, $2) ON CONFLICT (organization_id) DO UPDATE SET default_currency = $2"
            )
            .bind(id)
            .bind(curr)
            .execute(&self.pool)
            .await;
        }
        self.get_by_id(ctx, OrganizationId::from(id))
            .await?
            .ok_or(SipError::Validation(
                "Organization not found after create".into(),
            ))
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: OrganizationId,
        name: Option<String>,
        timezone: Option<String>,
        default_currency: Option<String>,
    ) -> Result<Organization, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        if let Some(name) = &name {
            sqlx::query("UPDATE organizations SET name = $1, updated_at = NOW() WHERE id = $2")
                .bind(name)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        sqlx::query(
            "INSERT INTO organization_settings (organization_id) VALUES ($1)
             ON CONFLICT (organization_id) DO NOTHING",
        )
        .bind(Uuid::from(id))
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        if let Some(tz) = timezone {
            sqlx::query("UPDATE organization_settings SET timezone = $1, updated_at = NOW() WHERE organization_id = $2")
                .bind(tz)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(currency) = default_currency {
            sqlx::query("UPDATE organization_settings SET default_currency = $1, updated_at = NOW() WHERE organization_id = $2")
                .bind(currency)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        self.get_by_id(ctx, id)
            .await?
            .ok_or(SipError::Validation("Organization not found".into()))
    }
}

fn map_organization_row(r: OrganizationRow) -> Organization {
    Organization {
        id: OrganizationId::from(r.id),
        name: r.name,
        slug: r.slug,
        timezone: r.timezone,
        default_currency: r.default_currency,
        created_at: r.created_at,
        updated_at: r.updated_at,
        archived_at: r.archived_at,
    }
}

#[derive(Debug, Clone)]
pub struct PgLocationRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct LocationRow {
    id: Uuid,
    organization_id: Uuid,
    parent_id: Option<Uuid>,
    name: String,
    location_type: String,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl PgLocationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Location>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, LocationRow>(
            "SELECT id, organization_id, parent_id, name, type as location_type, metadata, created_at, updated_at FROM locations ORDER BY name LIMIT 200"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_location_row).collect())
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: LocationId,
    ) -> Result<Option<Location>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, LocationRow>(
            "SELECT id, organization_id, parent_id, name, type as location_type, metadata, created_at, updated_at FROM locations WHERE id = $1"
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_location_row))
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        location: &Location,
    ) -> Result<Location, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO locations (id, organization_id, parent_id, name, type, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(Uuid::from(location.id))
        .bind(Uuid::from(location.organization_id))
        .bind(location.parent_id.map(Uuid::from))
        .bind(&location.name)
        .bind(format!("{:?}", location.location_type).to_uppercase())
        .bind(&location.metadata)
        .bind(location.created_at)
        .bind(location.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        self.get(ctx, location.id)
            .await?
            .ok_or(SipError::Validation(
                "Location not found after insert".into(),
            ))
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: LocationId,
        name: Option<String>,
        location_type: Option<LocationType>,
    ) -> Result<Location, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        if let Some(name) = &name {
            sqlx::query("UPDATE locations SET name = $1, updated_at = NOW() WHERE id = $2")
                .bind(name)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(lt) = location_type {
            sqlx::query("UPDATE locations SET type = $1, updated_at = NOW() WHERE id = $2")
                .bind(format!("{:?}", lt).to_uppercase())
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        let row = sqlx::query_as::<_, LocationRow>(
            "SELECT id, organization_id, parent_id, name, type as location_type, metadata, created_at, updated_at FROM locations WHERE id = $1"
        )
        .bind(Uuid::from(id))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_location_row(row))
    }

    pub async fn list_children(
        &self,
        ctx: &TenantContext,
        parent_id: LocationId,
    ) -> Result<Vec<Location>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, LocationRow>(
            "SELECT id, organization_id, parent_id, name, type as location_type, metadata, created_at, updated_at FROM locations WHERE parent_id = $1 ORDER BY name LIMIT 200"
        )
        .bind(Uuid::from(parent_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_location_row).collect())
    }

    pub async fn list_assets_at(
        &self,
        ctx: &TenantContext,
        location_id: LocationId,
    ) -> Result<Vec<Asset>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, AssetRow>(
            "SELECT * FROM assets WHERE location_id = $1 ORDER BY name LIMIT 200",
        )
        .bind(Uuid::from(location_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_asset_row).collect())
    }
}

fn map_location_row(r: LocationRow) -> Location {
    Location {
        id: LocationId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        parent_id: r.parent_id.map(LocationId::from),
        name: r.name,
        location_type: match r.location_type.as_str() {
            "SITE" => LocationType::Site,
            "BUILDING" => LocationType::Building,
            "FLOOR" => LocationType::Floor,
            "ROOM" => LocationType::Room,
            "AREA" => LocationType::Area,
            _ => LocationType::Other,
        },
        geo_latitude: None,
        geo_longitude: None,
        metadata: r.metadata,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(Debug, Clone)]
pub struct PgAssetTypeRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct AssetTypeRow {
    id: Uuid,
    organization_id: Option<Uuid>,
    name: String,
    category: String,
    description: Option<String>,
    schema: Option<serde_json::Value>,
    default_pm_schedules: Option<Vec<serde_json::Value>>,
    default_inspection_template: Option<serde_json::Value>,
    icon: Option<String>,
    is_system: bool,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl PgAssetTypeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<AssetType>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, AssetTypeRow>(
            "SELECT id, organization_id, name, category, description, schema, default_pm_schedules, default_inspection_template, icon, is_system, created_at, updated_at FROM asset_types ORDER BY name LIMIT 200"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_asset_type_row).collect())
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: AssetTypeId,
    ) -> Result<Option<AssetType>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, AssetTypeRow>(
            "SELECT id, organization_id, name, category, description, schema, default_pm_schedules, default_inspection_template, icon, is_system, created_at, updated_at FROM asset_types WHERE id = $1"
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_asset_type_row))
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        asset_type: &AssetType,
    ) -> Result<AssetType, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO asset_types (id, organization_id, name, category, description, schema, default_pm_schedules, default_inspection_template, icon, is_system, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
        )
        .bind(Uuid::from(asset_type.id))
        .bind(asset_type.organization_id.map(Uuid::from))
        .bind(&asset_type.name)
        .bind(&asset_type.category)
        .bind(&asset_type.description)
        .bind(&asset_type.schema)
        .bind(&asset_type.default_pm_schedules)
        .bind(&asset_type.default_inspection_template)
        .bind(&asset_type.icon)
        .bind(asset_type.is_system)
        .bind(asset_type.created_at)
        .bind(asset_type.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(asset_type.clone())
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: AssetTypeId,
        name: Option<String>,
        category: Option<String>,
        description: Option<String>,
    ) -> Result<AssetType, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        if let Some(ref n) = name {
            sqlx::query("UPDATE asset_types SET name = $1, updated_at = NOW() WHERE id = $2")
                .bind(n)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(ref c) = category {
            sqlx::query("UPDATE asset_types SET category = $1, updated_at = NOW() WHERE id = $2")
                .bind(c)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(ref d) = description {
            sqlx::query(
                "UPDATE asset_types SET description = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(d)
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        self.get(ctx, id).await?.ok_or(SipError::Validation(
            "Asset type not found after update".into(),
        ))
    }
}

fn map_asset_type_row(r: AssetTypeRow) -> AssetType {
    AssetType {
        id: AssetTypeId::from(r.id),
        organization_id: r.organization_id.map(OrganizationId::from),
        name: r.name,
        category: r.category,
        description: r.description,
        schema: r.schema,
        default_pm_schedules: r.default_pm_schedules,
        default_inspection_template: r.default_inspection_template,
        icon: r.icon,
        is_system: r.is_system,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(Debug, Clone)]
pub struct PgManufacturerRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct ManufacturerRow {
    id: Uuid,
    organization_id: Option<Uuid>,
    name: String,
    website: Option<String>,
    support_url: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl PgManufacturerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Manufacturer>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, ManufacturerRow>(
            "SELECT id, organization_id, name, website, support_url, created_at, updated_at FROM manufacturers ORDER BY name LIMIT 200"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_manufacturer_row).collect())
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: ManufacturerId,
    ) -> Result<Option<Manufacturer>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, ManufacturerRow>(
            "SELECT id, organization_id, name, website, support_url, created_at, updated_at FROM manufacturers WHERE id = $1"
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_manufacturer_row))
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        manufacturer: &Manufacturer,
    ) -> Result<Manufacturer, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO manufacturers (id, organization_id, name, website, support_url, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(Uuid::from(manufacturer.id))
        .bind(manufacturer.organization_id.map(Uuid::from))
        .bind(&manufacturer.name)
        .bind(&manufacturer.website)
        .bind(&manufacturer.support_url)
        .bind(manufacturer.created_at)
        .bind(manufacturer.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(manufacturer.clone())
    }
}

fn map_manufacturer_row(r: ManufacturerRow) -> Manufacturer {
    Manufacturer {
        id: ManufacturerId::from(r.id),
        organization_id: r.organization_id.map(OrganizationId::from),
        name: r.name,
        website: r.website,
        support_url: r.support_url,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(Debug, Clone)]
pub struct PgAssetModelRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct AssetModelRow {
    id: Uuid,
    organization_id: Option<Uuid>,
    manufacturer_id: Uuid,
    name: String,
    model_number: String,
    revision: Option<String>,
    lifecycle_status: String,
    asset_type_id: Uuid,
    documentation_url: Option<String>,
    default_attributes: Option<serde_json::Value>,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl PgAssetModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        ctx: &TenantContext,
        manufacturer_id: Option<ManufacturerId>,
    ) -> Result<Vec<AssetModel>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let (query, has_filter) = if manufacturer_id.is_some() {
            ("SELECT id, organization_id, manufacturer_id, name, model_number, revision, lifecycle_status, asset_type_id, documentation_url, default_attributes, metadata, created_at, updated_at FROM asset_models WHERE manufacturer_id = $1 ORDER BY name LIMIT 200", true)
        } else {
            ("SELECT id, organization_id, manufacturer_id, name, model_number, revision, lifecycle_status, asset_type_id, documentation_url, default_attributes, metadata, created_at, updated_at FROM asset_models ORDER BY name LIMIT 200", false)
        };
        let rows = if has_filter {
            sqlx::query_as::<_, AssetModelRow>(query)
                .bind(Uuid::from(manufacturer_id.unwrap()))
                .fetch_all(&self.pool)
                .await
        } else {
            sqlx::query_as::<_, AssetModelRow>(query)
                .fetch_all(&self.pool)
                .await
        }
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_asset_model_row).collect())
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: AssetModelId,
    ) -> Result<Option<AssetModel>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, AssetModelRow>(
            "SELECT id, organization_id, manufacturer_id, name, model_number, revision, lifecycle_status, asset_type_id, documentation_url, default_attributes, metadata, created_at, updated_at FROM asset_models WHERE id = $1"
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_asset_model_row))
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        model: &AssetModel,
    ) -> Result<AssetModel, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, AssetModelRow>(
            "INSERT INTO asset_models (id, organization_id, manufacturer_id, name, model_number, revision, lifecycle_status, asset_type_id, documentation_url, default_attributes, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             RETURNING *"
        )
        .bind(Uuid::from(model.id))
        .bind(model.organization_id.map(Uuid::from))
        .bind(Uuid::from(model.manufacturer_id))
        .bind(&model.name)
        .bind(&model.model_number)
        .bind(&model.revision)
        .bind(format!("{:?}", model.lifecycle_status).to_uppercase())
        .bind(Uuid::from(model.asset_type_id))
        .bind(&model.documentation_url)
        .bind(&model.default_attributes)
        .bind(&model.metadata)
        .bind(model.created_at)
        .bind(model.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_asset_model_row(row))
    }
}

fn map_asset_model_row(r: AssetModelRow) -> AssetModel {
    AssetModel {
        id: AssetModelId::from(r.id),
        organization_id: r.organization_id.map(OrganizationId::from),
        manufacturer_id: ManufacturerId::from(r.manufacturer_id),
        name: r.name,
        model_number: r.model_number,
        revision: r.revision,
        lifecycle_status: match r.lifecycle_status.as_str() {
            "ACTIVE" => LifecycleStatus::Active,
            "DEPRECATED" => LifecycleStatus::Deprecated,
            "END_OF_SUPPORT" => LifecycleStatus::EndOfSupport,
            _ => LifecycleStatus::Retired,
        },
        asset_type_id: AssetTypeId::from(r.asset_type_id),
        documentation_url: r.documentation_url,
        default_attributes: r.default_attributes,
        metadata: r.metadata,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(Debug, Clone)]
pub struct PgTeamRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct TeamRow {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    description: Option<String>,
    lead_id: Option<Uuid>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl PgTeamRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Team>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, TeamRow>(
            "SELECT id, organization_id, name, description, lead_id, created_at, updated_at FROM teams ORDER BY name LIMIT 200"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_team_row).collect())
    }

    pub async fn get(&self, ctx: &TenantContext, id: TeamId) -> Result<Option<Team>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, TeamRow>(
            "SELECT id, organization_id, name, description, lead_id, created_at, updated_at FROM teams WHERE id = $1"
        )
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_team_row))
    }

    pub async fn create(&self, ctx: &TenantContext, team: &Team) -> Result<Team, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, TeamRow>(
            "INSERT INTO teams (id, organization_id, name, description, lead_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING *"
        )
        .bind(Uuid::from(team.id))
        .bind(Uuid::from(team.organization_id))
        .bind(&team.name)
        .bind(&team.description)
        .bind(team.lead_id.map(Uuid::from))
        .bind(team.created_at)
        .bind(team.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_team_row(row))
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: TeamId,
        name: Option<String>,
        description: Option<String>,
        lead_id: Option<UserId>,
    ) -> Result<Team, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        if let Some(ref n) = name {
            sqlx::query("UPDATE teams SET name = $1, updated_at = NOW() WHERE id = $2")
                .bind(n)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(ref d) = description {
            sqlx::query("UPDATE teams SET description = $1, updated_at = NOW() WHERE id = $2")
                .bind(d)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(ref lid) = lead_id {
            sqlx::query("UPDATE teams SET lead_id = $1, updated_at = NOW() WHERE id = $2")
                .bind(Uuid::from(*lid))
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        self.get(ctx, id)
            .await?
            .ok_or(SipError::Validation("Team not found after update".into()))
    }
}

fn map_team_row(r: TeamRow) -> Team {
    Team {
        id: TeamId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        name: r.name,
        description: r.description,
        lead_id: r.lead_id.map(UserId::from),
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(Debug, Clone)]
pub struct PgActivityRepository {
    pool: PgPool,
}

impl PgActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ActivityRepository for PgActivityRepository {
    async fn create_activity(
        &self,
        ctx: &TenantContext,
        activity: &Activity,
    ) -> Result<Activity, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO activities (
                id, organization_id, actor_id, actor_type, agent_identity_id,
                plugin_id, entity_type, entity_id, action, changes,
                request_id, correlation_id, source, reason, metadata,
                ip_address, user_agent, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)"
        )
        .bind(Uuid::from(activity.id))
        .bind(Uuid::from(activity.organization_id))
        .bind(activity.actor_id.map(Uuid::from))
        .bind(format!("{:?}", activity.actor_type).to_uppercase())
        .bind(activity.agent_identity_id.map(Uuid::from))
        .bind(activity.plugin_id.map(Uuid::from))
        .bind(&activity.entity_type)
        .bind(activity.entity_id)
        .bind(&activity.action)
        .bind(&activity.changes)
        .bind(&activity.request_id)
        .bind(&activity.correlation_id)
        .bind(format!("{:?}", activity.source).to_uppercase())
        .bind(&activity.reason)
        .bind(&activity.metadata)
        .bind(&activity.ip_address)
        .bind(&activity.user_agent)
        .bind(activity.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(activity.clone())
    }

    async fn list_activities_by_entity(
        &self,
        ctx: &TenantContext,
        entity_type: &str,
        entity_id: uuid::Uuid,
    ) -> Result<Vec<Activity>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, ActivityRow>(
            "SELECT * FROM activities WHERE entity_type = $1 AND entity_id = $2 ORDER BY created_at DESC LIMIT 200"
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_activity_row).collect())
    }

    async fn list_activities(&self, ctx: &TenantContext) -> Result<Vec<Activity>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, ActivityRow>(
            "SELECT * FROM activities ORDER BY created_at DESC LIMIT 200",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_activity_row).collect())
    }
}

#[allow(dead_code)]
#[derive(FromRow)]
struct ActivityRow {
    id: Uuid,
    organization_id: Uuid,
    actor_id: Option<Uuid>,
    actor_type: String,
    agent_identity_id: Option<Uuid>,
    plugin_id: Option<Uuid>,
    entity_type: String,
    entity_id: Uuid,
    action: String,
    changes: Option<serde_json::Value>,
    request_id: Option<String>,
    correlation_id: Option<String>,
    source: String,
    reason: Option<String>,
    metadata: Option<serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    created_at: chrono::DateTime<Utc>,
}

fn map_activity_row(r: ActivityRow) -> Activity {
    Activity {
        id: ActivityId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        actor_id: r.actor_id.map(UserId::from),
        actor_type: match r.actor_type.as_str() {
            "AI_AGENT" => ActorType::AiAgent,
            "SYSTEM" => ActorType::System,
            "PLUGIN" => ActorType::Plugin,
            _ => ActorType::Human,
        },
        agent_identity_id: None,
        plugin_id: None,
        entity_type: r.entity_type,
        entity_id: r.entity_id,
        action: r.action,
        changes: r.changes,
        request_id: r.request_id,
        correlation_id: r.correlation_id,
        source: match r.source.as_str() {
            "API" => ActivitySource::Api,
            "UI" => ActivitySource::Ui,
            "AI" => ActivitySource::Ai,
            "AUTOMATION" => ActivitySource::Automation,
            "PLUGIN" => ActivitySource::Plugin,
            _ => ActivitySource::System,
        },
        reason: r.reason,
        metadata: r.metadata,
        ip_address: r.ip_address,
        user_agent: r.user_agent,
        created_at: r.created_at,
    }
}

#[derive(FromRow)]
struct AIConversationRow {
    id: Uuid,
    organization_id: Uuid,
    user_id: Uuid,
    title: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

fn map_ai_conversation_row(r: AIConversationRow) -> AIConversation {
    AIConversation {
        id: AIConversationId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        user_id: UserId::from(r.user_id),
        title: r.title,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(FromRow)]
struct AIMessageRow {
    id: Uuid,
    conversation_id: Uuid,
    role: String,
    content: String,
    sources: Option<serde_json::Value>,
    created_at: chrono::DateTime<Utc>,
}

fn map_ai_message_row(r: AIMessageRow) -> AIMessage {
    AIMessage {
        id: AIMessageId::from(r.id),
        conversation_id: AIConversationId::from(r.conversation_id),
        role: r.role,
        content: r.content,
        sources: r.sources,
        created_at: r.created_at,
    }
}

#[derive(FromRow)]
struct AIRetrievalTraceRow {
    id: Uuid,
    organization_id: Uuid,
    message_id: Uuid,
    retriever_type: String,
    source_type: String,
    source_id: Uuid,
    score: Option<f64>,
    included_in_context: bool,
    query: Option<String>,
    strategy: Option<String>,
    records_queried: Option<i32>,
    records_returned: Option<i32>,
    duration_ms: Option<i32>,
    source_scope: Option<String>,
    verified_by_sql: Option<bool>,
    rank: Option<i32>,
    created_at: chrono::DateTime<Utc>,
}

fn map_ai_retrieval_trace_row(r: AIRetrievalTraceRow) -> AIRetrievalTrace {
    AIRetrievalTrace {
        id: AIRetrievalTraceId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        message_id: AIMessageId::from(r.message_id),
        retriever_type: match r.retriever_type.as_str() {
            "VECTOR" => RetrieverType::Vector,
            "RAG" => RetrieverType::RAG,
            "GRAPH" => RetrieverType::GRAPH,
            "TEMPORAL" => RetrieverType::TEMPORAL,
            "TOOL" => RetrieverType::Tool,
            _ => RetrieverType::SQL,
        },
        source_type: r.source_type,
        source_id: r.source_id,
        query: r.query.unwrap_or_default(),
        strategy: r.strategy.unwrap_or_default(),
        records_queried: r.records_queried.unwrap_or(0),
        records_returned: r.records_returned.unwrap_or(0),
        duration_ms: r.duration_ms.unwrap_or(0),
        score: r.score.and_then(rust_decimal::Decimal::from_f64),
        included_in_context: r.included_in_context,
        source_scope: r.source_scope,
        verified_by_sql: r.verified_by_sql.unwrap_or(false),
        rank: r.rank,
        created_at: r.created_at,
    }
}

#[derive(Debug, Clone)]
pub struct PgAIConversationRepository {
    pool: PgPool,
}

impl PgAIConversationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AIConversationRepository for PgAIConversationRepository {
    async fn create_conversation(
        &self,
        ctx: &TenantContext,
        conversation: &AIConversation,
    ) -> Result<AIConversation, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO ai_conversations (id, organization_id, user_id, title, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(Uuid::from(conversation.id))
        .bind(Uuid::from(conversation.organization_id))
        .bind(Uuid::from(conversation.user_id))
        .bind(&conversation.title)
        .bind(conversation.created_at)
        .bind(conversation.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(conversation.clone())
    }

    async fn create_message(
        &self,
        ctx: &TenantContext,
        message: &AIMessage,
    ) -> Result<AIMessage, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO ai_messages (
                id, organization_id, conversation_id, role, content,
                content_retained, retention_policy_at_creation, sources, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(Uuid::from(message.id))
        .bind(Uuid::from(ctx.organization_id))
        .bind(Uuid::from(message.conversation_id))
        .bind(&message.role)
        .bind(&message.content)
        .bind(true)
        .bind("METADATA_ONLY")
        .bind(&message.sources)
        .bind(message.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(message.clone())
    }

    async fn create_retrieval_trace(
        &self,
        ctx: &TenantContext,
        trace: &AIRetrievalTrace,
    ) -> Result<AIRetrievalTrace, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let retriever_type = match trace.retriever_type {
            RetrieverType::Vector => "VECTOR",
            RetrieverType::RAG => "RAG",
            RetrieverType::GRAPH => "GRAPH",
            RetrieverType::TEMPORAL => "TEMPORAL",
            RetrieverType::Tool => "TOOL",
            RetrieverType::SQL => "RELATIONAL",
        };
        sqlx::query(
            "INSERT INTO ai_retrieval_traces (
                id, organization_id, message_id, retriever_type, source_type,
                source_id, score, included_in_context, query, strategy,
                records_queried, records_returned, duration_ms,
                source_scope, verified_by_sql, rank, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
        )
        .bind(Uuid::from(trace.id))
        .bind(Uuid::from(ctx.organization_id))
        .bind(Uuid::from(trace.message_id))
        .bind(retriever_type)
        .bind(&trace.source_type)
        .bind(trace.source_id)
        .bind(trace.score.map(|d| d.to_f64().unwrap_or_default()))
        .bind(trace.included_in_context)
        .bind(&trace.query)
        .bind(&trace.strategy)
        .bind(trace.records_queried)
        .bind(trace.records_returned)
        .bind(trace.duration_ms)
        .bind(&trace.source_scope)
        .bind(trace.verified_by_sql)
        .bind(trace.rank)
        .bind(trace.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(trace.clone())
    }

    async fn list_conversations_by_user(
        &self,
        ctx: &TenantContext,
        user_id: UserId,
    ) -> Result<Vec<AIConversation>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, AIConversationRow>(
            "SELECT id, organization_id, user_id, title, created_at, updated_at FROM ai_conversations WHERE user_id = $1 ORDER BY updated_at DESC LIMIT 100"
        )
        .bind(Uuid::from(user_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_ai_conversation_row).collect())
    }

    async fn get_messages_by_conversation(
        &self,
        ctx: &TenantContext,
        conversation_id: AIConversationId,
    ) -> Result<Vec<AIMessage>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, AIMessageRow>(
            "SELECT id, conversation_id, role, content, sources, created_at FROM ai_messages WHERE conversation_id = $1 ORDER BY created_at ASC LIMIT 200"
        )
        .bind(Uuid::from(conversation_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_ai_message_row).collect())
    }

    async fn list_traces_by_message(
        &self,
        ctx: &TenantContext,
        message_id: AIMessageId,
    ) -> Result<Vec<AIRetrievalTrace>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, AIRetrievalTraceRow>(
            "SELECT id, organization_id, message_id,
                    COALESCE(retriever_type::TEXT, 'SQL') as retriever_type,
                    source_type, source_id, score, included_in_context,
                    query, strategy, records_queried, records_returned, duration_ms,
                    source_scope, verified_by_sql, rank, created_at
             FROM ai_retrieval_traces WHERE message_id = $1 ORDER BY created_at ASC",
        )
        .bind(Uuid::from(message_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_ai_retrieval_trace_row).collect())
    }

    async fn submit_feedback(
        &self,
        ctx: &TenantContext,
        message_id: AIMessageId,
        rating: i32,
        comment: Option<String>,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rating_str = match rating {
            1 => "CORRECT",
            2 => "INCORRECT",
            3 => "INCOMPLETE",
            4 => "NOT_HELPFUL",
            5 => "HELPFUL",
            _ => "HELPFUL",
        };
        sqlx::query(
            "INSERT INTO ai_answer_feedback (id, organization_id, message_id, user_id, rating, comment, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, NOW())"
        )
        .bind(Uuid::new_v4())
        .bind(Uuid::from(ctx.organization_id))
        .bind(Uuid::from(message_id))
        .bind(ctx.user_id.map(Uuid::from).unwrap_or_else(Uuid::nil))
        .bind(rating_str)
        .bind(&comment)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PgDocumentRepository {
    pool: PgPool,
}

impl PgDocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct DocumentRow {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    document_type: String,
    mime_type: String,
    size_bytes: i64,
    document_version: Option<String>,
    checksum: String,
    source_type: String,
    source_system: Option<String>,
    external_id: Option<String>,
    external_url: Option<String>,
    storage_path: String,
    visibility: String,
    processing_status: String,
    processing_error: Option<String>,
    extracted_text_path: Option<String>,
    text_content: Option<String>,
    effective_date: Option<chrono::NaiveDate>,
    expiration_date: Option<chrono::NaiveDate>,
    supersedes_document_id: Option<Uuid>,
    version: i32,
    archived_at: Option<chrono::DateTime<Utc>>,
    archived_by_id: Option<Uuid>,
    archive_reason: Option<String>,
    metadata: Option<serde_json::Value>,
    uploaded_by_id: Uuid,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

fn map_document_row(r: DocumentRow) -> Document {
    Document {
        id: DocumentId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        name: r.name,
        document_type: match r.document_type.as_str() {
            "MANUAL" => DocumentType::Manual,
            "PROCEDURE" => DocumentType::Procedure,
            "DIAGRAM" => DocumentType::Diagram,
            "WARRANTY" => DocumentType::Warranty,
            "CERTIFICATE" => DocumentType::Certificate,
            "PHOTO" => DocumentType::Photo,
            _ => DocumentType::Other,
        },
        mime_type: r.mime_type,
        size_bytes: r.size_bytes,
        document_version: r.document_version,
        checksum: r.checksum,
        source_type: match r.source_type.as_str() {
            "API" => DocumentSourceType::Api,
            "PLUGIN" => DocumentSourceType::Plugin,
            "SYSTEM" => DocumentSourceType::System,
            "VENDOR" => DocumentSourceType::Vendor,
            "PUBLIC_IMPORT" => DocumentSourceType::PublicImport,
            _ => DocumentSourceType::Upload,
        },
        source_system: r.source_system,
        external_id: r.external_id,
        external_url: r.external_url,
        storage_path: r.storage_path,
        visibility: match r.visibility.as_str() {
            "PRIVATE_TENANT" => Visibility::PrivateTenant,
            "SHARED_VENDOR" => Visibility::SharedVendor,
            "PUBLIC" => Visibility::Public,
            _ => Visibility::SystemDefault,
        },
        processing_status: match r.processing_status.as_str() {
            "PENDING" => ProcessingStatus::Pending,
            "EXTRACTING" => ProcessingStatus::Extracting,
            "EXTRACTED" => ProcessingStatus::Extracted,
            "CHUNKING" => ProcessingStatus::Chunking,
            "EMBEDDING" => ProcessingStatus::Embedding,
            "INDEXED" => ProcessingStatus::Indexed,
            _ => ProcessingStatus::Failed,
        },
        processing_error: r.processing_error,
        extracted_text_path: r.extracted_text_path,
        text_content: r.text_content,
        effective_date: r.effective_date,
        expiration_date: r.expiration_date,
        supersedes_document_id: r.supersedes_document_id.map(DocumentId::from),
        version: r.version,
        archived_at: r.archived_at,
        archived_by_id: r.archived_by_id.map(UserId::from),
        archive_reason: r.archive_reason,
        metadata: r.metadata,
        uploaded_by_id: UserId::from(r.uploaded_by_id),
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[async_trait]
impl DocumentRepository for PgDocumentRepository {
    async fn create_document(
        &self,
        ctx: &TenantContext,
        document: &Document,
    ) -> Result<Document, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, DocumentRow>(
            "INSERT INTO documents (id, organization_id, name, type, mime_type, size_bytes, document_version, checksum, source_type, source_system, external_id, external_url, storage_path, visibility, processing_status, processing_error, extracted_text_path, text_content, effective_date, expiration_date, supersedes_document_id, version, archived_at, archived_by_id, archive_reason, metadata, uploaded_by_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29)
             RETURNING *"
        )
        .bind(Uuid::from(document.id))
        .bind(Uuid::from(document.organization_id))
        .bind(&document.name)
        .bind(format!("{:?}", document.document_type).to_uppercase())
        .bind(&document.mime_type)
        .bind(document.size_bytes)
        .bind(&document.document_version)
        .bind(&document.checksum)
        .bind(format!("{:?}", document.source_type).to_uppercase())
        .bind(&document.source_system)
        .bind(&document.external_id)
        .bind(&document.external_url)
        .bind(&document.storage_path)
        .bind(format!("{:?}", document.visibility).to_uppercase())
        .bind(format!("{:?}", document.processing_status).to_uppercase())
        .bind(&document.processing_error)
        .bind(&document.extracted_text_path)
        .bind(&document.text_content)
        .bind(document.effective_date)
        .bind(document.expiration_date)
        .bind(document.supersedes_document_id.map(Uuid::from))
        .bind(document.version)
        .bind(document.archived_at)
        .bind(document.archived_by_id.map(Uuid::from))
        .bind(&document.archive_reason)
        .bind(&document.metadata)
        .bind(Uuid::from(document.uploaded_by_id))
        .bind(document.created_at)
        .bind(document.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_document_row(row))
    }

    async fn get_document(
        &self,
        ctx: &TenantContext,
        id: DocumentId,
    ) -> Result<Option<Document>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, DocumentRow>("SELECT * FROM documents WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_document_row))
    }

    async fn update_document_processing_status(
        &self,
        ctx: &TenantContext,
        id: DocumentId,
        status: ProcessingStatus,
        error: Option<String>,
    ) -> Result<Document, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query("UPDATE documents SET processing_status = $1, processing_error = $2, updated_at = NOW() WHERE id = $3")
            .bind(format!("{:?}", status).to_uppercase())
            .bind(&error)
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, DocumentRow>("SELECT * FROM documents WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_document_row(row))
    }

    async fn list_documents(&self, ctx: &TenantContext) -> Result<Vec<Document>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, DocumentRow>(
            "SELECT * FROM documents ORDER BY created_at DESC LIMIT 200",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_document_row).collect())
    }

    async fn archive(
        &self,
        ctx: &TenantContext,
        id: DocumentId,
        archived_by_id: UserId,
        reason: &str,
    ) -> Result<Document, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query("UPDATE documents SET archived_at = NOW(), archived_by_id = $1, archive_reason = $2, updated_at = NOW() WHERE id = $3")
            .bind(Uuid::from(archived_by_id))
            .bind(reason)
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, DocumentRow>("SELECT * FROM documents WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_document_row(row))
    }
}

#[derive(Debug, Clone)]
pub struct PgWorkOrderAssignmentRepository {
    pool: PgPool,
}

impl PgWorkOrderAssignmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct WorkOrderAssignmentRow {
    id: Uuid,
    organization_id: Uuid,
    work_order_id: Uuid,
    assignee_type: String,
    assignee_id: Uuid,
    role: String,
    assigned_at: chrono::DateTime<Utc>,
    assigned_by: Uuid,
    accepted_at: Option<chrono::DateTime<Utc>>,
    removed_at: Option<chrono::DateTime<Utc>>,
    status: String,
}

fn map_work_order_assignment_row(r: WorkOrderAssignmentRow) -> WorkOrderAssignment {
    WorkOrderAssignment {
        id: WorkOrderAssignmentId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        work_order_id: WorkOrderId::from(r.work_order_id),
        assignee_type: match r.assignee_type.as_str() {
            "USER" => AssigneeType::User,
            "TEAM" => AssigneeType::Team,
            "VENDOR" => AssigneeType::Vendor,
            _ => AssigneeType::AiAgent,
        },
        assignee_id: UserId::from(r.assignee_id),
        role: match r.role.as_str() {
            "PRIMARY" => AssignmentRole::Primary,
            "SECONDARY" => AssignmentRole::Secondary,
            "OBSERVER" => AssignmentRole::Observer,
            "APPROVER" => AssignmentRole::Approver,
            "DISPATCHED_TECH" => AssignmentRole::DispatchedTech,
            _ => AssignmentRole::RemoteSupport,
        },
        assigned_at: r.assigned_at,
        assigned_by: UserId::from(r.assigned_by),
        accepted_at: r.accepted_at,
        removed_at: r.removed_at,
        status: match r.status.as_str() {
            "ASSIGNED" => AssignmentStatus::Assigned,
            "ACCEPTED" => AssignmentStatus::Accepted,
            "DECLINED" => AssignmentStatus::Declined,
            "REMOVED" => AssignmentStatus::Removed,
            _ => AssignmentStatus::Completed,
        },
    }
}

#[async_trait]
impl WorkOrderAssignmentRepository for PgWorkOrderAssignmentRepository {
    async fn create_assignment(
        &self,
        ctx: &TenantContext,
        assignment: &WorkOrderAssignment,
    ) -> Result<WorkOrderAssignment, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, WorkOrderAssignmentRow>(
            "INSERT INTO work_order_assignments (id, organization_id, work_order_id, assignee_type, assignee_id, role, assigned_at, assigned_by, accepted_at, removed_at, status)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING *"
        )
        .bind(Uuid::from(assignment.id))
        .bind(Uuid::from(assignment.organization_id))
        .bind(Uuid::from(assignment.work_order_id))
        .bind(format!("{:?}", assignment.assignee_type).to_uppercase())
        .bind(Uuid::from(assignment.assignee_id))
        .bind(format!("{:?}", assignment.role).to_uppercase())
        .bind(assignment.assigned_at)
        .bind(Uuid::from(assignment.assigned_by))
        .bind(assignment.accepted_at)
        .bind(assignment.removed_at)
        .bind(format!("{:?}", assignment.status).to_uppercase())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_work_order_assignment_row(row))
    }

    async fn list_assignments_by_work_order(
        &self,
        ctx: &TenantContext,
        work_order_id: WorkOrderId,
    ) -> Result<Vec<WorkOrderAssignment>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, WorkOrderAssignmentRow>(
            "SELECT * FROM work_order_assignments WHERE work_order_id = $1 ORDER BY assigned_at DESC"
        )
        .bind(Uuid::from(work_order_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(map_work_order_assignment_row)
            .collect())
    }

    async fn delete_assignment(
        &self,
        ctx: &TenantContext,
        id: WorkOrderAssignmentId,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query("DELETE FROM work_order_assignments WHERE id = $1")
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }

    async fn update_assignment(
        &self,
        ctx: &TenantContext,
        id: WorkOrderAssignmentId,
        status: AssignmentStatus,
    ) -> Result<WorkOrderAssignment, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let now = Utc::now();
        if matches!(status, AssignmentStatus::Accepted) {
            sqlx::query(
                "UPDATE work_order_assignments SET status = $1, accepted_at = $2 WHERE id = $3",
            )
            .bind(format!("{:?}", status).to_uppercase())
            .bind(now)
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        } else {
            sqlx::query("UPDATE work_order_assignments SET status = $1 WHERE id = $2")
                .bind(format!("{:?}", status).to_uppercase())
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        let row = sqlx::query_as::<_, WorkOrderAssignmentRow>(
            "SELECT * FROM work_order_assignments WHERE id = $1",
        )
        .bind(Uuid::from(id))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_work_order_assignment_row(row))
    }
}

#[derive(Debug, Clone)]
pub struct PgPartUsageRepository {
    pool: PgPool,
}

impl PgPartUsageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct PartUsageRow {
    id: Uuid,
    work_order_id: Uuid,
    part_id: Uuid,
    quantity: f64,
    used_by_id: Uuid,
}

fn map_part_usage_row(r: PartUsageRow) -> PartUsage {
    PartUsage {
        id: PartUsageId::from(r.id),
        work_order_id: WorkOrderId::from(r.work_order_id),
        part_id: PartId::from(r.part_id),
        quantity: rust_decimal::Decimal::from_f64(r.quantity).unwrap_or_default(),
        used_by_id: UserId::from(r.used_by_id),
    }
}

#[async_trait]
impl PartUsageRepository for PgPartUsageRepository {
    async fn create_part_usage(
        &self,
        ctx: &TenantContext,
        usage: &PartUsage,
    ) -> Result<PartUsage, SipError> {
        let mut tx = begin_tx_with_rls(&self.pool, ctx)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;

        let result = sqlx::query(
            "UPDATE parts SET quantity_on_hand = quantity_on_hand - $1, updated_at = NOW() WHERE id = $2 AND quantity_on_hand >= $1"
        )
        .bind(usage.quantity.to_f64().unwrap_or(0.0))
        .bind(Uuid::from(usage.part_id))
        .execute(&mut *tx)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(SipError::Validation("Insufficient part quantity".into()));
        }

        let low: Option<(f64, Option<f64>)> =
            sqlx::query_as("SELECT quantity_on_hand, quantity_minimum FROM parts WHERE id = $1")
                .bind(Uuid::from(usage.part_id))
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;

        if let Some((on_hand, minimum)) = low {
            if let Some(min) = minimum {
                if on_hand < min {
                    tracing::warn!(
                        "Low stock for part {}: {} on hand, {} minimum",
                        Uuid::from(usage.part_id),
                        on_hand,
                        min
                    );
                }
            }
        }

        sqlx::query(
            "INSERT INTO part_usage (id, work_order_id, part_id, quantity, used_by_id)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::from(usage.id))
        .bind(Uuid::from(usage.work_order_id))
        .bind(Uuid::from(usage.part_id))
        .bind(usage.quantity.to_f64().unwrap_or_default())
        .bind(Uuid::from(usage.used_by_id))
        .execute(&mut *tx)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(usage.clone())
    }

    async fn list_part_usage_by_work_order(
        &self,
        ctx: &TenantContext,
        work_order_id: WorkOrderId,
    ) -> Result<Vec<PartUsage>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows =
            sqlx::query_as::<_, PartUsageRow>("SELECT * FROM part_usage WHERE work_order_id = $1")
                .bind(Uuid::from(work_order_id))
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_part_usage_row).collect())
    }
}

#[derive(Debug, Clone)]
pub struct PgWorkOrderStatusHistoryRepository {
    pool: PgPool,
}

impl PgWorkOrderStatusHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[allow(dead_code)]
#[derive(FromRow)]
struct WorkOrderStatusHistoryRow {
    id: Uuid,
    organization_id: Uuid,
    work_order_id: Uuid,
    from_status: Option<String>,
    to_status: String,
    changed_by_id: Option<Uuid>,
    actor_type: String,
    agent_identity_id: Option<Uuid>,
    plugin_id: Option<Uuid>,
    reason: Option<String>,
    created_at: chrono::DateTime<Utc>,
}

fn map_work_order_status_history_row(r: WorkOrderStatusHistoryRow) -> WorkOrderStatusHistory {
    WorkOrderStatusHistory {
        id: WorkOrderStatusHistoryId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        work_order_id: WorkOrderId::from(r.work_order_id),
        from_status: r.from_status.map(|s| match s.as_str() {
            "DRAFT" => WorkOrderStatus::Draft,
            "OPEN" => WorkOrderStatus::Open,
            "ASSIGNED" => WorkOrderStatus::Assigned,
            "ACCEPTED" => WorkOrderStatus::Accepted,
            "IN_PROGRESS" => WorkOrderStatus::InProgress,
            "ON_HOLD" => WorkOrderStatus::OnHold,
            "COMPLETED" => WorkOrderStatus::Completed,
            "REVIEWED" => WorkOrderStatus::Reviewed,
            "CLOSED" => WorkOrderStatus::Closed,
            _ => WorkOrderStatus::Cancelled,
        }),
        to_status: match r.to_status.as_str() {
            "DRAFT" => WorkOrderStatus::Draft,
            "OPEN" => WorkOrderStatus::Open,
            "ASSIGNED" => WorkOrderStatus::Assigned,
            "ACCEPTED" => WorkOrderStatus::Accepted,
            "IN_PROGRESS" => WorkOrderStatus::InProgress,
            "ON_HOLD" => WorkOrderStatus::OnHold,
            "COMPLETED" => WorkOrderStatus::Completed,
            "REVIEWED" => WorkOrderStatus::Reviewed,
            "CLOSED" => WorkOrderStatus::Closed,
            _ => WorkOrderStatus::Cancelled,
        },
        changed_by_id: r.changed_by_id.map(UserId::from),
        actor_type: match r.actor_type.as_str() {
            "AI_AGENT" => ActorType::AiAgent,
            "SYSTEM" => ActorType::System,
            "PLUGIN" => ActorType::Plugin,
            _ => ActorType::Human,
        },
        agent_identity_id: None,
        plugin_id: None,
        reason: r.reason,
        created_at: r.created_at,
    }
}

#[async_trait]
impl WorkOrderStatusHistoryRepository for PgWorkOrderStatusHistoryRepository {
    async fn create_status_history(
        &self,
        ctx: &TenantContext,
        record: &WorkOrderStatusHistory,
    ) -> Result<WorkOrderStatusHistory, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, WorkOrderStatusHistoryRow>(
            "INSERT INTO work_order_status_history (id, organization_id, work_order_id, from_status, to_status, changed_by_id, actor_type, agent_identity_id, plugin_id, reason, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             RETURNING *"
        )
        .bind(Uuid::from(record.id))
        .bind(Uuid::from(record.organization_id))
        .bind(Uuid::from(record.work_order_id))
        .bind(record.from_status.map(|s| format!("{:?}", s).to_uppercase()))
        .bind(format!("{:?}", record.to_status).to_uppercase())
        .bind(record.changed_by_id.map(Uuid::from))
        .bind(format!("{:?}", record.actor_type).to_uppercase())
        .bind(record.agent_identity_id.map(Uuid::from))
        .bind(record.plugin_id.map(Uuid::from))
        .bind(&record.reason)
        .bind(record.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_work_order_status_history_row(row))
    }
}

#[derive(Debug, Clone)]
pub struct PgInspectionRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct InspectionRow {
    id: Uuid,
    work_order_id: Uuid,
    template_name: Option<String>,
}

fn map_inspection_row(r: InspectionRow) -> Inspection {
    Inspection {
        id: InspectionId::from(r.id),
        work_order_id: WorkOrderId::from(r.work_order_id),
        template_name: r.template_name.unwrap_or_default(),
    }
}

impl PgInspectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InspectionRepository for PgInspectionRepository {
    async fn get(
        &self,
        ctx: &TenantContext,
        id: InspectionId,
    ) -> Result<Option<Inspection>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, InspectionRow>(
            "SELECT i.id, i.work_order_id, i.template_name
             FROM inspections i
             JOIN work_orders w ON w.id = i.work_order_id AND w.organization_id = $1
             WHERE i.id = $2",
        )
        .bind(Uuid::from(ctx.organization_id))
        .bind(Uuid::from(id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_inspection_row))
    }

    async fn list(
        &self,
        ctx: &TenantContext,
        work_order_id: Option<WorkOrderId>,
    ) -> Result<Vec<Inspection>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = if let Some(wo_id) = work_order_id {
            sqlx::query_as::<_, InspectionRow>(
                "SELECT i.id, i.work_order_id, i.template_name
                 FROM inspections i
                 JOIN work_orders w ON w.id = i.work_order_id AND w.organization_id = $1
                 WHERE i.work_order_id = $2
                 ORDER BY i.id LIMIT 200",
            )
            .bind(Uuid::from(ctx.organization_id))
            .bind(Uuid::from(wo_id))
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, InspectionRow>(
                "SELECT i.id, i.work_order_id, i.template_name
                 FROM inspections i
                 JOIN work_orders w ON w.id = i.work_order_id AND w.organization_id = $1
                 ORDER BY i.id LIMIT 200",
            )
            .bind(Uuid::from(ctx.organization_id))
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_inspection_row).collect())
    }

    async fn update_items(
        &self,
        ctx: &TenantContext,
        inspection_id: InspectionId,
        items: Vec<InspectionChecklistItem>,
    ) -> Result<Inspection, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;

        for item in &items {
            let result_str = item.result.map(|r| match r {
                ChecklistResult::Pass => "PASS".to_string(),
                ChecklistResult::Fail => "FAIL".to_string(),
                ChecklistResult::NotApplicable => "N_A".to_string(),
            });
            sqlx::query(
                "UPDATE inspection_checklist_items
                 SET actual_value = COALESCE($1, actual_value),
                     result = COALESCE($2, result),
                     finding = COALESCE($3, finding),
                     photo_url = COALESCE($4, photo_url)
                 WHERE id = $5 AND inspection_id = $6",
            )
            .bind(&item.actual_value)
            .bind(&result_str)
            .bind(&item.finding)
            .bind(&item.photo_url)
            .bind(Uuid::from(item.id))
            .bind(Uuid::from(inspection_id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }

        self.get(ctx, inspection_id)
            .await?
            .ok_or(SipError::Validation(
                "Inspection not found after update".into(),
            ))
    }
}

#[derive(Debug, Clone)]
pub struct PgPartRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct PartRow {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    part_number: Option<String>,
    description: Option<String>,
    quantity_on_hand: f64,
    quantity_minimum: Option<f64>,
    unit: String,
    unit_cost: Option<f64>,
    storage_location: Option<String>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

fn map_part_row(r: PartRow) -> Part {
    Part {
        id: PartId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        name: r.name,
        part_number: r.part_number,
        description: r.description,
        quantity_on_hand: rust_decimal::Decimal::from_f64(r.quantity_on_hand).unwrap_or_default(),
        quantity_minimum: r.quantity_minimum.and_then(rust_decimal::Decimal::from_f64),
        unit: r.unit,
        unit_cost: r.unit_cost.and_then(rust_decimal::Decimal::from_f64),
        storage_location: r.storage_location,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

impl PgPartRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PartRepository for PgPartRepository {
    async fn create(&self, ctx: &TenantContext, part: &Part) -> Result<Part, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, PartRow>(
            "INSERT INTO parts (id, organization_id, name, part_number, description, quantity_on_hand, quantity_minimum, unit, unit_cost, storage_location, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING *"
        )
        .bind(Uuid::from(part.id))
        .bind(Uuid::from(part.organization_id))
        .bind(&part.name)
        .bind(&part.part_number)
        .bind(&part.description)
        .bind(part.quantity_on_hand.to_f64().unwrap_or_default())
        .bind(part.quantity_minimum.map(|d| d.to_f64().unwrap_or_default()))
        .bind(&part.unit)
        .bind(part.unit_cost.map(|d| d.to_f64().unwrap_or_default()))
        .bind(&part.storage_location)
        .bind(part.created_at)
        .bind(part.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_part_row(row))
    }

    async fn get(&self, ctx: &TenantContext, id: PartId) -> Result<Option<Part>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, PartRow>(
            "SELECT * FROM parts WHERE id = $1 AND organization_id = $2",
        )
        .bind(Uuid::from(id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_part_row))
    }

    async fn list(&self, ctx: &TenantContext) -> Result<Vec<Part>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, PartRow>(
            "SELECT * FROM parts WHERE organization_id = $1 ORDER BY name LIMIT 200",
        )
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_part_row).collect())
    }

    async fn update(
        &self,
        ctx: &TenantContext,
        id: PartId,
        patch: serde_json::Value,
    ) -> Result<Part, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;

        let name = patch.get("name").and_then(|v| v.as_str());
        let part_number = patch.get("part_number").and_then(|v| v.as_str());
        let description = patch.get("description").and_then(|v| v.as_str());
        let quantity_on_hand = patch.get("quantity_on_hand").and_then(|v| v.as_f64());
        let quantity_minimum = patch.get("quantity_minimum").and_then(|v| v.as_f64());
        let unit = patch.get("unit").and_then(|v| v.as_str());
        let unit_cost = patch.get("unit_cost").and_then(|v| v.as_f64());
        let storage_location = patch.get("storage_location").and_then(|v| v.as_str());

        sqlx::query(
            "UPDATE parts SET
                name = COALESCE($1, name),
                part_number = COALESCE($2, part_number),
                description = COALESCE($3, description),
                quantity_on_hand = COALESCE($4, quantity_on_hand),
                quantity_minimum = COALESCE($5, quantity_minimum),
                unit = COALESCE($6, unit),
                unit_cost = COALESCE($7, unit_cost),
                storage_location = COALESCE($8, storage_location),
                updated_at = NOW()
             WHERE id = $9 AND organization_id = $10",
        )
        .bind(name)
        .bind(part_number)
        .bind(description)
        .bind(quantity_on_hand)
        .bind(quantity_minimum)
        .bind(unit)
        .bind(unit_cost)
        .bind(storage_location)
        .bind(Uuid::from(id))
        .bind(Uuid::from(ctx.organization_id))
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        let row = sqlx::query_as::<_, PartRow>(
            "SELECT * FROM parts WHERE id = $1 AND organization_id = $2",
        )
        .bind(Uuid::from(id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_part_row(row))
    }
}

#[derive(Debug, Clone)]
pub struct PgScheduleRepository {
    pool: PgPool,
}

#[derive(FromRow)]
struct ScheduleRow {
    id: Uuid,
    organization_id: Uuid,
    asset_id: Uuid,
    name: String,
    trigger_type: String,
    trigger_config: Option<serde_json::Value>,
    work_order_template: Option<serde_json::Value>,
    next_due: Option<chrono::DateTime<Utc>>,
    last_triggered: Option<chrono::DateTime<Utc>>,
    enabled: bool,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
    archived_at: Option<chrono::DateTime<Utc>>,
    archived_by_id: Option<Uuid>,
    archive_reason: Option<String>,
}

impl PgScheduleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_schedule_row(r: ScheduleRow) -> Schedule {
    Schedule {
        id: ScheduleId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        asset_id: AssetId::from(r.asset_id),
        name: r.name,
        trigger_type: match r.trigger_type.as_str() {
            "METER" => ScheduleTriggerType::Meter,
            _ => ScheduleTriggerType::Cron,
        },
        trigger_config: r.trigger_config,
        work_order_template: r.work_order_template,
        next_due: r.next_due,
        last_triggered: r.last_triggered,
        enabled: r.enabled,
        created_at: r.created_at,
        updated_at: r.updated_at,
        archived_at: r.archived_at,
        archived_by_id: r.archived_by_id.map(UserId::from),
        archive_reason: r.archive_reason,
    }
}

#[async_trait]
impl ScheduleRepository for PgScheduleRepository {
    async fn create(&self, ctx: &TenantContext, schedule: &Schedule) -> Result<Schedule, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, ScheduleRow>(
            "INSERT INTO schedules (id, organization_id, asset_id, name, trigger_type, trigger_config, work_order_template, next_due, last_triggered, enabled, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
             RETURNING *"
        )
        .bind(Uuid::from(schedule.id))
        .bind(Uuid::from(schedule.organization_id))
        .bind(Uuid::from(schedule.asset_id))
        .bind(&schedule.name)
        .bind(format!("{:?}", schedule.trigger_type).to_uppercase())
        .bind(&schedule.trigger_config)
        .bind(&schedule.work_order_template)
        .bind(schedule.next_due)
        .bind(schedule.last_triggered)
        .bind(schedule.enabled)
        .bind(schedule.created_at)
        .bind(schedule.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_schedule_row(row))
    }

    async fn get(&self, ctx: &TenantContext, id: ScheduleId) -> Result<Option<Schedule>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, ScheduleRow>("SELECT * FROM schedules WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_schedule_row))
    }

    async fn list(&self, ctx: &TenantContext) -> Result<Vec<Schedule>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, ScheduleRow>(
            "SELECT * FROM schedules WHERE archived_at IS NULL ORDER BY created_at DESC LIMIT 200",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_schedule_row).collect())
    }

    async fn update(
        &self,
        ctx: &TenantContext,
        id: ScheduleId,
        patch: serde_json::Value,
    ) -> Result<Schedule, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        if let Some(name) = patch.get("name").and_then(|v| v.as_str()) {
            sqlx::query("UPDATE schedules SET name = $1, updated_at = NOW() WHERE id = $2")
                .bind(name)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(enabled) = patch.get("enabled").and_then(|v| v.as_bool()) {
            sqlx::query("UPDATE schedules SET enabled = $1, updated_at = NOW() WHERE id = $2")
                .bind(enabled)
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(next_due) = patch.get("next_due").and_then(|v| v.as_str()) {
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(next_due) {
                sqlx::query("UPDATE schedules SET next_due = $1, updated_at = NOW() WHERE id = $2")
                    .bind(ts.with_timezone(&Utc))
                    .bind(Uuid::from(id))
                    .execute(&self.pool)
                    .await
                    .map_err(|e| SipError::Validation(e.to_string()))?;
            }
        }
        if let Some(last_triggered) = patch.get("last_triggered").and_then(|v| v.as_str()) {
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(last_triggered) {
                sqlx::query(
                    "UPDATE schedules SET last_triggered = $1, updated_at = NOW() WHERE id = $2",
                )
                .bind(ts.with_timezone(&Utc))
                .bind(Uuid::from(id))
                .execute(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;
            }
        }
        if let Some(trigger_config) = patch.get("trigger_config") {
            sqlx::query(
                "UPDATE schedules SET trigger_config = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(trigger_config)
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        if let Some(work_order_template) = patch.get("work_order_template") {
            sqlx::query(
                "UPDATE schedules SET work_order_template = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(work_order_template)
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        let row = sqlx::query_as::<_, ScheduleRow>("SELECT * FROM schedules WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_schedule_row(row))
    }

    async fn archive(
        &self,
        ctx: &TenantContext,
        id: ScheduleId,
        archived_by: UserId,
    ) -> Result<Schedule, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query("UPDATE schedules SET archived_at = NOW(), archived_by_id = $1, updated_at = NOW() WHERE id = $2")
            .bind(Uuid::from(archived_by))
            .bind(Uuid::from(id))
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, ScheduleRow>("SELECT * FROM schedules WHERE id = $1")
            .bind(Uuid::from(id))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(map_schedule_row(row))
    }

    async fn list_active(&self, ctx: &TenantContext) -> Result<Vec<Schedule>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, ScheduleRow>(
            "SELECT * FROM schedules WHERE archived_at IS NULL AND enabled = true ORDER BY next_due ASC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_schedule_row).collect())
    }
}

#[derive(Debug, Clone)]
pub struct PgEmbeddingRecordRepository {
    pool: PgPool,
}

impl PgEmbeddingRecordRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[allow(dead_code)]
#[derive(FromRow)]
struct EmbeddingRecordRow {
    id: Uuid,
    organization_id: Uuid,
    source_type: String,
    source_id: Uuid,
    document_chunk_id: Option<Uuid>,
    collection: String,
    content: String,
    embedding_model: String,
    model_name: String,
    dimensions: i32,
    source_scope: String,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

#[allow(dead_code)]
fn map_embedding_record_row(r: EmbeddingRecordRow) -> EmbeddingRecord {
    EmbeddingRecord {
        id: EmbeddingRecordId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        source_type: match r.source_type.as_str() {
            "ASSET" => EmbeddingSourceType::Asset,
            "WORK_ORDER" => EmbeddingSourceType::WorkOrder,
            "INSPECTION_FINDING" => EmbeddingSourceType::InspectionFinding,
            _ => EmbeddingSourceType::DocumentChunk,
        },
        source_id: r.source_id,
        document_chunk_id: r
            .document_chunk_id
            .map(sip_domain::id::DocumentChunkId::from),
        collection: r.collection,
        content: r.content,
        embedding_model: r.embedding_model,
        model_name: r.model_name,
        dimensions: r.dimensions,
        source_scope: r.source_scope,
        metadata: r.metadata,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[async_trait]
impl EmbeddingRecordRepository for PgEmbeddingRecordRepository {
    async fn insert_embedding_record(
        &self,
        ctx: &TenantContext,
        record: &EmbeddingRecord,
    ) -> Result<EmbeddingRecord, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO embedding_records (
                id, organization_id, source_type, source_id, document_chunk_id,
                collection, content, embedding_model, model_name, dimensions,
                source_scope, metadata, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(Uuid::from(record.id))
        .bind(Uuid::from(record.organization_id))
        .bind(format!("{:?}", record.source_type).to_uppercase())
        .bind(record.source_id)
        .bind(record.document_chunk_id.map(Uuid::from))
        .bind(&record.collection)
        .bind(&record.content)
        .bind(&record.embedding_model)
        .bind(&record.model_name)
        .bind(record.dimensions)
        .bind(&record.source_scope)
        .bind(&record.metadata)
        .bind(record.created_at)
        .bind(record.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(record.clone())
    }

    async fn delete_embeddings_by_source(
        &self,
        ctx: &TenantContext,
        source_type: EmbeddingSourceType,
        source_id: uuid::Uuid,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query("DELETE FROM embedding_records WHERE source_type = $1 AND source_id = $2")
            .bind(format!("{:?}", source_type).to_uppercase())
            .bind(source_id)
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }
}

// ── Migration Repository ──

#[derive(Debug, Clone)]
pub struct PgMigrationRepository {
    pool: PgPool,
}

impl PgMigrationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_job(
        &self,
        ctx: &TenantContext,
        job: &MigrationJob,
    ) -> Result<MigrationJob, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO migration_jobs (id, organization_id, name, description, source_system, source_object_type, status, source_record_count, valid_record_count, imported_record_count, error_count, created_by, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"
        )
        .bind(Uuid::from(job.id))
        .bind(Uuid::from(job.organization_id))
        .bind(&job.name)
        .bind(&job.description)
        .bind(&job.source_system)
        .bind(&job.source_object_type)
        .bind(format!("{:?}", job.status).to_uppercase())
        .bind(job.source_record_count)
        .bind(job.valid_record_count)
        .bind(job.imported_record_count)
        .bind(job.error_count)
        .bind(Uuid::from(job.created_by))
        .bind(&job.metadata)
        .bind(job.created_at)
        .bind(job.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(job.clone())
    }

    pub async fn get_job(
        &self,
        ctx: &TenantContext,
        id: MigrationJobId,
    ) -> Result<Option<MigrationJob>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row = sqlx::query_as::<_, MigrationJobRow>(
            "SELECT * FROM migration_jobs WHERE id = $1 AND organization_id = $2",
        )
        .bind(Uuid::from(id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(map_migration_job_row))
    }

    pub async fn list_jobs(&self, ctx: &TenantContext) -> Result<Vec<MigrationJob>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, MigrationJobRow>(
            "SELECT * FROM migration_jobs WHERE organization_id = $1 ORDER BY created_at DESC LIMIT 200"
        )
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_migration_job_row).collect())
    }

    pub async fn update_job_status(
        &self,
        ctx: &TenantContext,
        id: MigrationJobId,
        status: MigrationJobStatus,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "UPDATE migration_jobs SET status = $1, updated_at = NOW() WHERE id = $2 AND organization_id = $3"
        )
        .bind(format!("{:?}", status).to_uppercase())
        .bind(Uuid::from(id))
        .bind(Uuid::from(ctx.organization_id))
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }

    pub async fn update_job_counts(
        &self,
        ctx: &TenantContext,
        id: MigrationJobId,
        source_record_count: i32,
        valid_record_count: i32,
        imported_record_count: i32,
        error_count: i32,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "UPDATE migration_jobs SET source_record_count = $1, valid_record_count = $2, imported_record_count = $3, error_count = $4, updated_at = NOW() WHERE id = $5 AND organization_id = $6"
        )
        .bind(source_record_count)
        .bind(valid_record_count)
        .bind(imported_record_count)
        .bind(error_count)
        .bind(Uuid::from(id))
        .bind(Uuid::from(ctx.organization_id))
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }

    pub async fn create_source_records(
        &self,
        ctx: &TenantContext,
        records: &[MigrationSourceRecord],
    ) -> Result<Vec<MigrationSourceRecord>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        for r in records {
            sqlx::query(
                "INSERT INTO migration_source_records (id, organization_id, job_id, batch_id, external_id, source_object_type, raw_data, status, row_number, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
            )
            .bind(Uuid::from(r.id))
            .bind(Uuid::from(r.organization_id))
            .bind(Uuid::from(r.job_id))
            .bind(r.batch_id.map(Uuid::from))
            .bind(&r.external_id)
            .bind(&r.source_object_type)
            .bind(&r.raw_data)
            .bind(&r.status)
            .bind(r.row_number)
            .bind(r.created_at)
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        Ok(records.to_vec())
    }

    pub async fn get_source_records(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationSourceRecord>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, MigrationSourceRecordRow>(
            "SELECT * FROM migration_source_records WHERE job_id = $1 AND organization_id = $2 ORDER BY row_number LIMIT 10000"
        )
        .bind(Uuid::from(job_id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(map_migration_source_record_row)
            .collect())
    }

    pub async fn create_staged_records(
        &self,
        ctx: &TenantContext,
        records: &[MigrationStagedRecord],
    ) -> Result<Vec<MigrationStagedRecord>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        for r in records {
            sqlx::query(
                "INSERT INTO migration_staged_records (id, organization_id, job_id, source_record_id, target_entity_type, canonical_data, status, validation_errors, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
            )
            .bind(Uuid::from(r.id))
            .bind(Uuid::from(r.organization_id))
            .bind(Uuid::from(r.job_id))
            .bind(Uuid::from(r.source_record_id))
            .bind(&r.target_entity_type)
            .bind(&r.canonical_data)
            .bind(&r.status)
            .bind(&r.validation_errors)
            .bind(r.created_at)
            .bind(r.updated_at)
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        Ok(records.to_vec())
    }

    pub async fn get_staged_records(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationStagedRecord>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, MigrationStagedRecordRow>(
            "SELECT * FROM migration_staged_records WHERE job_id = $1 AND organization_id = $2 ORDER BY created_at LIMIT 10000"
        )
        .bind(Uuid::from(job_id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(map_migration_staged_record_row)
            .collect())
    }

    pub async fn update_staged_record_status(
        &self,
        ctx: &TenantContext,
        id: MigrationStagedRecordId,
        status: &str,
        validation_errors: Option<serde_json::Value>,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "UPDATE migration_staged_records SET status = $1, validation_errors = $2, updated_at = NOW() WHERE id = $3 AND organization_id = $4"
        )
        .bind(status)
        .bind(&validation_errors)
        .bind(Uuid::from(id))
        .bind(Uuid::from(ctx.organization_id))
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }

    pub async fn save_mappings(
        &self,
        ctx: &TenantContext,
        mappings: &[MigrationFieldMapping],
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        for m in mappings {
            sqlx::query(
                "INSERT INTO migration_field_mappings (id, organization_id, job_id, target_entity_type, source_field, target_field, transform_expression, default_value, is_required, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                 ON CONFLICT (job_id, target_entity_type, source_field) DO UPDATE SET target_field = $6, transform_expression = $7, default_value = $8, is_required = $9"
            )
            .bind(Uuid::from(m.id))
            .bind(Uuid::from(m.organization_id))
            .bind(Uuid::from(m.job_id))
            .bind(&m.target_entity_type)
            .bind(&m.source_field)
            .bind(&m.target_field)
            .bind(&m.transform_expression)
            .bind(&m.default_value)
            .bind(m.is_required)
            .bind(m.created_at)
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn get_mappings(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationFieldMapping>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, MigrationFieldMappingRow>(
            "SELECT * FROM migration_field_mappings WHERE job_id = $1 AND organization_id = $2 ORDER BY target_entity_type, source_field"
        )
        .bind(Uuid::from(job_id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(map_migration_field_mapping_row)
            .collect())
    }

    pub async fn create_import_result(
        &self,
        ctx: &TenantContext,
        result: &MigrationImportResult,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO migration_import_results (id, organization_id, job_id, run_id, staged_record_id, sip_entity_type, sip_entity_id, action, error_message, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(Uuid::from(result.id))
        .bind(Uuid::from(result.organization_id))
        .bind(Uuid::from(result.job_id))
        .bind(Uuid::from(result.run_id))
        .bind(Uuid::from(result.staged_record_id))
        .bind(&result.sip_entity_type)
        .bind(result.sip_entity_id)
        .bind(&result.action)
        .bind(&result.error_message)
        .bind(result.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }

    pub async fn get_import_results(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationImportResult>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, MigrationImportResultRow>(
            "SELECT * FROM migration_import_results WHERE job_id = $1 AND organization_id = $2 ORDER BY created_at"
        )
        .bind(Uuid::from(job_id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(map_migration_import_result_row)
            .collect())
    }

    pub async fn create_external_id_map(
        &self,
        ctx: &TenantContext,
        entry: &MigrationExternalIdMap,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO migration_external_id_maps (id, organization_id, job_id, run_id, source_system, source_object_type, source_external_id, sip_entity_type, sip_entity_id, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT (source_system, source_object_type, source_external_id, organization_id) DO UPDATE SET sip_entity_id = $9"
        )
        .bind(Uuid::from(entry.id))
        .bind(Uuid::from(entry.organization_id))
        .bind(Uuid::from(entry.job_id))
        .bind(Uuid::from(entry.run_id))
        .bind(&entry.source_system)
        .bind(&entry.source_object_type)
        .bind(&entry.source_external_id)
        .bind(&entry.sip_entity_type)
        .bind(entry.sip_entity_id)
        .bind(entry.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }

    pub async fn lookup_external_id(
        &self,
        ctx: &TenantContext,
        source_system: &str,
        source_object_type: &str,
        source_external_id: &str,
    ) -> Result<Option<Uuid>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT sip_entity_id FROM migration_external_id_maps WHERE organization_id = $1 AND source_system = $2 AND source_object_type = $3 AND source_external_id = $4"
        )
        .bind(Uuid::from(ctx.organization_id))
        .bind(source_system)
        .bind(source_object_type)
        .bind(source_external_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(row.map(|r| r.0))
    }

    pub async fn get_external_id_maps(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationExternalIdMap>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, MigrationExternalIdMapRow>(
            "SELECT * FROM migration_external_id_maps WHERE job_id = $1 AND organization_id = $2 ORDER BY source_object_type, source_external_id"
        )
        .bind(Uuid::from(job_id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(map_migration_external_id_map_row)
            .collect())
    }

    pub async fn create_run(
        &self,
        ctx: &TenantContext,
        run: &MigrationRun,
    ) -> Result<MigrationRun, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO migration_runs (id, organization_id, job_id, run_type, status, records_processed, records_created, records_updated, records_skipped, records_failed, started_at, completed_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)"
        )
        .bind(Uuid::from(run.id))
        .bind(Uuid::from(run.organization_id))
        .bind(Uuid::from(run.job_id))
        .bind(format!("{:?}", run.run_type).to_uppercase())
        .bind(format!("{:?}", run.status).to_uppercase())
        .bind(run.records_processed)
        .bind(run.records_created)
        .bind(run.records_updated)
        .bind(run.records_skipped)
        .bind(run.records_failed)
        .bind(run.started_at)
        .bind(run.completed_at)
        .bind(run.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(run.clone())
    }

    pub async fn update_run(
        &self,
        ctx: &TenantContext,
        run: &MigrationRun,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "UPDATE migration_runs SET status = $1, records_processed = $2, records_created = $3, records_updated = $4, records_skipped = $5, records_failed = $6, started_at = $7, completed_at = $8 WHERE id = $9 AND organization_id = $10"
        )
        .bind(format!("{:?}", run.status).to_uppercase())
        .bind(run.records_processed)
        .bind(run.records_created)
        .bind(run.records_updated)
        .bind(run.records_skipped)
        .bind(run.records_failed)
        .bind(run.started_at)
        .bind(run.completed_at)
        .bind(Uuid::from(run.id))
        .bind(Uuid::from(ctx.organization_id))
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }

    pub async fn create_batch(
        &self,
        ctx: &TenantContext,
        batch: &MigrationBatch,
    ) -> Result<MigrationBatch, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO migration_batches (id, organization_id, job_id, batch_number, record_count, status, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (job_id, batch_number) DO NOTHING"
        )
        .bind(Uuid::from(batch.id))
        .bind(Uuid::from(batch.organization_id))
        .bind(Uuid::from(batch.job_id))
        .bind(batch.batch_number)
        .bind(batch.record_count)
        .bind(&batch.status)
        .bind(batch.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(batch.clone())
    }

    pub async fn create_validation_issues(
        &self,
        ctx: &TenantContext,
        issues: &[MigrationValidationIssue],
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        for i in issues {
            sqlx::query(
                "INSERT INTO migration_validation_issues (id, organization_id, job_id, staged_record_id, severity, field, message, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(Uuid::from(i.id))
            .bind(Uuid::from(i.organization_id))
            .bind(Uuid::from(i.job_id))
            .bind(Uuid::from(i.staged_record_id))
            .bind(&i.severity)
            .bind(&i.field)
            .bind(&i.message)
            .bind(i.created_at)
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn get_validation_issues(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationValidationIssue>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, MigrationValidationIssueRow>(
            "SELECT * FROM migration_validation_issues WHERE job_id = $1 AND organization_id = $2 ORDER BY severity, created_at"
        )
        .bind(Uuid::from(job_id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(map_migration_validation_issue_row)
            .collect())
    }

    pub async fn create_duplicate_candidates(
        &self,
        ctx: &TenantContext,
        candidates: &[MigrationDuplicateCandidate],
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        for c in candidates {
            sqlx::query(
                "INSERT INTO migration_duplicate_candidates (id, organization_id, job_id, staged_record_id, sip_entity_type, sip_entity_id, confidence_score, match_reason, status, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
            )
            .bind(Uuid::from(c.id))
            .bind(Uuid::from(c.organization_id))
            .bind(Uuid::from(c.job_id))
            .bind(Uuid::from(c.staged_record_id))
            .bind(&c.sip_entity_type)
            .bind(c.sip_entity_id)
            .bind(c.confidence_score)
            .bind(&c.match_reason)
            .bind(&c.status)
            .bind(c.created_at)
            .execute(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn get_duplicate_candidates(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationDuplicateCandidate>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, MigrationDuplicateCandidateRow>(
            "SELECT * FROM migration_duplicate_candidates WHERE job_id = $1 AND organization_id = $2 ORDER BY confidence_score DESC"
        )
        .bind(Uuid::from(job_id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(map_migration_duplicate_candidate_row)
            .collect())
    }

    pub async fn create_checkpoint(
        &self,
        ctx: &TenantContext,
        checkpoint: &MigrationCheckpoint,
    ) -> Result<(), SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        sqlx::query(
            "INSERT INTO migration_checkpoints (id, organization_id, job_id, run_id, checkpoint_type, state_data, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(Uuid::from(checkpoint.id))
        .bind(Uuid::from(checkpoint.organization_id))
        .bind(Uuid::from(checkpoint.job_id))
        .bind(Uuid::from(checkpoint.run_id))
        .bind(&checkpoint.checkpoint_type)
        .bind(&checkpoint.state_data)
        .bind(checkpoint.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(())
    }

    pub async fn get_checkpoints(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationCheckpoint>, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let rows = sqlx::query_as::<_, MigrationCheckpointRow>(
            "SELECT * FROM migration_checkpoints WHERE job_id = $1 AND organization_id = $2 ORDER BY created_at"
        )
        .bind(Uuid::from(job_id))
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok(rows.into_iter().map(map_migration_checkpoint_row).collect())
    }
}

// ── Migration Row Types and Mappers ──

#[derive(FromRow)]
struct MigrationJobRow {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    description: Option<String>,
    source_system: String,
    source_object_type: String,
    status: String,
    source_record_count: i32,
    valid_record_count: i32,
    imported_record_count: i32,
    error_count: i32,
    created_by: Uuid,
    metadata: Option<serde_json::Value>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

fn map_migration_job_row(r: MigrationJobRow) -> MigrationJob {
    MigrationJob {
        id: MigrationJobId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        name: r.name,
        description: r.description,
        source_system: r.source_system,
        source_object_type: r.source_object_type,
        status: match r.status.as_str() {
            "DRAFT" => MigrationJobStatus::Draft,
            "UPLOADED" => MigrationJobStatus::Uploaded,
            "MAPPED" => MigrationJobStatus::Mapped,
            "VALIDATED" => MigrationJobStatus::Validated,
            "READY_FOR_IMPORT" => MigrationJobStatus::ReadyForImport,
            "IMPORTING" => MigrationJobStatus::Importing,
            "COMPLETED" => MigrationJobStatus::Completed,
            "COMPLETED_WITH_WARNINGS" => MigrationJobStatus::CompletedWithWarnings,
            "FAILED" => MigrationJobStatus::Failed,
            "CANCELLED" => MigrationJobStatus::Cancelled,
            _ => MigrationJobStatus::RolledBack,
        },
        source_record_count: r.source_record_count,
        valid_record_count: r.valid_record_count,
        imported_record_count: r.imported_record_count,
        error_count: r.error_count,
        created_by: UserId::from(r.created_by),
        metadata: r.metadata,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(FromRow)]
struct MigrationSourceRecordRow {
    id: Uuid,
    organization_id: Uuid,
    job_id: Uuid,
    batch_id: Option<Uuid>,
    external_id: Option<String>,
    source_object_type: String,
    raw_data: serde_json::Value,
    status: String,
    row_number: Option<i32>,
    created_at: chrono::DateTime<Utc>,
}

fn map_migration_source_record_row(r: MigrationSourceRecordRow) -> MigrationSourceRecord {
    MigrationSourceRecord {
        id: MigrationSourceRecordId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        job_id: MigrationJobId::from(r.job_id),
        batch_id: r.batch_id.map(MigrationBatchId::from),
        external_id: r.external_id,
        source_object_type: r.source_object_type,
        raw_data: r.raw_data,
        status: r.status,
        row_number: r.row_number,
        created_at: r.created_at,
    }
}

#[derive(FromRow)]
struct MigrationStagedRecordRow {
    id: Uuid,
    organization_id: Uuid,
    job_id: Uuid,
    source_record_id: Uuid,
    target_entity_type: String,
    canonical_data: serde_json::Value,
    status: String,
    validation_errors: Option<serde_json::Value>,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

fn map_migration_staged_record_row(r: MigrationStagedRecordRow) -> MigrationStagedRecord {
    MigrationStagedRecord {
        id: MigrationStagedRecordId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        job_id: MigrationJobId::from(r.job_id),
        source_record_id: MigrationSourceRecordId::from(r.source_record_id),
        target_entity_type: r.target_entity_type,
        canonical_data: r.canonical_data,
        status: r.status,
        validation_errors: r.validation_errors,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

#[derive(FromRow)]
struct MigrationFieldMappingRow {
    id: Uuid,
    organization_id: Uuid,
    job_id: Uuid,
    target_entity_type: String,
    source_field: String,
    target_field: String,
    transform_expression: Option<String>,
    default_value: Option<String>,
    is_required: bool,
    created_at: chrono::DateTime<Utc>,
}

fn map_migration_field_mapping_row(r: MigrationFieldMappingRow) -> MigrationFieldMapping {
    MigrationFieldMapping {
        id: MigrationFieldMappingId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        job_id: MigrationJobId::from(r.job_id),
        target_entity_type: r.target_entity_type,
        source_field: r.source_field,
        target_field: r.target_field,
        transform_expression: r.transform_expression,
        default_value: r.default_value,
        is_required: r.is_required,
        created_at: r.created_at,
    }
}

#[derive(FromRow)]
struct MigrationImportResultRow {
    id: Uuid,
    organization_id: Uuid,
    job_id: Uuid,
    run_id: Uuid,
    staged_record_id: Uuid,
    sip_entity_type: String,
    sip_entity_id: Option<Uuid>,
    action: String,
    error_message: Option<String>,
    created_at: chrono::DateTime<Utc>,
}

fn map_migration_import_result_row(r: MigrationImportResultRow) -> MigrationImportResult {
    MigrationImportResult {
        id: MigrationImportResultId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        job_id: MigrationJobId::from(r.job_id),
        run_id: MigrationRunId::from(r.run_id),
        staged_record_id: MigrationStagedRecordId::from(r.staged_record_id),
        sip_entity_type: r.sip_entity_type,
        sip_entity_id: r.sip_entity_id,
        action: r.action,
        error_message: r.error_message,
        created_at: r.created_at,
    }
}

#[derive(FromRow)]
struct MigrationExternalIdMapRow {
    id: Uuid,
    organization_id: Uuid,
    job_id: Uuid,
    run_id: Uuid,
    source_system: String,
    source_object_type: String,
    source_external_id: String,
    sip_entity_type: String,
    sip_entity_id: Uuid,
    created_at: chrono::DateTime<Utc>,
}

fn map_migration_external_id_map_row(r: MigrationExternalIdMapRow) -> MigrationExternalIdMap {
    MigrationExternalIdMap {
        id: MigrationExternalIdMapId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        job_id: MigrationJobId::from(r.job_id),
        run_id: MigrationRunId::from(r.run_id),
        source_system: r.source_system,
        source_object_type: r.source_object_type,
        source_external_id: r.source_external_id,
        sip_entity_type: r.sip_entity_type,
        sip_entity_id: r.sip_entity_id,
        created_at: r.created_at,
    }
}

#[derive(FromRow)]
struct MigrationValidationIssueRow {
    id: Uuid,
    organization_id: Uuid,
    job_id: Uuid,
    staged_record_id: Uuid,
    severity: String,
    field: String,
    message: String,
    created_at: chrono::DateTime<Utc>,
}

fn map_migration_validation_issue_row(r: MigrationValidationIssueRow) -> MigrationValidationIssue {
    MigrationValidationIssue {
        id: MigrationValidationIssueId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        job_id: MigrationJobId::from(r.job_id),
        staged_record_id: MigrationStagedRecordId::from(r.staged_record_id),
        severity: r.severity,
        field: r.field,
        message: r.message,
        created_at: r.created_at,
    }
}

#[derive(FromRow)]
struct MigrationDuplicateCandidateRow {
    id: Uuid,
    organization_id: Uuid,
    job_id: Uuid,
    staged_record_id: Uuid,
    sip_entity_type: String,
    sip_entity_id: Uuid,
    confidence_score: f64,
    match_reason: String,
    status: String,
    created_at: chrono::DateTime<Utc>,
}

fn map_migration_duplicate_candidate_row(
    r: MigrationDuplicateCandidateRow,
) -> MigrationDuplicateCandidate {
    MigrationDuplicateCandidate {
        id: MigrationDuplicateCandidateId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        job_id: MigrationJobId::from(r.job_id),
        staged_record_id: MigrationStagedRecordId::from(r.staged_record_id),
        sip_entity_type: r.sip_entity_type,
        sip_entity_id: r.sip_entity_id,
        confidence_score: r.confidence_score,
        match_reason: r.match_reason,
        status: r.status,
        created_at: r.created_at,
    }
}

#[derive(FromRow)]
struct MigrationCheckpointRow {
    id: Uuid,
    organization_id: Uuid,
    job_id: Uuid,
    run_id: Uuid,
    checkpoint_type: String,
    state_data: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
}

fn map_migration_checkpoint_row(r: MigrationCheckpointRow) -> MigrationCheckpoint {
    MigrationCheckpoint {
        id: MigrationCheckpointId::from(r.id),
        organization_id: OrganizationId::from(r.organization_id),
        job_id: MigrationJobId::from(r.job_id),
        run_id: MigrationRunId::from(r.run_id),
        checkpoint_type: r.checkpoint_type,
        state_data: r.state_data,
        created_at: r.created_at,
    }
}

#[cfg(test)]
mod migration_repo_tests {
    use super::*;

    #[test]
    fn test_map_migration_job_row() {
        let row = MigrationJobRow {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "Test Job".into(),
            description: Some("Test description".into()),
            source_system: "Maximo".into(),
            source_object_type: "asset".into(),
            status: "DRAFT".into(),
            source_record_count: 100,
            valid_record_count: 90,
            imported_record_count: 0,
            error_count: 0,
            created_by: Uuid::new_v4(),
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let job = map_migration_job_row(row);
        assert_eq!(job.name, "Test Job");
        assert_eq!(job.status, MigrationJobStatus::Draft);
        assert_eq!(job.source_record_count, 100);
    }
}
