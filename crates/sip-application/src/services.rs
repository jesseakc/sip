use chrono::{Datelike, Utc};
use serde::Serialize;
use sip_ai::{CompletionOptions, LlmProvider};
use sip_auth::{decode_jwt, encode_jwt, verify_password, Claims};
use sip_domain::{
    entity::activity::{Activity, ActivitySource},
    entity::ai_conversation::{AIConversation, AIMessage, AIRetrievalTrace, RetrieverType, Source},
    entity::asset::{Asset, AssetStatus},
    entity::asset_model::{AssetModel, LifecycleStatus},
    entity::asset_type::AssetType,
    entity::document::{Document, DocumentSourceType, DocumentType, ProcessingStatus, Visibility},
    entity::inspection::{ChecklistResult, Inspection, InspectionChecklistItem},
    entity::location::{Location, LocationType},
    entity::manufacturer::Manufacturer,
    entity::migration::{
        MigrationDuplicateCandidate, MigrationExternalIdMap,
        MigrationFieldMapping, MigrationImportResult, MigrationJob, MigrationJobStatus,
        MigrationRun, MigrationRunStatus, MigrationRunType, MigrationSourceRecord,
        MigrationStagedRecord, MigrationValidationIssue,
    },
    entity::organization::Organization,
    entity::part::Part,
    entity::part::PartUsage,
    entity::schedule::{Schedule, ScheduleTriggerType},
    entity::team::Team,
    entity::user::{User, UserRole},
    entity::work_order::{
        ActorType, WorkOrder, WorkOrderAssignment, WorkOrderPriority, WorkOrderSourceType,
        WorkOrderStatus, WorkOrderStatusHistory, WorkOrderType,
    },
    error::SipError,
    id::{
        AIConversationId, AIMessageId, AIRetrievalTraceId, ActivityId, AgentIdentityId, AssetId,
        AssetModelId, AssetTypeId, DocumentId, InspectionChecklistItemId, InspectionId, LocationId,
        ManufacturerId,
        MigrationExternalIdMapId, MigrationImportResultId, MigrationJobId,
        MigrationRunId, MigrationSourceRecordId, MigrationStagedRecordId,
        MigrationValidationIssueId, OrganizationId, PartId, ScheduleId, TeamId, UserId,
        WorkOrderAssignmentId, WorkOrderId, WorkOrderStatusHistoryId,
    },
    repository::{
        AIConversationRepository, ActivityRepository, AssetRepository, DocumentRepository,
        InspectionRepository, PartRepository, PartUsageRepository, ScheduleRepository,
        WorkOrderAssignmentRepository, WorkOrderStatusHistoryRepository,
    },
    tenant::TenantContext,
};
use sip_infrastructure::repositories::{
    PgAIConversationRepository, PgActivityRepository, PgAssetModelRepository,
    PgAssetTypeRepository, PgLocationRepository, PgManufacturerRepository, PgMigrationRepository,
    PgOrganizationRepository, PgTeamRepository, PgUserRepository,
};
use sip_tenancy::set_rls_org_pool;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuthService {
    user_repo: sip_infrastructure::repositories::PgUserRepository,
    jwt_secret: String,
    jwt_expiration: u64,
    refresh_expiration: u64,
}

impl AuthService {
    pub fn new(
        pool: PgPool,
        jwt_secret: String,
        jwt_expiration: u64,
        refresh_expiration: u64,
    ) -> Self {
        Self {
            user_repo: sip_infrastructure::repositories::PgUserRepository::new(pool),
            jwt_secret,
            jwt_expiration,
            refresh_expiration,
        }
    }

    pub async fn login(&self, email: &str, password: &str) -> Result<(String, String), SipError> {
        let (user, pass_hash) = self
            .user_repo
            .get_by_email(email)
            .await?
            .ok_or(SipError::PermissionDenied)?;
        if !verify_password(password, &pass_hash)
            .map_err(|e| SipError::Validation(e.to_string()))?
        {
            return Err(SipError::PermissionDenied);
        }
        let permissions = role_permissions(&user.role);
        let claims = Claims {
            sub: user.id.to_string(),
            org_id: user.organization_id.to_string(),
            role: format!("{:?}", user.role).to_uppercase(),
            permissions,
            exp: (chrono::Utc::now().timestamp() as u64 + self.jwt_expiration) as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };
        let token = encode_jwt(&claims, &self.jwt_secret)
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let refresh = encode_jwt(
            &Claims {
                sub: user.id.to_string(),
                org_id: user.organization_id.to_string(),
                role: format!("{:?}", user.role).to_uppercase(),
                permissions: vec![],
                exp: (chrono::Utc::now().timestamp() as u64 + self.refresh_expiration) as usize,
                iat: chrono::Utc::now().timestamp() as usize,
            },
            &self.jwt_secret,
        )
        .map_err(|e| SipError::Validation(e.to_string()))?;
        Ok((token, refresh))
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<String, SipError> {
        let claims = decode_jwt(refresh_token, &self.jwt_secret)
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let user_id =
            UserId::from_str(&claims.sub).map_err(|e| SipError::Validation(e.to_string()))?;
        let org_id = OrganizationId::from_str(&claims.org_id)
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let ctx = TenantContext::new(org_id, Some(user_id));
        let user = self
            .user_repo
            .get(&ctx, user_id)
            .await?
            .ok_or(SipError::PermissionDenied)?;
        let permissions = role_permissions(&user.role);
        let new_claims = Claims {
            sub: user.id.to_string(),
            org_id: user.organization_id.to_string(),
            role: format!("{:?}", user.role).to_uppercase(),
            permissions,
            exp: (chrono::Utc::now().timestamp() as u64 + self.jwt_expiration) as usize,
            iat: chrono::Utc::now().timestamp() as usize,
        };
        encode_jwt(&new_claims, &self.jwt_secret).map_err(|e| SipError::Validation(e.to_string()))
    }

    pub async fn me(&self, token: &str) -> Result<User, SipError> {
        let claims =
            decode_jwt(token, &self.jwt_secret).map_err(|e| SipError::Validation(e.to_string()))?;
        let user_id =
            UserId::from_str(&claims.sub).map_err(|e| SipError::Validation(e.to_string()))?;
        let org_id = OrganizationId::from_str(&claims.org_id)
            .map_err(|e| SipError::Validation(e.to_string()))?;
        let ctx = TenantContext::new(org_id, Some(user_id)).with_permissions(claims.permissions);
        self.user_repo
            .get(&ctx, user_id)
            .await?
            .ok_or(SipError::PermissionDenied)
    }
}

fn role_permissions(role: &UserRole) -> Vec<String> {
    match role {
        UserRole::Admin => vec!["*".to_string()],
        UserRole::Manager => vec![
            "org:read",
            "org:manage",
            "asset:create",
            "asset:update",
            "asset:read",
            "asset:archive",
            "work_order:create",
            "work_order:update",
            "work_order:read",
            "work_order:cancel",
            "work_order:assign",
            "work_order:review",
            "schedule:manage",
            "schedule:read",
            "user:read",
            "user:manage",
            "team:manage",
            "part:manage",
            "part:read",
            "document:upload",
            "document:read",
            "document:archive",
            "activity:read",
            "ai:query",
            "migration:create",
            "migration:read",
            "migration:map",
            "migration:validate",
            "migration:dry_run",
            "migration:execute",
            "migration:rollback",
            "migration:delete",
            "migration:view_raw_source",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
        UserRole::Technician => vec![
            "org:read",
            "asset:read",
            "work_order:create",
            "work_order:update",
            "work_order:read",
            "schedule:read",
            "user:read",
            "part:consume",
            "part:read",
            "document:upload",
            "document:read",
            "ai:query",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
        UserRole::Viewer => vec![
            "org:read",
            "asset:read",
            "work_order:read",
            "schedule:read",
            "user:read",
            "part:read",
            "document:read",
            "ai:query",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
        UserRole::Vendor => vec![
            "asset:read",
            "work_order:read",
            "part:read",
            "document:read",
            "ai:query",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
        UserRole::Auditor => vec![
            "org:read",
            "asset:read",
            "work_order:read",
            "schedule:read",
            "user:read",
            "part:read",
            "document:read",
            "activity:read",
            "ai:query",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
    }
}

pub struct AssetService<R: sip_domain::repository::AssetRepository> {
    repo: R,
}

impl<R: sip_domain::repository::AssetRepository> AssetService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        input: CreateAssetInput,
    ) -> Result<Asset, SipError> {
        ctx.require_permission("asset:create")?;
        let asset = Asset {
            id: AssetId::new(),
            organization_id: ctx.organization_id,
            location_id: input.location_id,
            parent_id: None,
            asset_type_id: input.asset_type_id,
            model_id: input.model_id,
            name: input.name,
            description: input.description,
            serial_number: input.serial_number,
            firmware_version: None,
            software_version: None,
            hardware_revision: None,
            status: AssetStatus::Operational,
            version: 1,
            criticality: input.criticality,
            installed_date: None,
            warranty_expiry: None,
            attributes: input.attributes,
            tags: input.tags.unwrap_or_default(),
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create_asset(ctx, &asset).await
    }

    pub async fn get(&self, ctx: &TenantContext, id: AssetId) -> Result<Option<Asset>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.get_asset(ctx, id).await
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Asset>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.list_assets(ctx).await
    }

    pub async fn update_status(
        &self,
        ctx: &TenantContext,
        id: AssetId,
        status: AssetStatus,
    ) -> Result<Asset, SipError> {
        ctx.require_permission("asset:update")?;
        let current = self
            .repo
            .get_asset(ctx, id)
            .await?
            .ok_or(SipError::Validation("Asset not found".into()))?;
        if !current.status.can_transition_to(status) {
            return Err(SipError::invalid_state_transition_asset(
                current.status,
                status,
            ));
        }
        let patch = serde_json::json!({"status": format!("{:?}", status).to_uppercase()});
        self.repo.update_asset(ctx, id, 1, patch).await
    }

    pub async fn archive(&self, ctx: &TenantContext, id: AssetId) -> Result<Asset, SipError> {
        ctx.require_permission("asset:archive")?;
        self.repo.archive_asset(ctx, id).await
    }

    pub async fn list_children(
        &self,
        ctx: &TenantContext,
        parent_id: AssetId,
    ) -> Result<Vec<Asset>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.list_children(ctx, parent_id).await
    }
}

pub struct CreateAssetInput {
    pub asset_type_id: sip_domain::id::AssetTypeId,
    pub model_id: Option<sip_domain::id::AssetModelId>,
    pub location_id: Option<LocationId>,
    pub name: String,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub criticality: sip_domain::entity::asset::Criticality,
    pub attributes: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
}

pub struct WorkOrderService<R: sip_domain::repository::WorkOrderRepository> {
    repo: R,
}

impl<R: sip_domain::repository::WorkOrderRepository> WorkOrderService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    async fn generate_display_number(&self, ctx: &TenantContext) -> Result<String, SipError> {
        let year = Utc::now().year();
        let max = self.repo.get_max_display_number_for_year(ctx, year).await?;
        let next_num = match max {
            Some(num) => {
                let suffix = num.split('-').next_back().unwrap_or("0");
                suffix.parse::<i32>().unwrap_or(0) + 1
            }
            None => 1,
        };
        Ok(format!("WO-{}-{:05}", year, next_num))
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        input: CreateWorkOrderInput,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:create")?;
        let wo = WorkOrder {
            id: WorkOrderId::new(),
            organization_id: ctx.organization_id,
            asset_id: input.asset_id,
            parent_id: None,
            schedule_id: None,
            work_order_type: input.work_order_type,
            priority: input.priority,
            status: WorkOrderStatus::Draft,
            title: input.title,
            display_number: self.generate_display_number(ctx).await?,
            description: input.description,
            scheduled_start: input.scheduled_start,
            scheduled_end: input.scheduled_end,
            actual_start: None,
            actual_end: None,
            due_at: input.due_at,
            estimated_hours: input.estimated_hours,
            actual_hours: None,
            resolution_notes: None,
            failure_code: None,
            root_cause: None,
            created_by_id: ctx.user_id.ok_or(SipError::PermissionDenied)?,
            source_type: WorkOrderSourceType::Manual,
            source_system: None,
            external_id: None,
            external_url: None,
            reopened_count: 0,
            last_reopened_at: None,
            last_reopened_by_id: None,
            version: 1,
            archived_at: None,
            archived_by_id: None,
            archive_reason: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create_work_order(ctx, &wo).await
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
    ) -> Result<Option<WorkOrder>, SipError> {
        ctx.require_permission("work_order:read")?;
        self.repo.get_work_order(ctx, id).await
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<WorkOrder>, SipError> {
        ctx.require_permission("work_order:read")?;
        self.repo.list_work_orders(ctx).await
    }

    pub async fn transition(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        new_status: WorkOrderStatus,
        resolution_notes: Option<String>,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:update")?;
        let current = self
            .repo
            .get_work_order(ctx, id)
            .await?
            .ok_or(SipError::Validation("Work order not found".into()))?;
        if !current.status.can_transition_to(new_status) {
            return Err(SipError::invalid_state_transition_work_order(
                current.status,
                new_status,
            ));
        }
        let mut patch = serde_json::json!({"status": format!("{:?}", new_status).to_uppercase()});
        if let Some(notes) = resolution_notes {
            patch["resolution_notes"] = serde_json::json!(notes);
        }
        self.repo
            .update_work_order(ctx, id, current.version, patch)
            .await
    }

    pub async fn list_by_asset(
        &self,
        ctx: &TenantContext,
        asset_id: AssetId,
    ) -> Result<Vec<WorkOrder>, SipError> {
        ctx.require_permission("work_order:read")?;
        self.repo.list_work_orders_by_asset(ctx, asset_id).await
    }

    async fn do_lifecycle_transition<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        new_status: WorkOrderStatus,
        resolution_notes: Option<String>,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        let current = self
            .repo
            .get_work_order(ctx, id)
            .await?
            .ok_or(SipError::Validation("Work order not found".into()))?;
        let old_status = current.status;
        if !old_status.can_transition_to(new_status) {
            return Err(SipError::invalid_state_transition_work_order(
                old_status, new_status,
            ));
        }
        let mut patch = serde_json::json!({"status": format!("{:?}", new_status).to_uppercase()});
        if let Some(ref notes) = resolution_notes {
            patch["resolution_notes"] = serde_json::json!(notes);
        }
        let wo = self
            .repo
            .update_work_order(ctx, id, current.version, patch)
            .await?;

        let _ = activity_repo.create_activity(ctx, &Activity {
            id: ActivityId::new(),
            organization_id: ctx.organization_id,
            actor_id: ctx.user_id,
            actor_type: ActorType::Human,
            agent_identity_id: None,
            plugin_id: None,
            entity_type: "work_order".to_string(),
            entity_id: wo.id.into(),
            action: "work_order.status_changed".to_string(),
            changes: None,
            request_id: None,
            correlation_id: None,
            source: ActivitySource::Api,
            reason: None,
            metadata: Some(serde_json::json!({"from": format!("{:?}", old_status), "to": format!("{:?}", new_status)})),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        }).await;

        let _ = history_repo
            .create_status_history(
                ctx,
                &WorkOrderStatusHistory {
                    id: WorkOrderStatusHistoryId::new(),
                    organization_id: ctx.organization_id,
                    work_order_id: wo.id,
                    from_status: Some(old_status),
                    to_status: new_status,
                    changed_by_id: ctx.user_id,
                    actor_type: ActorType::Human,
                    agent_identity_id: None,
                    plugin_id: None,
                    reason: resolution_notes,
                    created_at: Utc::now(),
                },
            )
            .await;

        Ok(wo)
    }

    pub async fn publish<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:update")?;
        self.do_lifecycle_transition(
            ctx,
            id,
            WorkOrderStatus::Open,
            None,
            activity_repo,
            history_repo,
        )
        .await
    }

    pub async fn start<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:update")?;
        self.do_lifecycle_transition(
            ctx,
            id,
            WorkOrderStatus::InProgress,
            None,
            activity_repo,
            history_repo,
        )
        .await
    }

    pub async fn hold<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:update")?;
        self.do_lifecycle_transition(
            ctx,
            id,
            WorkOrderStatus::OnHold,
            None,
            activity_repo,
            history_repo,
        )
        .await
    }

    pub async fn resume<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:update")?;
        self.do_lifecycle_transition(
            ctx,
            id,
            WorkOrderStatus::InProgress,
            None,
            activity_repo,
            history_repo,
        )
        .await
    }

    pub async fn complete<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        resolution_notes: Option<String>,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:update")?;
        self.do_lifecycle_transition(
            ctx,
            id,
            WorkOrderStatus::Completed,
            resolution_notes,
            activity_repo,
            history_repo,
        )
        .await
    }

    pub async fn review<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:review")?;
        self.do_lifecycle_transition(
            ctx,
            id,
            WorkOrderStatus::Reviewed,
            None,
            activity_repo,
            history_repo,
        )
        .await
    }

    pub async fn close<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:update")?;
        self.do_lifecycle_transition(
            ctx,
            id,
            WorkOrderStatus::Closed,
            None,
            activity_repo,
            history_repo,
        )
        .await
    }

    pub async fn cancel<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:cancel")?;
        self.do_lifecycle_transition(
            ctx,
            id,
            WorkOrderStatus::Cancelled,
            None,
            activity_repo,
            history_repo,
        )
        .await
    }

    pub async fn archive<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        activity_repo: &A,
        history_repo: &H,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:update")?;
        let current = self
            .repo
            .get_work_order(ctx, id)
            .await?
            .ok_or(SipError::Validation("Work order not found".into()))?;
        let old_status = current.status;
        let archived_by_id = ctx.user_id.ok_or(SipError::PermissionDenied)?;
        let wo = self
            .repo
            .archive_work_order(ctx, id, archived_by_id)
            .await?;

        let _ = activity_repo
            .create_activity(
                ctx,
                &Activity {
                    id: ActivityId::new(),
                    organization_id: ctx.organization_id,
                    actor_id: ctx.user_id,
                    actor_type: ActorType::Human,
                    agent_identity_id: None,
                    plugin_id: None,
                    entity_type: "work_order".to_string(),
                    entity_id: wo.id.into(),
                    action: "work_order.archived".to_string(),
                    changes: None,
                    request_id: None,
                    correlation_id: None,
                    source: ActivitySource::Api,
                    reason: None,
                    metadata: Some(
                        serde_json::json!({"from": format!("{:?}", old_status), "to": "ARCHIVED"}),
                    ),
                    ip_address: None,
                    user_agent: None,
                    created_at: Utc::now(),
                },
            )
            .await;

        let _ = history_repo
            .create_status_history(
                ctx,
                &WorkOrderStatusHistory {
                    id: WorkOrderStatusHistoryId::new(),
                    organization_id: ctx.organization_id,
                    work_order_id: wo.id,
                    from_status: Some(old_status),
                    to_status: WorkOrderStatus::Closed,
                    changed_by_id: ctx.user_id,
                    actor_type: ActorType::Human,
                    agent_identity_id: None,
                    plugin_id: None,
                    reason: None,
                    created_at: Utc::now(),
                },
            )
            .await;

        Ok(wo)
    }

    pub async fn reopen<A: ActivityRepository, H: WorkOrderStatusHistoryRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
        activity_repo: &A,
        history_repo: &H,
        resume_in_progress: bool,
    ) -> Result<WorkOrder, SipError> {
        ctx.require_permission("work_order:update")?;

        let current = self
            .repo
            .get_work_order(ctx, id)
            .await?
            .ok_or(SipError::Validation("Work order not found".into()))?;

        if current.status != WorkOrderStatus::Closed {
            return Err(SipError::Validation(
                "Only CLOSED work orders can be reopened".into(),
            ));
        }

        let new_status = if resume_in_progress {
            WorkOrderStatus::InProgress
        } else {
            WorkOrderStatus::Open
        };

        let patch = serde_json::json!({
            "status": format!("{:?}", new_status).to_uppercase(),
            "reopened_count": current.reopened_count + 1,
            "last_reopened_at": Utc::now().to_rfc3339(),
            "last_reopened_by_id": ctx.user_id.map(|id| id.to_string()).unwrap_or_default(),
        });

        let wo = self
            .repo
            .update_work_order(ctx, id, current.version, patch)
            .await?;

        let _ = activity_repo.create_activity(ctx, &Activity {
            id: ActivityId::new(),
            organization_id: ctx.organization_id,
            actor_id: ctx.user_id,
            actor_type: ActorType::Human,
            agent_identity_id: None,
            plugin_id: None,
            entity_type: "work_order".to_string(),
            entity_id: id.into(),
            action: "work_order.reopened".to_string(),
            changes: None,
            request_id: None,
            correlation_id: None,
            source: ActivitySource::Api,
            reason: None,
            metadata: Some(serde_json::json!({"from_status": format!("{:?}", current.status), "to_status": format!("{:?}", new_status), "reopened_count": current.reopened_count + 1})),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        }).await;

        let _ = history_repo
            .create_status_history(
                ctx,
                &WorkOrderStatusHistory {
                    id: WorkOrderStatusHistoryId::new(),
                    organization_id: ctx.organization_id,
                    work_order_id: id,
                    from_status: Some(current.status),
                    to_status: new_status,
                    changed_by_id: ctx.user_id,
                    actor_type: ActorType::Human,
                    agent_identity_id: None,
                    plugin_id: None,
                    reason: Some("Work order reopened".to_string()),
                    created_at: Utc::now(),
                },
            )
            .await;

        Ok(wo)
    }

    pub async fn list_assignments<A: WorkOrderAssignmentRepository>(
        &self,
        ctx: &TenantContext,
        work_order_id: WorkOrderId,
        assignment_repo: &A,
    ) -> Result<Vec<WorkOrderAssignment>, SipError> {
        ctx.require_permission("work_order:read")?;
        assignment_repo
            .list_assignments_by_work_order(ctx, work_order_id)
            .await
    }

    pub async fn create_assignment<A: WorkOrderAssignmentRepository>(
        &self,
        ctx: &TenantContext,
        assignment: WorkOrderAssignment,
        assignment_repo: &A,
    ) -> Result<WorkOrderAssignment, SipError> {
        ctx.require_permission("work_order:assign")?;
        assignment_repo.create_assignment(ctx, &assignment).await
    }

    pub async fn delete_assignment<A: WorkOrderAssignmentRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderAssignmentId,
        assignment_repo: &A,
    ) -> Result<(), SipError> {
        ctx.require_permission("work_order:assign")?;
        assignment_repo.delete_assignment(ctx, id).await
    }

    pub async fn list_parts<P: PartUsageRepository>(
        &self,
        ctx: &TenantContext,
        work_order_id: WorkOrderId,
        part_repo: &P,
    ) -> Result<Vec<PartUsage>, SipError> {
        ctx.require_permission("part:read")?;
        part_repo
            .list_part_usage_by_work_order(ctx, work_order_id)
            .await
    }

    pub async fn add_part<P: PartUsageRepository>(
        &self,
        ctx: &TenantContext,
        usage: PartUsage,
        part_repo: &P,
    ) -> Result<PartUsage, SipError> {
        ctx.require_permission("part:consume")?;
        part_repo.create_part_usage(ctx, &usage).await
    }

    pub async fn update_assignment<A: WorkOrderAssignmentRepository>(
        &self,
        ctx: &TenantContext,
        id: WorkOrderAssignmentId,
        status: sip_domain::entity::work_order::AssignmentStatus,
        assignment_repo: &A,
    ) -> Result<WorkOrderAssignment, SipError> {
        ctx.require_permission("work_order:assign")?;
        assignment_repo.update_assignment(ctx, id, status).await
    }
}

pub struct CreateWorkOrderInput {
    pub asset_id: AssetId,
    pub work_order_type: WorkOrderType,
    pub priority: WorkOrderPriority,
    pub title: String,
    pub description: String,
    pub scheduled_start: Option<chrono::DateTime<Utc>>,
    pub scheduled_end: Option<chrono::DateTime<Utc>>,
    pub due_at: Option<chrono::DateTime<Utc>>,
    pub estimated_hours: Option<rust_decimal::Decimal>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum AIStreamEvent {
    #[serde(rename = "context")]
    Context {
        records_queried: i32,
        paths: Vec<String>,
    },
    #[serde(rename = "chunk")]
    Chunk { text: String },
    #[serde(rename = "verification")]
    Verification {
        checks_passed: i32,
        checks_total: i32,
        status: String,
    },
    #[serde(rename = "done")]
    Done {
        sources: Vec<Source>,
        verification_status: String,
        retrieval_paths: Vec<String>,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

#[allow(dead_code)]
struct ContextResult {
    context_text: String,
    records_queried: usize,
    sources: Vec<Source>,
    retrieval_paths: Vec<String>,
}

struct FetchResult {
    parts: Vec<String>,
    records: usize,
    sources: Vec<Source>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
struct SearchResultRow {
    source_type: String,
    source_id: Uuid,
    collection: String,
    content: String,
    similarity: f64,
    organization_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub source_type: String,
    pub source_id: Uuid,
    pub collection: String,
    pub excerpt: String,
    pub score: f64,
    pub source_scope: String,
    pub verified_by_sql: bool,
}

pub struct DocumentService<R: DocumentRepository> {
    repo: R,
}

impl<R: DocumentRepository> DocumentService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        input: CreateDocumentInput,
    ) -> Result<Document, SipError> {
        ctx.require_permission("document:upload")?;
        let doc = Document {
            id: DocumentId::new(),
            organization_id: ctx.organization_id,
            name: input.name,
            document_type: input.document_type,
            mime_type: input.mime_type,
            size_bytes: input.size_bytes,
            document_version: input.document_version,
            checksum: input.checksum,
            source_type: input.source_type,
            source_system: input.source_system,
            external_id: input.external_id,
            external_url: input.external_url,
            storage_path: input.storage_path,
            visibility: input.visibility,
            processing_status: ProcessingStatus::Pending,
            processing_error: None,
            extracted_text_path: None,
            text_content: None,
            effective_date: input.effective_date,
            expiration_date: input.expiration_date,
            supersedes_document_id: input.supersedes_document_id,
            version: 1,
            archived_at: None,
            archived_by_id: None,
            archive_reason: None,
            metadata: input.metadata,
            uploaded_by_id: ctx.user_id.ok_or(SipError::PermissionDenied)?,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create_document(ctx, &doc).await
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: DocumentId,
    ) -> Result<Option<Document>, SipError> {
        ctx.require_permission("document:read")?;
        self.repo.get_document(ctx, id).await
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Document>, SipError> {
        ctx.require_permission("document:read")?;
        self.repo.list_documents(ctx).await
    }

    pub async fn archive(
        &self,
        ctx: &TenantContext,
        id: DocumentId,
        reason: &str,
    ) -> Result<Document, SipError> {
        ctx.require_permission("document:archive")?;
        let archived_by_id = ctx.user_id.ok_or(SipError::PermissionDenied)?;
        self.repo.archive(ctx, id, archived_by_id, reason).await
    }
}

pub struct CreateDocumentInput {
    pub name: String,
    pub document_type: DocumentType,
    pub mime_type: String,
    pub size_bytes: i64,
    pub document_version: Option<String>,
    pub checksum: String,
    pub source_type: DocumentSourceType,
    pub source_system: Option<String>,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
    pub storage_path: String,
    pub visibility: Visibility,
    pub effective_date: Option<chrono::NaiveDate>,
    pub expiration_date: Option<chrono::NaiveDate>,
    pub supersedes_document_id: Option<DocumentId>,
    pub metadata: Option<serde_json::Value>,
}

pub struct ActivityService<R: ActivityRepository> {
    repo: R,
}

impl<R: ActivityRepository> ActivityService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Activity>, SipError> {
        ctx.require_permission("activity:read")?;
        self.repo.list_activities(ctx).await
    }

    pub async fn list_by_entity(
        &self,
        ctx: &TenantContext,
        entity_type: &str,
        entity_id: uuid::Uuid,
    ) -> Result<Vec<Activity>, SipError> {
        ctx.require_permission("activity:read")?;
        self.repo
            .list_activities_by_entity(ctx, entity_type, entity_id)
            .await
    }
}

#[derive(Debug)]
struct ClassifiedQuery {
    question_type: QuestionType,
    #[allow(dead_code)]
    entities: Vec<EntityReference>,
    #[allow(dead_code)]
    time_filter: Option<String>,
}

#[derive(Debug)]
enum QuestionType {
    ExactFact,
    Similarity,
    Relationship,
    Temporal,
    DocumentQA,
    RootCause,
    GeneralHybrid,
}

#[derive(Debug)]
struct EntityReference {
    #[allow(dead_code)]
    entity_type: String,
    #[allow(dead_code)]
    name_hint: String,
}

pub struct AIService {
    provider: Arc<dyn LlmProvider>,
    pool: PgPool,
}

impl AIService {
    pub fn new(provider: Box<dyn LlmProvider>, pool: PgPool) -> Self {
        Self {
            provider: Arc::from(provider),
            pool,
        }
    }

    pub async fn chat_stream(
        &self,
        ctx: &TenantContext,
        message: &str,
        conversation_id: Option<String>,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<AIStreamEvent>, SipError> {
        ctx.require_permission("ai:query")?;
        let ctx = ctx.clone();
        let message = message.to_string();
        let pool = self.pool.clone();
        let provider = self.provider.clone();

        let (conversation_id, context, records_queried, retrieval_paths) =
            self.setup_chat(&ctx, &message, conversation_id).await?;

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            if let Err(e) = Self::run_chat_stream(
                &ctx,
                &message,
                conversation_id,
                context,
                records_queried,
                retrieval_paths,
                provider,
                &pool,
                tx.clone(),
            )
            .await
            {
                let _ = tx.send(AIStreamEvent::Error {
                    message: e.to_string(),
                });
            }
        });

        Ok(rx)
    }

    async fn setup_chat(
        &self,
        ctx: &TenantContext,
        message: &str,
        conversation_id: Option<String>,
    ) -> Result<(AIConversationId, String, usize, Vec<String>), SipError> {
        let ai_repo = PgAIConversationRepository::new(self.pool.clone());

        let conversation_id = match conversation_id {
            Some(id) => {
                AIConversationId::from_str(&id).map_err(|e| SipError::Validation(e.to_string()))?
            }
            None => {
                let user_id = ctx.user_id.ok_or(SipError::PermissionDenied)?;
                let conversation = AIConversation {
                    id: AIConversationId::new(),
                    organization_id: ctx.organization_id,
                    user_id,
                    title: Some(message.chars().take(50).collect()),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                ai_repo.create_conversation(ctx, &conversation).await?;
                conversation.id
            }
        };

        let user_message = AIMessage {
            id: AIMessageId::new(),
            conversation_id,
            role: "user".to_string(),
            content: message.to_string(),
            sources: None,
            created_at: Utc::now(),
        };
        ai_repo.create_message(ctx, &user_message).await?;

        let context = self.build_context(ctx, message).await?;
        let records_queried = context.records_queried;

        Ok((
            conversation_id,
            context.context_text,
            records_queried,
            context.retrieval_paths,
        ))
    }

    fn classify_query(query: &str) -> ClassifiedQuery {
        let q = query.to_lowercase();

        let question_type = if q.contains("serial")
            || q.contains("status of")
            || q.contains("assigned to")
            || q.contains("model number")
            || q.contains("firmware")
            || q.contains("who owns")
        {
            QuestionType::ExactFact
        } else if q.contains("similar")
            || q.contains("pattern")
            || q.contains("symptom")
            || q.contains("seen before")
            || q.contains("like this")
            || q.contains("same issue")
        {
            QuestionType::Similarity
        } else if q.contains("depends on")
            || q.contains("connected to")
            || q.contains("uses same")
            || q.contains("powered by")
            || q.contains("controlled by")
            || q.contains("related")
        {
            QuestionType::Relationship
        } else if q.contains("what changed")
            || q.contains("since")
            || q.contains("last time")
            || q.contains("history of")
            || q.contains("over time")
            || q.contains("trend")
        {
            QuestionType::Temporal
        } else if q.contains("manual")
            || q.contains("procedure")
            || q.contains("instruction")
            || q.contains("how to")
            || q.contains("spec sheet")
            || q.contains("documentation")
        {
            QuestionType::DocumentQA
        } else if q.contains("why")
            || q.contains("root cause")
            || q.contains("keeps failing")
            || q.contains("cause of")
            || q.contains("reason for")
        {
            QuestionType::RootCause
        } else {
            QuestionType::GeneralHybrid
        };

        ClassifiedQuery {
            question_type,
            entities: vec![],
            time_filter: None,
        }
    }

    async fn build_context(
        &self,
        ctx: &TenantContext,
        query: &str,
    ) -> Result<ContextResult, SipError> {
        set_rls_org_pool(&self.pool, ctx.organization_id)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;

        let classified = Self::classify_query(query);
        let mut context_parts: Vec<String> = Vec::new();
        let mut sources: Vec<Source> = Vec::new();
        let mut paths: Vec<String> = Vec::new();
        let mut records_queried: usize = 0;

        match classified.question_type {
            QuestionType::ExactFact => {
                paths.push("SQL".into());
                let facts = self.fetch_exact_facts(ctx, query).await?;
                context_parts.extend(facts.parts);
                sources.extend(facts.sources);
                records_queried += facts.records;
            }
            QuestionType::Similarity => {
                paths.push("VECTOR".into());
                paths.push("SQL".into());
                let similar = self.fetch_similar(ctx, query).await?;
                context_parts.extend(similar.parts);
                sources.extend(similar.sources);
                records_queried += similar.records;
            }
            QuestionType::Relationship => {
                paths.push("GRAPH".into());
                paths.push("SQL".into());
                let related = self.fetch_related(ctx, query).await?;
                context_parts.extend(related.parts);
                sources.extend(related.sources);
                records_queried += related.records;
            }
            QuestionType::Temporal => {
                paths.push("TEMPORAL".into());
                paths.push("SQL".into());
                let temporal = self.fetch_temporal(ctx, query).await?;
                context_parts.extend(temporal.parts);
                sources.extend(temporal.sources);
                records_queried += temporal.records;
            }
            QuestionType::DocumentQA => {
                paths.push("VECTOR".into());
                paths.push("RAG".into());
                let docs = self.fetch_documents(ctx, query).await?;
                context_parts.extend(docs.parts);
                sources.extend(docs.sources);
                records_queried += docs.records;
            }
            QuestionType::RootCause | QuestionType::GeneralHybrid => {
                paths.push("HYBRID".into());
                let relational = self.fetch_recent_work_orders(ctx).await?;
                context_parts.extend(relational.parts);
                sources.extend(relational.sources);
                records_queried += relational.records;
                let similar = self.fetch_similar(ctx, query).await?;
                context_parts.extend(similar.parts);
                sources.extend(similar.sources);
                records_queried += similar.records;
            }
        }

        let context_text = if context_parts.is_empty() {
            "No relevant context found.".to_string()
        } else {
            context_parts.join("\n")
        };

        Ok(ContextResult {
            context_text,
            records_queried,
            sources,
            retrieval_paths: paths,
        })
    }

    async fn fetch_exact_facts(
        &self,
        ctx: &TenantContext,
        query: &str,
    ) -> Result<FetchResult, SipError> {
        let message_lower = query.to_lowercase();

        let mut parts = Vec::new();
        let mut records: usize = 0;
        let mut sources = Vec::new();

        // Search assets by name/serial
        let asset_rows = sqlx::query_as::<_, (Uuid, String, String, Option<String>, Option<String>)>(
            "SELECT id, name, status, criticality, serial_number FROM assets WHERE organization_id = $1 LIMIT 200"
        )
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        let matching_assets: Vec<_> = asset_rows
            .iter()
            .filter(|(_, name, _, _, sn)| {
                message_lower.contains(&name.to_lowercase())
                    || sn
                        .as_deref()
                        .map(|s| message_lower.contains(&s.to_lowercase()))
                        .unwrap_or(false)
            })
            .collect();

        for (id, name, status, criticality, sn) in &matching_assets {
            parts.push(format!(
                "Asset: {} (Status: {}, Criticality: {}, Serial: {})",
                name,
                status,
                criticality.as_deref().unwrap_or("N/A"),
                sn.as_deref().unwrap_or("N/A")
            ));
            records += 1;
            sources.push(Source {
                id: id.to_string(),
                source_type: "asset".to_string(),
                title: name.clone(),
                relevance: 1.0,
                field: Some("name".into()),
                source_scope: Some("PRIVATE_TENANT".into()),
                retrieval_type: Some("SQL".into()),
                verified_by_sql: Some(true),
                quote: None,
                timestamp: None,
            });

            // Fetch work orders for this asset
            let wos =
                sqlx::query_as::<_, (Uuid, String, String, String, Option<chrono::DateTime<Utc>>)>(
                    "SELECT id, title, status, priority, created_at FROM work_orders
                 WHERE organization_id = $1 AND asset_id = $2 ORDER BY created_at DESC LIMIT 20",
                )
                .bind(Uuid::from(ctx.organization_id))
                .bind(id)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;

            for (wo_id, title, status, priority, created_at) in &wos {
                parts.push(format!(
                    "Work Order {}: {} (Status: {}, Priority: {}, Created: {})",
                    wo_id,
                    title,
                    status,
                    priority,
                    created_at.map(|d| d.to_string()).unwrap_or_default()
                ));
                records += 1;
                sources.push(Source {
                    id: wo_id.to_string(),
                    source_type: "work_order".into(),
                    title: title.clone(),
                    relevance: 0.9,
                    field: Some("title".into()),
                    source_scope: Some("PRIVATE_TENANT".into()),
                    retrieval_type: Some("SQL".into()),
                    verified_by_sql: Some(true),
                    quote: None,
                    timestamp: created_at.map(|d| d.to_rfc3339()),
                });
            }
        }

        Ok(FetchResult {
            parts,
            records,
            sources,
        })
    }

    async fn fetch_similar(
        &self,
        ctx: &TenantContext,
        query: &str,
    ) -> Result<FetchResult, SipError> {
        let mut parts = Vec::new();
        let mut records: usize = 0;
        let mut sources = Vec::new();

        // Try pgvector similarity search first
        let vector_sources = self.vector_search(ctx, query).await;
        for vs in &vector_sources {
            parts.push(format!(
                "Vector Match [{}]: {}",
                vs.source_type,
                vs.quote.as_deref().unwrap_or(&vs.title)
            ));
            records += 1;
            sources.push(Source {
                id: vs.id.clone(),
                source_type: vs.source_type.clone(),
                title: vs.title.clone(),
                relevance: vs.relevance,
                field: Some("content".into()),
                source_scope: Some("PRIVATE_TENANT".into()),
                retrieval_type: Some("VECTOR".into()),
                verified_by_sql: Some(false),
                quote: vs.quote.clone(),
                timestamp: None,
            });
        }

        // Fallback: text-based search on work orders
        let query_lower = query.to_lowercase();
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();
        if !keywords.is_empty() {
            for keyword in &keywords[..keywords.len().min(3)] {
                let wos = sqlx::query_as::<_, (Uuid, String, String, Option<String>)>(
                    "SELECT id, title, status, resolution_notes FROM work_orders
                     WHERE organization_id = $1
                     AND (LOWER(title) LIKE $2 OR LOWER(resolution_notes) LIKE $2)
                     AND created_at > NOW() - INTERVAL '180 days'
                     ORDER BY created_at DESC LIMIT 10",
                )
                .bind(Uuid::from(ctx.organization_id))
                .bind(format!("%{}%", keyword))
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;

                for (id, title, status, notes) in &wos {
                    parts.push(format!(
                        "Similar Work Order {}: {} (Status: {}, Notes: {})",
                        id,
                        title,
                        status,
                        notes.as_deref().unwrap_or("N/A")
                    ));
                    records += 1;
                    sources.push(Source {
                        id: id.to_string(),
                        source_type: "work_order".into(),
                        title: title.clone(),
                        relevance: 0.7,
                        field: Some("resolution_notes".into()),
                        source_scope: Some("PRIVATE_TENANT".into()),
                        retrieval_type: Some("VECTOR".into()),
                        verified_by_sql: Some(true),
                        quote: None,
                        timestamp: None,
                    });
                }
            }
        }

        Ok(FetchResult {
            parts,
            records,
            sources,
        })
    }

    async fn fetch_related(
        &self,
        ctx: &TenantContext,
        query: &str,
    ) -> Result<FetchResult, SipError> {
        let mut parts = Vec::new();
        let mut records: usize = 0;
        let mut sources = Vec::new();

        let query_lower = query.to_lowercase();

        // Find assets matching query and their related work orders
        let asset_rows = sqlx::query_as::<_, (Uuid, String, Option<Uuid>)>(
            "SELECT id, name, location_id FROM assets WHERE organization_id = $1 LIMIT 200",
        )
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        let matched: Vec<_> = asset_rows
            .iter()
            .filter(|(_, name, _)| query_lower.contains(&name.to_lowercase()))
            .collect();

        for (id, name, location_id) in &matched {
            parts.push(format!(
                "Related Asset: {} (Location: {:?})",
                name, location_id
            ));
            records += 1;
            sources.push(Source {
                id: id.to_string(),
                source_type: "asset".into(),
                title: name.clone(),
                relevance: 0.85,
                field: Some("name".into()),
                source_scope: Some("PRIVATE_TENANT".into()),
                retrieval_type: Some("GRAPH".into()),
                verified_by_sql: Some(true),
                quote: None,
                timestamp: None,
            });

            // Get related work orders
            let wos = sqlx::query_as::<_, (Uuid, String, String)>(
                "SELECT id, title, status FROM work_orders
                 WHERE organization_id = $1 AND asset_id = $2 AND status NOT IN ('COMPLETED', 'CANCELLED')
                 ORDER BY created_at DESC LIMIT 10"
            )
            .bind(Uuid::from(ctx.organization_id))
            .bind(id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;

            for (wo_id, title, status) in &wos {
                parts.push(format!(
                    "Related Work Order {}: {} (Status: {})",
                    wo_id, title, status
                ));
                records += 1;
                sources.push(Source {
                    id: wo_id.to_string(),
                    source_type: "work_order".into(),
                    title: title.clone(),
                    relevance: 0.7,
                    field: Some("title".into()),
                    source_scope: Some("PRIVATE_TENANT".into()),
                    retrieval_type: Some("GRAPH".into()),
                    verified_by_sql: Some(true),
                    quote: None,
                    timestamp: None,
                });
            }
        }

        Ok(FetchResult {
            parts,
            records,
            sources,
        })
    }

    async fn fetch_temporal(
        &self,
        ctx: &TenantContext,
        query: &str,
    ) -> Result<FetchResult, SipError> {
        let mut parts = Vec::new();
        let mut records: usize = 0;
        let mut sources = Vec::new();

        let query_lower = query.to_lowercase();

        // Find target asset if referenced
        let asset_rows = sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, name FROM assets WHERE organization_id = $1 LIMIT 200",
        )
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        let target_id = asset_rows
            .iter()
            .find(|(_, name)| query_lower.contains(&name.to_lowercase()))
            .map(|(id, _)| *id);

        // Fetch status history
        if let Some(asset_id) = target_id {
            let history = sqlx::query_as::<_, (String, Option<String>, chrono::DateTime<Utc>)>(
                "SELECT to_status::text, reason, created_at FROM work_order_status_history
                 WHERE work_order_id IN (SELECT id FROM work_orders WHERE asset_id = $1)
                 ORDER BY created_at DESC LIMIT 20",
            )
            .bind(asset_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;

            for (to_status, reason, created_at) in &history {
                parts.push(format!(
                    "Status Change: {} at {} (Reason: {})",
                    to_status,
                    created_at,
                    reason.as_deref().unwrap_or("N/A")
                ));
                records += 1;
                sources.push(Source {
                    id: asset_id.to_string(),
                    source_type: "activity".into(),
                    title: format!("Status change to {}", to_status),
                    relevance: 0.7,
                    field: Some("status_history".into()),
                    source_scope: Some("PRIVATE_TENANT".into()),
                    retrieval_type: Some("TEMPORAL".into()),
                    verified_by_sql: Some(true),
                    quote: None,
                    timestamp: Some(created_at.to_rfc3339()),
                });
            }
        }

        // Fallback: recent activity across org
        let activities = sqlx::query_as::<_, (String, String, String, chrono::DateTime<Utc>)>(
            "SELECT entity_type, entity_id::text, action, created_at FROM activities
             WHERE organization_id = $1 AND created_at > NOW() - INTERVAL '90 days'
             ORDER BY created_at DESC LIMIT 30",
        )
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        for (entity_type, entity_id, action, created_at) in &activities {
            parts.push(format!(
                "Activity: {} {} {} at {}",
                action, entity_type, entity_id, created_at
            ));
            records += 1;
            sources.push(Source {
                id: entity_id.clone(),
                source_type: "activity".into(),
                title: format!("{} {}", action, entity_type),
                relevance: 0.5,
                field: Some("action".into()),
                source_scope: Some("PRIVATE_TENANT".into()),
                retrieval_type: Some("TEMPORAL".into()),
                verified_by_sql: Some(true),
                quote: None,
                timestamp: Some(created_at.to_rfc3339()),
            });
        }

        Ok(FetchResult {
            parts,
            records,
            sources,
        })
    }

    async fn fetch_documents(
        &self,
        ctx: &TenantContext,
        query: &str,
    ) -> Result<FetchResult, SipError> {
        let mut parts = Vec::new();
        let mut records: usize = 0;
        let mut sources = Vec::new();

        // TODO: Integrate pgvector for document chunk semantic search
        // For now, text-based fallback on document names
        let query_lower = query.to_lowercase();
        let keywords: Vec<&str> = query_lower.split_whitespace().collect();

        if !keywords.is_empty() {
            for keyword in &keywords[..keywords.len().min(3)] {
                let docs = sqlx::query_as::<_, (Uuid, String, String, Option<String>)>(
                    "SELECT id, name, document_type::text, text_content FROM documents
                     WHERE organization_id = $1
                     AND (LOWER(name) LIKE $2 OR LOWER(COALESCE(text_content, '')) LIKE $2)
                     AND archived_at IS NULL
                     LIMIT 10",
                )
                .bind(Uuid::from(ctx.organization_id))
                .bind(format!("%{}%", keyword))
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SipError::Validation(e.to_string()))?;

                for (id, name, doc_type, text_content) in &docs {
                    let excerpt = text_content
                        .as_deref()
                        .unwrap_or("")
                        .chars()
                        .take(200)
                        .collect::<String>();
                    parts.push(format!(
                        "Document: {} (Type: {})\nExcerpt: {}",
                        name, doc_type, excerpt
                    ));
                    records += 1;
                    sources.push(Source {
                        id: id.to_string(),
                        source_type: "document".into(),
                        title: name.clone(),
                        relevance: 0.65,
                        field: Some("text_content".into()),
                        source_scope: Some("PRIVATE_TENANT".into()),
                        retrieval_type: Some("RAG".into()),
                        verified_by_sql: Some(true),
                        quote: if !excerpt.is_empty() {
                            Some(excerpt)
                        } else {
                            None
                        },
                        timestamp: None,
                    });
                }
            }
        }

        Ok(FetchResult {
            parts,
            records,
            sources,
        })
    }

    async fn fetch_recent_work_orders(&self, ctx: &TenantContext) -> Result<FetchResult, SipError> {
        let mut parts = Vec::new();
        let mut records: usize = 0;
        let mut sources = Vec::new();

        // Recent work orders
        let wos =
            sqlx::query_as::<_, (Uuid, String, String, String, Option<chrono::DateTime<Utc>>)>(
                "SELECT id, title, status, priority, created_at FROM work_orders
             WHERE organization_id = $1 AND created_at > NOW() - INTERVAL '30 days'
             ORDER BY created_at DESC LIMIT 20",
            )
            .bind(Uuid::from(ctx.organization_id))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;

        for (id, title, status, priority, created_at) in &wos {
            parts.push(format!(
                "Recent Work Order {}: {} (Status: {}, Priority: {}, Created: {})",
                id,
                title,
                status,
                priority,
                created_at.map(|d| d.to_string()).unwrap_or_default()
            ));
            records += 1;
            sources.push(Source {
                id: id.to_string(),
                source_type: "work_order".into(),
                title: title.clone(),
                relevance: 0.8,
                field: Some("title".into()),
                source_scope: Some("PRIVATE_TENANT".into()),
                retrieval_type: Some("SQL".into()),
                verified_by_sql: Some(true),
                quote: None,
                timestamp: created_at.map(|d| d.to_rfc3339()),
            });
        }

        // Critical assets (DOWN/DEGRADED)
        let assets = sqlx::query_as::<_, (Uuid, String, String, String)>(
            "SELECT id, name, status, criticality FROM assets
             WHERE organization_id = $1 AND status IN ('DOWN', 'DEGRADED')
             LIMIT 20",
        )
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        for (id, name, status, criticality) in &assets {
            parts.push(format!(
                "Asset with issue {}: {} (Status: {}, Criticality: {})",
                id, name, status, criticality
            ));
            records += 1;
            sources.push(Source {
                id: id.to_string(),
                source_type: "asset".into(),
                title: name.clone(),
                relevance: 0.85,
                field: Some("status".into()),
                source_scope: Some("PRIVATE_TENANT".into()),
                retrieval_type: Some("SQL".into()),
                verified_by_sql: Some(true),
                quote: None,
                timestamp: None,
            });
        }

        // Open work orders
        let open_wos = sqlx::query_as::<_, (Uuid, String, String)>(
            "SELECT id, title, priority FROM work_orders
             WHERE organization_id = $1 AND status NOT IN ('COMPLETED', 'CLOSED', 'CANCELLED')
             ORDER BY created_at DESC LIMIT 20",
        )
        .bind(Uuid::from(ctx.organization_id))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        for (id, title, priority) in &open_wos {
            parts.push(format!(
                "Open Work Order {}: {} (Priority: {})",
                id, title, priority
            ));
            records += 1;
            sources.push(Source {
                id: id.to_string(),
                source_type: "work_order".into(),
                title: title.clone(),
                relevance: 0.75,
                field: Some("title".into()),
                source_scope: Some("PRIVATE_TENANT".into()),
                retrieval_type: Some("SQL".into()),
                verified_by_sql: Some(true),
                quote: None,
                timestamp: None,
            });
        }

        Ok(FetchResult {
            parts,
            records,
            sources,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_chat_stream(
        ctx: &TenantContext,
        message: &str,
        conversation_id: AIConversationId,
        context: String,
        records_queried: usize,
        retrieval_paths: Vec<String>,
        provider: Arc<dyn LlmProvider>,
        pool: &PgPool,
        tx: tokio::sync::mpsc::UnboundedSender<AIStreamEvent>,
    ) -> Result<(), SipError> {
        let _ = tx.send(AIStreamEvent::Context {
            records_queried: records_queried as i32,
            paths: retrieval_paths.clone(),
        });

        // Load conversation history
        let recent_rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT role, content FROM ai_messages WHERE conversation_id = $1 ORDER BY created_at ASC LIMIT 20"
        )
        .bind(Uuid::from(conversation_id))
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        let history_text = recent_rows
            .iter()
            .map(|(role, content)| format!("{}: {}", role, content))
            .collect::<Vec<_>>()
            .join("\n");

        let system_prompt = r#"You are a maintenance intelligence assistant for the SIP platform. Answer based ONLY on the context provided below. If the information is not in the context, say "I don't have enough information to answer that question." Do not fabricate."#.to_string();

        let user_prompt = format!(
            r#"CONVERSATION HISTORY:
{history}

CONTEXT:
{context}

RETRIEVAL PATHS USED: {paths}

USER QUESTION: {message}

Respond with valid JSON in this exact format:
{{
  "answer": "your detailed answer here",
  "confidence": 0.0-1.0,
  "verification_status": "VERIFIED|PARTIALLY_VERIFIED|UNSUPPORTED|CONTRADICTED",
  "retrieval_paths": ["SQL", "VECTOR"],
  "sources": [
    {{
      "type": "work_order|asset|document|activity",
      "id": "uuid",
      "field": "resolution_notes|description|name",
      "source_scope": "PRIVATE_TENANT|PUBLIC|SYSTEM_DEFAULT",
      "retrieval_type": "SQL|VECTOR|RAG|GRAPH|TEMPORAL",
      "verified_by_sql": true,
      "quote": "relevant excerpt from source",
      "timestamp": "ISO8601 datetime"
    }}
  ],
  "verified_claims": [
    {{
      "claim": "reproducible claim text",
      "status": "VERIFIED|UNSUPPORTED|CONTRADICTED",
      "supporting_sources": ["uuid1", "uuid2"]
    }}
  ],
  "unsupported_claims": [],
  "contradicted_claims": [],
  "recommended_actions": []
}}"#,
            history = history_text,
            context = context,
            paths = retrieval_paths.join(", "),
            message = message,
        );

        let options = CompletionOptions {
            temperature: 0.2,
            max_tokens: 4096,
            stream: true,
        };

        let mut chunk_rx = match provider
            .complete_stream(&system_prompt, &user_prompt, &options)
            .await
        {
            Ok(rx) => rx,
            Err(e) => {
                let _ = tx.send(AIStreamEvent::Error {
                    message: e.to_string(),
                });
                return Err(SipError::Validation(e.to_string()));
            }
        };

        let mut full_answer = String::new();

        while let Some(chunk_result) = chunk_rx.recv().await {
            match chunk_result {
                Ok(text) => {
                    full_answer.push_str(&text);
                    let _ = tx.send(AIStreamEvent::Chunk { text });
                }
                Err(e) => {
                    let _ = tx.send(AIStreamEvent::Error {
                        message: e.to_string(),
                    });
                    return Err(SipError::Validation(e.to_string()));
                }
            }
        }

        // Parse the full response to extract structured data
        let structured: serde_json::Value =
            serde_json::from_str(&full_answer).unwrap_or_else(|_| {
                serde_json::json!({
                    "answer": full_answer.clone(),
                    "confidence": 0.5,
                    "verification_status": "UNSUPPORTED",
                    "retrieval_paths": [],
                    "sources": [],
                    "verified_claims": [],
                    "unsupported_claims": [],
                    "contradicted_claims": [],
                    "recommended_actions": []
                })
            });

        let answer = structured
            .get("answer")
            .and_then(|v| v.as_str())
            .unwrap_or(&full_answer)
            .to_string();
        let sources = structured
            .get("sources")
            .cloned()
            .unwrap_or(serde_json::json!([]));
        let verification_status = structured
            .get("verification_status")
            .and_then(|v| v.as_str())
            .unwrap_or("UNSUPPORTED")
            .to_string();

        // Emit verification event
        let verified_claims = structured.get("verified_claims").and_then(|v| v.as_array());
        let checks_total = verified_claims.map(|a| a.len() as i32).unwrap_or(0);
        let checks_passed = verified_claims
            .map(|a| {
                a.iter()
                    .filter(|c| c.get("status").and_then(|s| s.as_str()) == Some("VERIFIED"))
                    .count() as i32
            })
            .unwrap_or(0);
        let _ = tx.send(AIStreamEvent::Verification {
            checks_passed,
            checks_total,
            status: verification_status.clone(),
        });

        // Save assistant message
        let ai_repo = PgAIConversationRepository::new(pool.clone());
        let assistant_message = AIMessage {
            id: AIMessageId::new(),
            conversation_id,
            role: "assistant".to_string(),
            content: answer.clone(),
            sources: Some(sources.clone()),
            created_at: Utc::now(),
        };
        ai_repo.create_message(ctx, &assistant_message).await?;

        // Save retrieval trace
        let trace = AIRetrievalTrace {
            id: AIRetrievalTraceId::new(),
            organization_id: ctx.organization_id,
            message_id: assistant_message.id,
            retriever_type: RetrieverType::SQL,
            source_type: "relational".to_string(),
            source_id: uuid::Uuid::nil(),
            query: message.to_string(),
            strategy: "relational".to_string(),
            records_queried: records_queried as i32,
            records_returned: records_queried as i32,
            duration_ms: 0,
            score: None,
            included_in_context: true,
            source_scope: Some("PRIVATE_TENANT".to_string()),
            verified_by_sql: false,
            rank: None,
            created_at: Utc::now(),
        };
        ai_repo.create_retrieval_trace(ctx, &trace).await?;

        // Log activity
        let activity_repo = PgActivityRepository::new(pool.clone());
        let agent_identity_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM agent_identities WHERE organization_id = $1 OR organization_id IS NULL LIMIT 1"
        )
        .bind(Uuid::from(ctx.organization_id))
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(AgentIdentityId::from);
        let activity = Activity {
            id: ActivityId::new(),
            organization_id: ctx.organization_id,
            actor_id: ctx.user_id,
            actor_type: ActorType::AiAgent,
            agent_identity_id,
            plugin_id: None,
            entity_type: "ai_conversation".to_string(),
            entity_id: conversation_id.into(),
            action: "ai.query".to_string(),
            changes: None,
            request_id: None,
            correlation_id: None,
            source: ActivitySource::Ai,
            reason: None,
            metadata: Some(serde_json::json!({
                "query": message,
                "records_queried": records_queried
            })),
            ip_address: None,
            user_agent: None,
            created_at: Utc::now(),
        };
        activity_repo.create_activity(ctx, &activity).await?;

        // Create verification trace
        let verification_trace_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO verification_traces (id, organization_id, conversation_id, message_id, verification_status, verifier_type, checked_claim_count, verified_claim_count, unsupported_claim_count, contradicted_claim_count, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())"
        )
        .bind(verification_trace_id)
        .bind(Uuid::from(ctx.organization_id))
        .bind(Uuid::from(conversation_id))
        .bind(Uuid::from(assistant_message.id))
        .bind(&verification_status)
        .bind("RULE_BASED")
        .bind(checks_total)
        .bind(checks_passed)
        .bind(checks_total - checks_passed)
        .bind(0i32)
        .execute(pool)
        .await
        .map_err(|e| SipError::Validation(e.to_string()))?;

        // Extract sources for done event
        let source_list: Vec<Source> = serde_json::from_value(sources).unwrap_or_default();
        let _ = tx.send(AIStreamEvent::Done {
            sources: source_list,
            verification_status,
            retrieval_paths,
        });

        Ok(())
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, SipError> {
        let embeddings = self
            .provider
            .embed(&[text.to_string()])
            .await
            .map_err(|e| SipError::Validation(e.to_string()))?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| SipError::Validation("No embedding returned".into()))
    }

    pub async fn vector_search(&self, ctx: &TenantContext, query: &str) -> Vec<Source> {
        let _ = set_rls_org_pool(&self.pool, ctx.organization_id).await;

        let embedding = match self.embed(query).await {
            Ok(e) => e,
            Err(_) => return vec![],
        };

        let embedding_str = format!(
            "[{}]",
            embedding
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );

        let rows: Result<Vec<SearchResultRow>, _> = sqlx::query_as(
            "SELECT
                er.source_type::TEXT as source_type,
                er.source_id,
                COALESCE(er.collection, 'default') as collection,
                er.content,
                1 - (er.embedding <=> $1::vector) as similarity,
                er.organization_id
             FROM embedding_records er
             WHERE er.organization_id = $2 OR er.organization_id IS NULL
             ORDER BY er.embedding <=> $1::vector
             LIMIT $3",
        )
        .bind(&embedding_str)
        .bind(Uuid::from(ctx.organization_id))
        .bind(5)
        .fetch_all(&self.pool)
        .await;

        match rows {
            Ok(results) => results
                .into_iter()
                .map(|r| Source {
                    id: r.source_id.to_string(),
                    source_type: r.source_type,
                    title: r.collection,
                    relevance: r.similarity as f32,
                    field: Some("content".into()),
                    source_scope: Some("PRIVATE_TENANT".into()),
                    retrieval_type: Some("VECTOR".into()),
                    verified_by_sql: Some(false),
                    quote: Some(r.content.chars().take(300).collect()),
                    timestamp: None,
                })
                .collect(),
            Err(_) => vec![],
        }
    }

    pub async fn list_conversations(
        &self,
        ctx: &TenantContext,
        user_id: UserId,
    ) -> Result<Vec<AIConversation>, SipError> {
        ctx.require_permission("ai:query")?;
        let repo = PgAIConversationRepository::new(self.pool.clone());
        repo.list_conversations_by_user(ctx, user_id).await
    }

    pub async fn get_conversation(
        &self,
        ctx: &TenantContext,
        conversation_id: AIConversationId,
    ) -> Result<Option<(AIConversation, Vec<AIMessage>)>, SipError> {
        ctx.require_permission("ai:query")?;
        let repo = PgAIConversationRepository::new(self.pool.clone());
        // Get conversation by listing and filtering - MVP approach
        let user_id = ctx.user_id.ok_or(SipError::PermissionDenied)?;
        let conversations = repo.list_conversations_by_user(ctx, user_id).await?;
        let conversation = conversations.into_iter().find(|c| c.id == conversation_id);
        match conversation {
            Some(conv) => {
                let messages = repo
                    .get_messages_by_conversation(ctx, conversation_id)
                    .await?;
                Ok(Some((conv, messages)))
            }
            None => Ok(None),
        }
    }

    pub async fn get_retrieval_trace(
        &self,
        ctx: &TenantContext,
        message_id: AIMessageId,
    ) -> Result<Vec<AIRetrievalTrace>, SipError> {
        ctx.require_permission("ai:query")?;
        let repo = PgAIConversationRepository::new(self.pool.clone());
        repo.list_traces_by_message(ctx, message_id).await
    }

    pub async fn submit_feedback(
        &self,
        ctx: &TenantContext,
        message_id: AIMessageId,
        rating: i32,
        comment: Option<String>,
    ) -> Result<(), SipError> {
        ctx.require_permission("ai:query")?;
        let repo = PgAIConversationRepository::new(self.pool.clone());
        repo.submit_feedback(ctx, message_id, rating, comment).await
    }
}

pub struct OrganizationService {
    repo: PgOrganizationRepository,
}

impl OrganizationService {
    pub fn new(repo: PgOrganizationRepository) -> Self {
        Self { repo }
    }

    pub async fn get(&self, ctx: &TenantContext) -> Result<Organization, SipError> {
        ctx.require_permission("org:read")?;
        self.repo
            .get_by_id(ctx, ctx.organization_id)
            .await?
            .ok_or(SipError::Validation("Organization not found".into()))
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        name: Option<String>,
        timezone: Option<String>,
        default_currency: Option<String>,
    ) -> Result<Organization, SipError> {
        ctx.require_permission("org:manage")?;
        self.repo
            .update(ctx, ctx.organization_id, name, timezone, default_currency)
            .await
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        name: &str,
        slug: &str,
        timezone: Option<&str>,
        default_currency: Option<&str>,
    ) -> Result<Organization, SipError> {
        ctx.require_permission("org:manage")?;
        self.repo
            .create(ctx, name, slug, timezone, default_currency)
            .await
    }
}

pub struct LocationService {
    repo: PgLocationRepository,
}

impl LocationService {
    pub fn new(repo: PgLocationRepository) -> Self {
        Self { repo }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Location>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.list(ctx).await
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: LocationId,
    ) -> Result<Option<Location>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.get(ctx, id).await
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        parent_id: Option<LocationId>,
        name: String,
        location_type: LocationType,
    ) -> Result<Location, SipError> {
        ctx.require_permission("asset:update")?;
        let location = Location {
            id: LocationId::new(),
            organization_id: ctx.organization_id,
            parent_id,
            name,
            location_type,
            geo_latitude: None,
            geo_longitude: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create(ctx, &location).await
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: LocationId,
        name: Option<String>,
        location_type: Option<LocationType>,
    ) -> Result<Location, SipError> {
        ctx.require_permission("asset:update")?;
        self.repo.update(ctx, id, name, location_type).await
    }

    pub async fn list_children(
        &self,
        ctx: &TenantContext,
        parent_id: LocationId,
    ) -> Result<Vec<Location>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.list_children(ctx, parent_id).await
    }

    pub async fn list_assets(
        &self,
        ctx: &TenantContext,
        location_id: LocationId,
    ) -> Result<Vec<Asset>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.list_assets_at(ctx, location_id).await
    }
}

pub struct AssetTypeService {
    repo: PgAssetTypeRepository,
}

impl AssetTypeService {
    pub fn new(repo: PgAssetTypeRepository) -> Self {
        Self { repo }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<AssetType>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.list(ctx).await
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: AssetTypeId,
    ) -> Result<Option<AssetType>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.get(ctx, id).await
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        name: String,
        category: String,
        description: Option<String>,
    ) -> Result<AssetType, SipError> {
        ctx.require_permission("asset:update")?;
        let asset_type = AssetType {
            id: AssetTypeId::new(),
            organization_id: Some(ctx.organization_id),
            name,
            category,
            description,
            schema: None,
            default_pm_schedules: None,
            default_inspection_template: None,
            icon: None,
            is_system: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create(ctx, &asset_type).await
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: AssetTypeId,
        name: Option<String>,
        category: Option<String>,
        description: Option<String>,
    ) -> Result<AssetType, SipError> {
        ctx.require_permission("asset:update")?;
        self.repo.update(ctx, id, name, category, description).await
    }
}

pub struct ManufacturerService {
    repo: PgManufacturerRepository,
}

impl ManufacturerService {
    pub fn new(repo: PgManufacturerRepository) -> Self {
        Self { repo }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Manufacturer>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.list(ctx).await
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: ManufacturerId,
    ) -> Result<Option<Manufacturer>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.get(ctx, id).await
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        name: String,
        website: Option<String>,
        support_url: Option<String>,
    ) -> Result<Manufacturer, SipError> {
        ctx.require_permission("asset:update")?;
        let manufacturer = Manufacturer {
            id: ManufacturerId::new(),
            organization_id: Some(ctx.organization_id),
            name,
            website,
            support_url,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create(ctx, &manufacturer).await
    }
}

pub struct AssetModelService {
    repo: PgAssetModelRepository,
}

impl AssetModelService {
    pub fn new(repo: PgAssetModelRepository) -> Self {
        Self { repo }
    }

    pub async fn list(
        &self,
        ctx: &TenantContext,
        manufacturer_id: Option<ManufacturerId>,
    ) -> Result<Vec<AssetModel>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.list(ctx, manufacturer_id).await
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: AssetModelId,
    ) -> Result<Option<AssetModel>, SipError> {
        ctx.require_permission("asset:read")?;
        self.repo.get(ctx, id).await
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        manufacturer_id: ManufacturerId,
        name: String,
        model_number: String,
        asset_type_id: AssetTypeId,
    ) -> Result<AssetModel, SipError> {
        ctx.require_permission("asset:update")?;
        let model = AssetModel {
            id: AssetModelId::new(),
            organization_id: Some(ctx.organization_id),
            manufacturer_id,
            name,
            model_number,
            revision: None,
            lifecycle_status: LifecycleStatus::Active,
            asset_type_id,
            documentation_url: None,
            default_attributes: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create(ctx, &model).await
    }
}

pub struct UserService {
    repo: PgUserRepository,
}

impl UserService {
    pub fn new(repo: PgUserRepository) -> Self {
        Self { repo }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<User>, SipError> {
        ctx.require_permission("user:read")?;
        self.repo.list(ctx).await
    }

    pub async fn get(&self, ctx: &TenantContext, id: UserId) -> Result<Option<User>, SipError> {
        ctx.require_permission("user:read")?;
        self.repo.get(ctx, id).await
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        email: String,
        name: String,
        role: UserRole,
        password_hash: String,
    ) -> Result<User, SipError> {
        ctx.require_permission("user:manage")?;
        let user = User {
            id: UserId::new(),
            organization_id: ctx.organization_id,
            email,
            name,
            role,
            skills: vec![],
            certifications: None,
            working_hours: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create(ctx, &user, &password_hash).await
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: UserId,
        name: Option<String>,
        role: Option<UserRole>,
        is_active: Option<bool>,
    ) -> Result<User, SipError> {
        ctx.require_permission("user:manage")?;
        self.repo.update(ctx, id, name, role, is_active).await
    }
}

pub struct TeamService {
    repo: PgTeamRepository,
}

impl TeamService {
    pub fn new(repo: PgTeamRepository) -> Self {
        Self { repo }
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Team>, SipError> {
        ctx.require_permission("team:manage")?;
        self.repo.list(ctx).await
    }

    pub async fn get(&self, ctx: &TenantContext, id: TeamId) -> Result<Option<Team>, SipError> {
        ctx.require_permission("team:manage")?;
        self.repo.get(ctx, id).await
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        name: String,
        description: Option<String>,
        lead_id: Option<UserId>,
    ) -> Result<Team, SipError> {
        ctx.require_permission("team:manage")?;
        let team = Team {
            id: TeamId::new(),
            organization_id: ctx.organization_id,
            name,
            description,
            lead_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create(ctx, &team).await
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: TeamId,
        name: Option<String>,
        description: Option<String>,
        lead_id: Option<UserId>,
    ) -> Result<Team, SipError> {
        ctx.require_permission("team:manage")?;
        self.repo.update(ctx, id, name, description, lead_id).await
    }
}

pub struct UpdateChecklistItemInput {
    pub id: InspectionChecklistItemId,
    pub result: Option<ChecklistResult>,
    pub actual_value: Option<String>,
    pub finding: Option<String>,
    pub photo_url: Option<String>,
}

pub struct InspectionService<R: InspectionRepository> {
    repo: R,
}

impl<R: InspectionRepository> InspectionService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: InspectionId,
    ) -> Result<Option<Inspection>, SipError> {
        ctx.require_permission("work_order:read")?;
        self.repo.get(ctx, id).await
    }

    pub async fn list(
        &self,
        ctx: &TenantContext,
        work_order_id: Option<WorkOrderId>,
    ) -> Result<Vec<Inspection>, SipError> {
        ctx.require_permission("work_order:read")?;
        self.repo.list(ctx, work_order_id).await
    }

    pub async fn update_items(
        &self,
        ctx: &TenantContext,
        inspection_id: InspectionId,
        items: Vec<UpdateChecklistItemInput>,
    ) -> Result<Inspection, SipError> {
        ctx.require_permission("work_order:update")?;
        let checklist_items: Vec<InspectionChecklistItem> = items
            .into_iter()
            .map(|i| InspectionChecklistItem {
                id: i.id,
                inspection_id,
                ordinal: 0,
                question: String::new(),
                response_type: sip_domain::entity::inspection::ChecklistResponseType::PassFail,
                expected_value: None,
                actual_value: i.actual_value,
                result: i.result,
                finding: i.finding,
                photo_url: i.photo_url,
            })
            .collect();
        self.repo
            .update_items(ctx, inspection_id, checklist_items)
            .await
    }
}

pub struct CreatePartInput {
    pub name: String,
    pub part_number: Option<String>,
    pub description: Option<String>,
    pub quantity_on_hand: rust_decimal::Decimal,
    pub quantity_minimum: Option<rust_decimal::Decimal>,
    pub unit: String,
    pub unit_cost: Option<rust_decimal::Decimal>,
    pub storage_location: Option<String>,
}

pub struct UpdatePartInput {
    pub name: Option<String>,
    pub part_number: Option<String>,
    pub description: Option<String>,
    pub quantity_on_hand: Option<rust_decimal::Decimal>,
    pub quantity_minimum: Option<rust_decimal::Decimal>,
    pub unit: Option<String>,
    pub unit_cost: Option<rust_decimal::Decimal>,
    pub storage_location: Option<String>,
}

pub struct PartService<R: PartRepository> {
    repo: R,
}

impl<R: PartRepository> PartService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        input: CreatePartInput,
    ) -> Result<Part, SipError> {
        ctx.require_permission("part:manage")?;
        let part = Part {
            id: PartId::new(),
            organization_id: ctx.organization_id,
            name: input.name,
            part_number: input.part_number,
            description: input.description,
            quantity_on_hand: input.quantity_on_hand,
            quantity_minimum: input.quantity_minimum,
            unit: input.unit,
            unit_cost: input.unit_cost,
            storage_location: input.storage_location,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.repo.create(ctx, &part).await
    }

    pub async fn get(&self, ctx: &TenantContext, id: PartId) -> Result<Option<Part>, SipError> {
        ctx.require_permission("part:read")?;
        self.repo.get(ctx, id).await
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Part>, SipError> {
        ctx.require_permission("part:read")?;
        self.repo.list(ctx).await
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: PartId,
        input: UpdatePartInput,
    ) -> Result<Part, SipError> {
        ctx.require_permission("part:manage")?;
        let mut patch = serde_json::json!({});
        if let Some(ref v) = input.name {
            patch["name"] = serde_json::json!(v);
        }
        if let Some(ref v) = input.part_number {
            patch["part_number"] = serde_json::json!(v);
        }
        if let Some(ref v) = input.description {
            patch["description"] = serde_json::json!(v);
        }
        if let Some(v) = input.quantity_on_hand {
            patch["quantity_on_hand"] = serde_json::json!(v);
        }
        if let Some(v) = input.quantity_minimum {
            patch["quantity_minimum"] = serde_json::json!(v);
        }
        if let Some(ref v) = input.unit {
            patch["unit"] = serde_json::json!(v);
        }
        if let Some(v) = input.unit_cost {
            patch["unit_cost"] = serde_json::json!(v);
        }
        if let Some(ref v) = input.storage_location {
            patch["storage_location"] = serde_json::json!(v);
        }
        self.repo.update(ctx, id, patch).await
    }
}

pub struct ScheduleService<R: ScheduleRepository> {
    repo: R,
}

impl<R: ScheduleRepository> ScheduleService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        ctx: &TenantContext,
        input: CreateScheduleInput,
    ) -> Result<Schedule, SipError> {
        ctx.require_permission("schedule:manage")?;
        let schedule = Schedule {
            id: ScheduleId::new(),
            organization_id: ctx.organization_id,
            asset_id: input.asset_id,
            name: input.name,
            trigger_type: input.trigger_type,
            trigger_config: input.trigger_config,
            work_order_template: input.work_order_template,
            next_due: input.next_due,
            last_triggered: None,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            archived_at: None,
            archived_by_id: None,
            archive_reason: None,
        };
        self.repo.create(ctx, &schedule).await
    }

    pub async fn get(
        &self,
        ctx: &TenantContext,
        id: ScheduleId,
    ) -> Result<Option<Schedule>, SipError> {
        ctx.require_permission("schedule:read")?;
        self.repo.get(ctx, id).await
    }

    pub async fn list(&self, ctx: &TenantContext) -> Result<Vec<Schedule>, SipError> {
        ctx.require_permission("schedule:read")?;
        self.repo.list(ctx).await
    }

    pub async fn update(
        &self,
        ctx: &TenantContext,
        id: ScheduleId,
        input: UpdateScheduleInput,
    ) -> Result<Schedule, SipError> {
        ctx.require_permission("schedule:manage")?;
        let mut patch = serde_json::json!({});
        if let Some(name) = input.name {
            patch["name"] = serde_json::json!(name);
        }
        if let Some(enabled) = input.enabled {
            patch["enabled"] = serde_json::json!(enabled);
        }
        if let Some(trigger_config) = input.trigger_config {
            patch["trigger_config"] = trigger_config;
        }
        if let Some(work_order_template) = input.work_order_template {
            patch["work_order_template"] = work_order_template;
        }
        if let Some(next_due) = input.next_due {
            patch["next_due"] = serde_json::json!(next_due.to_rfc3339());
        }
        self.repo.update(ctx, id, patch).await
    }

    pub async fn archive(&self, ctx: &TenantContext, id: ScheduleId) -> Result<Schedule, SipError> {
        ctx.require_permission("schedule:manage")?;
        let archived_by = ctx.user_id.ok_or(SipError::PermissionDenied)?;
        self.repo.archive(ctx, id, archived_by).await
    }
}

pub struct CreateScheduleInput {
    pub asset_id: AssetId,
    pub name: String,
    pub trigger_type: ScheduleTriggerType,
    pub trigger_config: Option<serde_json::Value>,
    pub work_order_template: Option<serde_json::Value>,
    pub next_due: Option<chrono::DateTime<Utc>>,
}

pub struct UpdateScheduleInput {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub trigger_config: Option<serde_json::Value>,
    pub work_order_template: Option<serde_json::Value>,
    pub next_due: Option<chrono::DateTime<Utc>>,
}

// ── Migration Service ──

pub struct MigrationService {
    pool: PgPool,
}

impl MigrationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_job(
        &self,
        ctx: &TenantContext,
        name: String,
        source_system: String,
        source_object_type: String,
        description: Option<String>,
    ) -> Result<MigrationJob, SipError> {
        ctx.require_permission("migration:create")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        let job = MigrationJob {
            id: MigrationJobId::new(),
            organization_id: ctx.organization_id,
            name,
            description,
            source_system,
            source_object_type,
            status: MigrationJobStatus::Draft,
            source_record_count: 0,
            valid_record_count: 0,
            imported_record_count: 0,
            error_count: 0,
            created_by: ctx.user_id.ok_or(SipError::PermissionDenied)?,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        repo.create_job(ctx, &job).await
    }

    pub async fn list_jobs(&self, ctx: &TenantContext) -> Result<Vec<MigrationJob>, SipError> {
        ctx.require_permission("migration:read")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.list_jobs(ctx).await
    }

    pub async fn get_job(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Option<MigrationJob>, SipError> {
        ctx.require_permission("migration:read")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.get_job(ctx, job_id).await
    }

    pub async fn add_source_records(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
        records: Vec<serde_json::Value>,
    ) -> Result<Vec<MigrationSourceRecord>, SipError> {
        ctx.require_permission("migration:create")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        let now = Utc::now();
        let source_records: Vec<MigrationSourceRecord> = records
            .into_iter()
            .enumerate()
            .map(|(i, raw)| MigrationSourceRecord {
                id: MigrationSourceRecordId::new(),
                organization_id: ctx.organization_id,
                job_id,
                batch_id: None,
                external_id: raw
                    .get("external_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                source_object_type: raw
                    .get("source_object_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                raw_data: raw,
                status: "pending".to_string(),
                row_number: Some((i + 1) as i32),
                created_at: now,
            })
            .collect();
        repo.create_source_records(ctx, &source_records).await?;
        let count = source_records.len() as i32;
        repo.update_job_counts(ctx, job_id, count, 0, 0, 0).await?;
        repo.update_job_status(ctx, job_id, MigrationJobStatus::Uploaded)
            .await?;
        Ok(source_records)
    }

    pub async fn save_field_mappings(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
        mappings: Vec<MigrationFieldMapping>,
    ) -> Result<(), SipError> {
        ctx.require_permission("migration:map")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.save_mappings(ctx, &mappings).await?;
        repo.update_job_status(ctx, job_id, MigrationJobStatus::Mapped)
            .await?;
        Ok(())
    }

    pub async fn get_field_mappings(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationFieldMapping>, SipError> {
        ctx.require_permission("migration:read")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.get_mappings(ctx, job_id).await
    }

    pub async fn get_source_records(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationSourceRecord>, SipError> {
        ctx.require_permission("migration:read")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.get_source_records(ctx, job_id).await
    }

    pub async fn get_staged_records(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationStagedRecord>, SipError> {
        ctx.require_permission("migration:read")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.get_staged_records(ctx, job_id).await
    }

    pub async fn validate_job(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationValidationIssue>, SipError> {
        ctx.require_permission("migration:validate")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        let source_records = repo.get_source_records(ctx, job_id).await?;
        let mappings = repo.get_mappings(ctx, job_id).await?;
        let now = Utc::now();
        let mut staged: Vec<MigrationStagedRecord> = Vec::new();
        let mut issues: Vec<MigrationValidationIssue> = Vec::new();

        for src in &source_records {
            let mut canonical = serde_json::json!({});
            let entity_type = &src.source_object_type;

            for mapping in mappings
                .iter()
                .filter(|m| m.target_entity_type == *entity_type)
            {
                let source_val = src.raw_data.get(&mapping.source_field);
                if source_val.is_none() && mapping.is_required {
                    issues.push(MigrationValidationIssue {
                        id: MigrationValidationIssueId::new(),
                        organization_id: ctx.organization_id,
                        job_id,
                        staged_record_id: MigrationStagedRecordId::new(),
                        severity: "error".to_string(),
                        field: mapping.source_field.clone(),
                        message: format!("Required field '{}' is missing", mapping.source_field),
                        created_at: now,
                    });
                } else if let Some(val) = source_val {
                    canonical[&mapping.target_field] = val.clone();
                } else if let Some(ref default) = mapping.default_value {
                    canonical[&mapping.target_field] = serde_json::Value::String(default.clone());
                }
            }

            let staged_record = MigrationStagedRecord {
                id: MigrationStagedRecordId::new(),
                organization_id: ctx.organization_id,
                job_id,
                source_record_id: src.id,
                target_entity_type: entity_type.clone(),
                canonical_data: canonical,
                status: if issues.is_empty() {
                    "pending_validation".to_string()
                } else {
                    "invalid".to_string()
                },
                validation_errors: if issues.is_empty() {
                    None
                } else {
                    Some(serde_json::json!(issues
                        .iter()
                        .map(|i| serde_json::json!({
                            "severity": i.severity,
                            "field": i.field,
                            "message": i.message,
                        }))
                        .collect::<Vec<_>>()))
                },
                created_at: now,
                updated_at: now,
            };
            staged.push(staged_record);
        }

        for staged_rec in staged.iter_mut() {
            let canonical: serde_json::Value = staged_rec.canonical_data.clone();

            match staged_rec.target_entity_type.as_str() {
                "asset"
                    if canonical
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map_or(true, |s| s.is_empty())
                    => {
                        issues.push(Self::create_issue(
                            ctx.organization_id,
                            job_id,
                            staged_rec.id,
                            "error",
                            "name",
                            "Asset name is required",
                        ));
                    }
                "work_order"
                    if canonical
                        .get("title")
                        .and_then(|v| v.as_str())
                        .map_or(true, |s| s.is_empty())
                    => {
                        issues.push(Self::create_issue(
                            ctx.organization_id,
                            job_id,
                            staged_rec.id,
                            "error",
                            "title",
                            "Work order title is required",
                        ));
                    }
                "location"
                    if canonical
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map_or(true, |s| s.is_empty())
                    => {
                        issues.push(Self::create_issue(
                            ctx.organization_id,
                            job_id,
                            staged_rec.id,
                            "error",
                            "name",
                            "Location name is required",
                        ));
                    }
                "part" => {
                    if canonical
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map_or(true, |s| s.is_empty())
                    {
                        issues.push(Self::create_issue(
                            ctx.organization_id,
                            job_id,
                            staged_rec.id,
                            "error",
                            "name",
                            "Part name is required",
                        ));
                    }
                    if canonical
                        .get("part_number")
                        .and_then(|v| v.as_str())
                        .map_or(true, |s| s.is_empty())
                    {
                        issues.push(Self::create_issue(
                            ctx.organization_id,
                            job_id,
                            staged_rec.id,
                            "error",
                            "part_number",
                            "Part number is required",
                        ));
                    }
                }
                _ => {}
            }

            for date_field in &[
                "purchase_date",
                "warranty_expiry",
                "due_date",
                "completed_date",
            ] {
                if let Some(val) = canonical.get(date_field).and_then(|v| v.as_str()) {
                    if !val.is_empty()
                        && chrono::DateTime::parse_from_rfc3339(val).is_err()
                        && chrono::NaiveDate::parse_from_str(val, "%Y-%m-%d").is_err()
                    {
                        issues.push(Self::create_issue(
                            ctx.organization_id,
                            job_id,
                            staged_rec.id,
                            "warning",
                            date_field,
                            &format!("Date '{}' is not in ISO 8601 format", val),
                        ));
                    }
                }
            }

            if let Some(status) = canonical.get("status").and_then(|v| v.as_str()) {
                let valid_statuses = [
                    "operational",
                    "degraded",
                    "down",
                    "maintenance",
                    "retired",
                    "draft",
                    "open",
                    "in_progress",
                    "on_hold",
                    "completed",
                    "reviewed",
                    "closed",
                    "cancelled",
                ];
                if !valid_statuses.contains(&status.to_lowercase().as_str()) {
                    issues.push(Self::create_issue(
                        ctx.organization_id,
                        job_id,
                        staged_rec.id,
                        "warning",
                        "status",
                        &format!("Unknown status '{}'", status),
                    ));
                }
            }
            if let Some(priority) = canonical.get("priority").and_then(|v| v.as_str()) {
                let valid_priorities = ["critical", "high", "medium", "low", "routine"];
                if !valid_priorities.contains(&priority.to_lowercase().as_str()) {
                    issues.push(Self::create_issue(
                        ctx.organization_id,
                        job_id,
                        staged_rec.id,
                        "warning",
                        "priority",
                        &format!("Unknown priority '{}'", priority),
                    ));
                }
            }
            if let Some(criticality) = canonical.get("criticality").and_then(|v| v.as_str()) {
                let valid_criticalities = ["critical", "high", "medium", "low"];
                if !valid_criticalities.contains(&criticality.to_lowercase().as_str()) {
                    issues.push(Self::create_issue(
                        ctx.organization_id,
                        job_id,
                        staged_rec.id,
                        "warning",
                        "criticality",
                        &format!("Unknown criticality '{}'", criticality),
                    ));
                }
            }

            if let Some(ext_id) = canonical.get("external_id").and_then(|v| v.as_str()) {
                if !ext_id.is_empty() {
                    let existing = sqlx::query_scalar::<_, i64>(
                        "SELECT COUNT(*) FROM migration_external_id_maps WHERE organization_id = $1 AND source_external_id = $2",
                    )
                    .bind(Uuid::from(ctx.organization_id))
                    .bind(ext_id)
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);
                    if existing > 0 {
                        issues.push(Self::create_issue(
                            ctx.organization_id,
                            job_id,
                            staged_rec.id,
                            "warning",
                            "external_id",
                            &format!("External ID '{}' already exists in SIP", ext_id),
                        ));
                    }
                }
            }

            if staged_rec.target_entity_type == "asset" {
                if let Some(sn) = canonical.get("serial_number").and_then(|v| v.as_str()) {
                    if !sn.is_empty() {
                        let existing = sqlx::query_scalar::<_, i64>(
                            "SELECT COUNT(*) FROM assets WHERE organization_id = $1 AND serial_number = $2",
                        )
                        .bind(Uuid::from(ctx.organization_id))
                        .bind(sn)
                        .fetch_one(&self.pool)
                        .await
                        .unwrap_or(0);
                        if existing > 0 {
                            issues.push(Self::create_issue(
                                ctx.organization_id,
                                job_id,
                                staged_rec.id,
                                "warning",
                                "serial_number",
                                &format!("Serial number '{}' already exists in SIP assets", sn),
                            ));
                        }
                    }
                }
            }

            let has_blocking = issues.iter().any(|i| {
                i.staged_record_id == staged_rec.id
                    && (i.severity == "blocking" || i.severity == "error")
            });
            if has_blocking {
                staged_rec.status = "invalid".to_string();
            }
        }

        if !staged.is_empty() {
            repo.create_staged_records(ctx, &staged).await?;
        }
        if !issues.is_empty() {
            repo.create_validation_issues(ctx, &issues).await?;
        }

        repo.update_job_status(ctx, job_id, MigrationJobStatus::Validated)
            .await?;
        let valid_count = staged
            .iter()
            .filter(|s| s.status == "pending_validation")
            .count() as i32;
        repo.update_job_counts(
            ctx,
            job_id,
            source_records.len() as i32,
            valid_count,
            0,
            issues.len() as i32,
        )
        .await?;
        Ok(issues)
    }

    fn create_issue(
        org_id: OrganizationId,
        job_id: MigrationJobId,
        staged_id: MigrationStagedRecordId,
        severity: &str,
        field: &str,
        message: &str,
    ) -> MigrationValidationIssue {
        MigrationValidationIssue {
            id: MigrationValidationIssueId::new(),
            organization_id: org_id,
            job_id,
            staged_record_id: staged_id,
            severity: severity.to_string(),
            field: field.to_string(),
            message: message.to_string(),
            created_at: Utc::now(),
        }
    }

    pub async fn dry_run(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<MigrationRun, SipError> {
        ctx.require_permission("migration:dry_run")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        let staged_records = repo.get_staged_records(ctx, job_id).await?;
        let now = Utc::now();

        let valid_count = staged_records
            .iter()
            .filter(|s| s.status == "pending_validation")
            .count() as i32;
        let error_count = staged_records
            .iter()
            .filter(|s| s.status == "invalid")
            .count() as i32;

        let run = MigrationRun {
            id: MigrationRunId::new(),
            organization_id: ctx.organization_id,
            job_id,
            run_type: MigrationRunType::DryRun,
            status: MigrationRunStatus::Completed,
            records_processed: staged_records.len() as i32,
            records_created: valid_count,
            records_updated: 0,
            records_skipped: 0,
            records_failed: error_count,
            started_at: Some(now),
            completed_at: Some(Utc::now()),
            created_at: now,
        };
        repo.create_run(ctx, &run).await?;
        Ok(run)
    }

    pub async fn execute_import(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<MigrationRun, SipError> {
        ctx.require_permission("migration:execute")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        let asset_repo =
            sip_infrastructure::repositories::PgAssetRepository::new(self.pool.clone());

        repo.update_job_status(ctx, job_id, MigrationJobStatus::Importing)
            .await?;

        let now = Utc::now();
        let mut run = MigrationRun {
            id: MigrationRunId::new(),
            organization_id: ctx.organization_id,
            job_id,
            run_type: MigrationRunType::Import,
            status: MigrationRunStatus::Running,
            records_processed: 0,
            records_created: 0,
            records_updated: 0,
            records_skipped: 0,
            records_failed: 0,
            started_at: Some(now),
            completed_at: None,
            created_at: now,
        };
        repo.create_run(ctx, &run).await?;

        let staged_records = repo.get_staged_records(ctx, job_id).await?;

        for staged in &staged_records {
            if staged.status != "pending_validation" {
                run.records_skipped += 1;
                continue;
            }

            let result = match staged.target_entity_type.as_str() {
                "asset" => {
                    let dto = serde_json::from_value::<
                        sip_domain::entity::migration::CanonicalAssetImport,
                    >(staged.canonical_data.clone());
                    match dto {
                        Ok(dto) => {
                            let asset = Asset {
                                id: AssetId::new(),
                                organization_id: ctx.organization_id,
                                location_id: None,
                                parent_id: None,
                                asset_type_id: AssetTypeId::from(Uuid::new_v4()),
                                model_id: None,
                                name: dto.name.clone(),
                                description: dto.notes.clone(),
                                serial_number: dto.serial_number.clone(),
                                firmware_version: None,
                                software_version: None,
                                hardware_revision: None,
                                status: AssetStatus::Operational,
                                version: 1,
                                criticality: sip_domain::entity::asset::Criticality::Medium,
                                installed_date: None,
                                warranty_expiry: None,
                                attributes: dto.custom_attributes.clone(),
                                tags: dto.tags.unwrap_or_default(),
                                metadata: None,
                                created_at: Utc::now(),
                                updated_at: Utc::now(),
                            };
                            match asset_repo.create_asset(ctx, &asset).await {
                                Ok(created) => {
                                    if let Some(ref ext_id) = dto.external_id {
                                        let _ = repo
                                            .create_external_id_map(
                                                ctx,
                                                &MigrationExternalIdMap {
                                                    id: MigrationExternalIdMapId::new(),
                                                    organization_id: ctx.organization_id,
                                                    job_id,
                                                    run_id: run.id,
                                                    source_system: "import".into(),
                                                    source_object_type: "asset".into(),
                                                    source_external_id: ext_id.clone(),
                                                    sip_entity_type: "asset".into(),
                                                    sip_entity_id: Uuid::from(created.id),
                                                    created_at: Utc::now(),
                                                },
                                            )
                                            .await;
                                    }
                                    MigrationImportResult {
                                        id: MigrationImportResultId::new(),
                                        organization_id: ctx.organization_id,
                                        job_id,
                                        run_id: run.id,
                                        staged_record_id: staged.id,
                                        sip_entity_type: "asset".into(),
                                        sip_entity_id: Some(Uuid::from(created.id)),
                                        action: "created".into(),
                                        error_message: None,
                                        created_at: Utc::now(),
                                    }
                                }
                                Err(e) => MigrationImportResult {
                                    id: MigrationImportResultId::new(),
                                    organization_id: ctx.organization_id,
                                    job_id,
                                    run_id: run.id,
                                    staged_record_id: staged.id,
                                    sip_entity_type: "asset".into(),
                                    sip_entity_id: None,
                                    action: "failed".into(),
                                    error_message: Some(e.to_string()),
                                    created_at: Utc::now(),
                                },
                            }
                        }
                        Err(e) => MigrationImportResult {
                            id: MigrationImportResultId::new(),
                            organization_id: ctx.organization_id,
                            job_id,
                            run_id: run.id,
                            staged_record_id: staged.id,
                            sip_entity_type: "asset".into(),
                            sip_entity_id: None,
                            action: "failed".into(),
                            error_message: Some(format!("Invalid DTO: {}", e)),
                            created_at: Utc::now(),
                        },
                    }
                }
                _ => MigrationImportResult {
                    id: MigrationImportResultId::new(),
                    organization_id: ctx.organization_id,
                    job_id,
                    run_id: run.id,
                    staged_record_id: staged.id,
                    sip_entity_type: staged.target_entity_type.clone(),
                    sip_entity_id: None,
                    action: "skipped".into(),
                    error_message: Some(format!(
                        "No import handler for entity type '{}'",
                        staged.target_entity_type
                    )),
                    created_at: Utc::now(),
                },
            };

            let is_error = result.action == "failed";
            if is_error {
                run.records_failed += 1;
            } else if result.action == "created" {
                run.records_created += 1;
            } else {
                run.records_skipped += 1;
            }
            run.records_processed += 1;

            let _ = repo.create_import_result(ctx, &result).await;
        }

        run.status = if run.records_failed > 0 {
            MigrationRunStatus::Failed
        } else {
            MigrationRunStatus::Completed
        };
        run.completed_at = Some(Utc::now());
        repo.update_run(ctx, &run).await?;

        let final_status = if run.records_failed > 0 {
            MigrationJobStatus::Failed
        } else if run.records_skipped > 0 {
            MigrationJobStatus::CompletedWithWarnings
        } else {
            MigrationJobStatus::Completed
        };
        repo.update_job_status(ctx, job_id, final_status).await?;
        repo.update_job_counts(
            ctx,
            job_id,
            staged_records.len() as i32,
            staged_records
                .iter()
                .filter(|s| s.status == "pending_validation")
                .count() as i32,
            run.records_created,
            run.records_failed,
        )
        .await?;

        Ok(run)
    }

    pub async fn cancel_job(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<(), SipError> {
        ctx.require_permission("migration:delete")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.update_job_status(ctx, job_id, MigrationJobStatus::Cancelled)
            .await
    }

    pub async fn rollback_job(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<MigrationRun, SipError> {
        ctx.require_permission("migration:rollback")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        let asset_repo =
            sip_infrastructure::repositories::PgAssetRepository::new(self.pool.clone());

        let now = Utc::now();
        let mut run = MigrationRun {
            id: MigrationRunId::new(),
            organization_id: ctx.organization_id,
            job_id,
            run_type: MigrationRunType::Rollback,
            status: MigrationRunStatus::Running,
            records_processed: 0,
            records_created: 0,
            records_updated: 0,
            records_skipped: 0,
            records_failed: 0,
            started_at: Some(now),
            completed_at: None,
            created_at: now,
        };
        repo.create_run(ctx, &run).await?;

        let external_id_maps = repo.get_external_id_maps(ctx, job_id).await?;

        for entry in &external_id_maps {
            if entry.sip_entity_type == "asset" {
                match asset_repo
                    .archive_asset(ctx, AssetId::from(entry.sip_entity_id))
                    .await
                {
                    Ok(_) => {
                        run.records_processed += 1;
                        run.records_created += 1;
                    }
                    Err(e) => {
                        run.records_failed += 1;
                        let _ = repo
                            .create_import_result(
                                ctx,
                                &MigrationImportResult {
                                    id: MigrationImportResultId::new(),
                                    organization_id: ctx.organization_id,
                                    job_id,
                                    run_id: run.id,
                                    staged_record_id: MigrationStagedRecordId::new(),
                                    sip_entity_type: "asset".into(),
                                    sip_entity_id: Some(entry.sip_entity_id),
                                    action: "rollback_failed".into(),
                                    error_message: Some(e.to_string()),
                                    created_at: Utc::now(),
                                },
                            )
                            .await;
                    }
                }
            }
        }

        run.status = if run.records_failed > 0 {
            MigrationRunStatus::Failed
        } else {
            MigrationRunStatus::Completed
        };
        run.completed_at = Some(Utc::now());
        repo.update_run(ctx, &run).await?;
        repo.update_job_status(ctx, job_id, MigrationJobStatus::RolledBack)
            .await?;

        Ok(run)
    }

    pub async fn get_validation_issues(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationValidationIssue>, SipError> {
        ctx.require_permission("migration:read")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.get_validation_issues(ctx, job_id).await
    }

    pub async fn get_duplicates(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationDuplicateCandidate>, SipError> {
        ctx.require_permission("migration:read")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.get_duplicate_candidates(ctx, job_id).await
    }

    pub async fn get_external_id_maps(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<Vec<MigrationExternalIdMap>, SipError> {
        ctx.require_permission("migration:read")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        repo.get_external_id_maps(ctx, job_id).await
    }

    pub async fn get_report(
        &self,
        ctx: &TenantContext,
        job_id: MigrationJobId,
    ) -> Result<serde_json::Value, SipError> {
        ctx.require_permission("migration:read")?;
        let repo = PgMigrationRepository::new(self.pool.clone());
        let job = repo
            .get_job(ctx, job_id)
            .await?
            .ok_or(SipError::Validation("Job not found".into()))?;
        let import_results = repo.get_import_results(ctx, job_id).await?;
        let validation_issues = repo.get_validation_issues(ctx, job_id).await?;
        let external_id_maps = repo.get_external_id_maps(ctx, job_id).await?;

        Ok(serde_json::json!({
            "job": {
                "id": job.id.to_string(),
                "name": job.name,
                "source_system": job.source_system,
                "source_object_type": job.source_object_type,
                "status": format!("{:?}", job.status),
                "source_record_count": job.source_record_count,
                "valid_record_count": job.valid_record_count,
                "imported_record_count": job.imported_record_count,
                "error_count": job.error_count,
            },
            "import_results": {
                "created": import_results.iter().filter(|r| r.action == "created").count(),
                "updated": import_results.iter().filter(|r| r.action == "updated").count(),
                "skipped": import_results.iter().filter(|r| r.action == "skipped").count(),
                "failed": import_results.iter().filter(|r| r.action == "failed").count(),
            },
            "validation_issues": {
                "total": validation_issues.len(),
                "errors": validation_issues.iter().filter(|i| i.severity == "error").count(),
                "warnings": validation_issues.iter().filter(|i| i.severity == "warning").count(),
            },
            "external_id_maps_count": external_id_maps.len(),
        }))
    }

    pub async fn stream_events(
        &self,
        ctx: &TenantContext,
        _job_id: MigrationJobId,
        _run_id: MigrationRunId,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<serde_json::Value>, SipError> {
        ctx.require_permission("migration:read")?;
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Ok(rx)
    }
}
