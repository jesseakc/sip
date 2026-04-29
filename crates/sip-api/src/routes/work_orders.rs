use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::{WorkOrderService, CreateWorkOrderInput};
use sip_domain::{
    entity::work_order::{WorkOrderPriority, WorkOrderStatus, WorkOrderType, WorkOrderAssignment, AssigneeType, AssignmentRole, AssignmentStatus},
    entity::part::PartUsage,
    id::{WorkOrderId, WorkOrderAssignmentId, PartId, PartUsageId},
    tenant::TenantContext,
};
use sip_infrastructure::repositories::{PgWorkOrderRepository, PgActivityRepository, PgWorkOrderStatusHistoryRepository, PgWorkOrderAssignmentRepository, PgPartUsageRepository};
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateWorkOrderRequest {
    pub asset_id: String,
    pub work_order_type: String,
    pub priority: String,
    pub title: String,
    pub description: String,
    pub due_at: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_hours: Option<rust_decimal::Decimal>,
}

pub async fn list_work_orders(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.list(&ctx).await {
        Ok(wos) => Ok(Json(json!({
            "data": wos.iter().map(|wo| json!({
                "id": wo.id.to_string(),
                "display_number": wo.display_number,
                "title": wo.title,
                "status": format!("{:?}", wo.status).to_uppercase(),
                "priority": format!("{:?}", wo.priority).to_uppercase(),
                "type": format!("{:?}", wo.work_order_type).to_uppercase(),
                "asset_id": wo.asset_id.to_string(),
                "due_at": wo.due_at,
                "created_at": wo.created_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn get_work_order(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.get(&ctx, wo_id).await {
        Ok(Some(wo)) => Ok(Json(json!({
            "data": {
                "id": wo.id.to_string(),
                "display_number": wo.display_number,
                "title": wo.title,
                "description": wo.description,
                "status": format!("{:?}", wo.status).to_uppercase(),
                "priority": format!("{:?}", wo.priority).to_uppercase(),
                "type": format!("{:?}", wo.work_order_type).to_uppercase(),
                "asset_id": wo.asset_id.to_string(),
                "due_at": wo.due_at,
                "scheduled_start": wo.scheduled_start,
                "scheduled_end": wo.scheduled_end,
                "actual_start": wo.actual_start,
                "actual_end": wo.actual_end,
                "resolution_notes": wo.resolution_notes,
                "created_at": wo.created_at,
                "updated_at": wo.updated_at,
            }
        }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": {"code": "NOT_FOUND", "message": "Work order not found"}})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn create_work_order(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateWorkOrderRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_id: sip_domain::id::AssetId = req.asset_id.parse().map_err(|e: uuid::Error| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let wo_type = match req.work_order_type.to_uppercase().as_str() {
        "PREVENTIVE" => WorkOrderType::Preventive,
        "INSPECTION" => WorkOrderType::Inspection,
        "EMERGENCY" => WorkOrderType::Emergency,
        _ => WorkOrderType::Corrective,
    };
    let priority = match req.priority.to_uppercase().as_str() {
        "LOW" => WorkOrderPriority::Low,
        "HIGH" => WorkOrderPriority::High,
        "CRITICAL" => WorkOrderPriority::Critical,
        _ => WorkOrderPriority::Medium,
    };
    let input = CreateWorkOrderInput {
        asset_id,
        work_order_type: wo_type,
        priority,
        title: req.title,
        description: req.description,
        scheduled_start: None,
        scheduled_end: None,
        due_at: req.due_at,
        estimated_hours: req.estimated_hours,
    };
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.create(&ctx, input).await {
        Ok(wo) => Ok(Json(json!({"data": {"id": wo.id.to_string(), "display_number": wo.display_number}}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn transition_work_order(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let status_str = req.get("status").and_then(|v| v.as_str()).ok_or((StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": "status required"}}))))?;
    let status = match status_str.to_uppercase().as_str() {
        "DRAFT" => WorkOrderStatus::Draft,
        "OPEN" => WorkOrderStatus::Open,
        "ASSIGNED" => WorkOrderStatus::Assigned,
        "ACCEPTED" => WorkOrderStatus::Accepted,
        "IN_PROGRESS" => WorkOrderStatus::InProgress,
        "ON_HOLD" => WorkOrderStatus::OnHold,
        "COMPLETED" => WorkOrderStatus::Completed,
        "REVIEWED" => WorkOrderStatus::Reviewed,
        "CLOSED" => WorkOrderStatus::Closed,
        "CANCELLED" => WorkOrderStatus::Cancelled,
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": "invalid status"}})))),
    };
    let resolution_notes = req.get("resolution_notes").and_then(|v| v.as_str()).map(|s| s.to_string());
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.transition(&ctx, wo_id, status, resolution_notes).await {
        Ok(wo) => Ok(Json(json!({"data": {"id": wo.id.to_string(), "status": format!("{:?}", wo.status).to_uppercase()}}))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})))),
    }
}

macro_rules! lifecycle_handler {
    ($name:ident, $method:ident, $status:expr) => {
        pub async fn $name(
            State(state): State<Arc<AppState>>,
            Extension(ctx): Extension<TenantContext>,
            Path(id): Path<String>,
        ) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
            let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
            let repo = PgWorkOrderRepository::new(state.pool.clone());
            let activity_repo = PgActivityRepository::new(state.pool.clone());
            let history_repo = PgWorkOrderStatusHistoryRepository::new(state.pool.clone());
            let service = WorkOrderService::new(repo);
            match service.$method(&ctx, wo_id, &activity_repo, &history_repo).await {
                Ok(wo) => Ok(Json(json!({"data": {"id": wo.id.to_string(), "status": format!("{:?}", wo.status).to_uppercase()}}))),
                Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})))),
            }
        }
    };
}

lifecycle_handler!(publish_work_order, publish, WorkOrderStatus::Open);
lifecycle_handler!(start_work_order, start, WorkOrderStatus::InProgress);
lifecycle_handler!(hold_work_order, hold, WorkOrderStatus::OnHold);
lifecycle_handler!(resume_work_order, resume, WorkOrderStatus::InProgress);
lifecycle_handler!(review_work_order, review, WorkOrderStatus::Reviewed);
lifecycle_handler!(close_work_order, close, WorkOrderStatus::Closed);
lifecycle_handler!(cancel_work_order, cancel, WorkOrderStatus::Cancelled);

pub async fn complete_work_order(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let resolution_notes = req.get("resolution_notes").and_then(|v| v.as_str()).map(|s| s.to_string());
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let activity_repo = PgActivityRepository::new(state.pool.clone());
    let history_repo = PgWorkOrderStatusHistoryRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.complete(&ctx, wo_id, resolution_notes, &activity_repo, &history_repo).await {
        Ok(wo) => Ok(Json(json!({"data": {"id": wo.id.to_string(), "status": format!("{:?}", wo.status).to_uppercase()}}))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})))),
    }
}

pub async fn archive_work_order(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let activity_repo = PgActivityRepository::new(state.pool.clone());
    let history_repo = PgWorkOrderStatusHistoryRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.archive(&ctx, wo_id, &activity_repo, &history_repo).await {
        Ok(wo) => Ok(Json(json!({"data": {"id": wo.id.to_string(), "status": format!("{:?}", wo.status).to_uppercase()}}))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})))),
    }
}

#[derive(Deserialize)]
pub struct CreateAssignmentRequest {
    pub assignee_type: String,
    pub assignee_id: String,
    pub role: String,
}

pub async fn list_assignments(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let assignment_repo = PgWorkOrderAssignmentRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.list_assignments(&ctx, wo_id, &assignment_repo).await {
        Ok(assignments) => Ok(Json(json!({
            "data": assignments.iter().map(|a| json!({
                "id": a.id.to_string(),
                "assignee_type": format!("{:?}", a.assignee_type).to_uppercase(),
                "assignee_id": a.assignee_id.to_string(),
                "role": format!("{:?}", a.role).to_uppercase(),
                "status": format!("{:?}", a.status).to_uppercase(),
                "assigned_at": a.assigned_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn create_assignment(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<CreateAssignmentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let assignee_id: sip_domain::id::UserId = req.assignee_id.parse().map_err(|e: uuid::Error| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let assignee_type = match req.assignee_type.to_uppercase().as_str() {
        "USER" => AssigneeType::User,
        "TEAM" => AssigneeType::Team,
        "VENDOR" => AssigneeType::Vendor,
        _ => AssigneeType::AiAgent,
    };
    let role = match req.role.to_uppercase().as_str() {
        "PRIMARY" => AssignmentRole::Primary,
        "SECONDARY" => AssignmentRole::Secondary,
        "OBSERVER" => AssignmentRole::Observer,
        "APPROVER" => AssignmentRole::Approver,
        "DISPATCHED_TECH" => AssignmentRole::DispatchedTech,
        _ => AssignmentRole::RemoteSupport,
    };
    let assignment = WorkOrderAssignment {
        id: sip_domain::id::WorkOrderAssignmentId::new(),
        organization_id: ctx.organization_id,
        work_order_id: wo_id,
        assignee_type,
        assignee_id,
        role,
        assigned_at: chrono::Utc::now(),
        assigned_by: ctx.user_id.ok_or((StatusCode::UNAUTHORIZED, Json(json!({"error": {"code": "UNAUTHORIZED", "message": "User ID required"}}))))?,
        accepted_at: None,
        removed_at: None,
        status: AssignmentStatus::Assigned,
    };
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let assignment_repo = PgWorkOrderAssignmentRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.create_assignment(&ctx, assignment, &assignment_repo).await {
        Ok(a) => Ok(Json(json!({"data": {"id": a.id.to_string()}}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn delete_assignment(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path((id, assignment_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let _wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let a_id = assignment_id.parse::<WorkOrderAssignmentId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let assignment_repo = PgWorkOrderAssignmentRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.delete_assignment(&ctx, a_id, &assignment_repo).await {
        Ok(()) => Ok(Json(json!({"data": null}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

#[derive(Deserialize)]
pub struct CreatePartUsageRequest {
    pub part_id: String,
    pub quantity: rust_decimal::Decimal,
}

pub async fn list_parts(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let part_repo = PgPartUsageRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.list_parts(&ctx, wo_id, &part_repo).await {
        Ok(parts) => Ok(Json(json!({
            "data": parts.iter().map(|p| json!({
                "id": p.id.to_string(),
                "part_id": p.part_id.to_string(),
                "quantity": p.quantity,
                "used_by_id": p.used_by_id.to_string(),
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn create_part_usage(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<CreatePartUsageRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let part_id: PartId = req.part_id.parse().map_err(|e: uuid::Error| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let usage = PartUsage {
        id: PartUsageId::new(),
        work_order_id: wo_id,
        part_id,
        quantity: req.quantity,
        used_by_id: ctx.user_id.ok_or((StatusCode::UNAUTHORIZED, Json(json!({"error": {"code": "UNAUTHORIZED", "message": "User ID required"}}))))?,
    };
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let part_repo = PgPartUsageRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.add_part(&ctx, usage, &part_repo).await {
        Ok(p) => Ok(Json(json!({"data": {"id": p.id.to_string()}}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn reopen_work_order(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_id = id.parse::<WorkOrderId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let resume = body.get("resume_in_progress").and_then(|v| v.as_bool()).unwrap_or(false);
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let activity_repo = PgActivityRepository::new(state.pool.clone());
    let history_repo = PgWorkOrderStatusHistoryRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.reopen(&ctx, wo_id, &activity_repo, &history_repo, resume).await {
        Ok(wo) => Ok(Json(json!({"data": {"id": wo.id.to_string(), "status": format!("{:?}", wo.status).to_uppercase()}}))),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})))),
    }
}

#[derive(Deserialize)]
pub struct UpdateAssignmentRequest {
    pub status: String,
}

pub async fn update_assignment(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path((_wo_id, assignment_id)): Path<(String, String)>,
    Json(req): Json<UpdateAssignmentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let a_id = assignment_id.parse::<WorkOrderAssignmentId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let status = match req.status.to_uppercase().as_str() {
        "ACCEPTED" => AssignmentStatus::Accepted,
        "DECLINED" => AssignmentStatus::Declined,
        "REMOVED" => AssignmentStatus::Removed,
        "COMPLETED" => AssignmentStatus::Completed,
        _ => AssignmentStatus::Assigned,
    };
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let assignment_repo = PgWorkOrderAssignmentRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.update_assignment(&ctx, a_id, status, &assignment_repo).await {
        Ok(a) => Ok(Json(json!({"data": {"id": a.id.to_string(), "status": format!("{:?}", a.status).to_uppercase()}}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}
