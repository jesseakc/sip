//! Offline sync contract for field clients.
//!
//! A technician works without a connection and reconciles later. Two properties
//! matter more than throughput:
//!
//! 1. **A retried write must not duplicate.** Creates therefore carry a
//!    client-generated UUID. Pushing the same id twice returns `already_applied`
//!    and the original row, instead of a second ticket.
//! 2. **A concurrent edit must not be silently lost.** The pull feed is ordered by
//!    `(updated_at, id)` so a page boundary cannot skip a row written in the same
//!    millisecond, and every record carries its `version` so the client can detect
//!    that the server moved on before it writes.
//!
//! The batch is never all-or-nothing: one rejected item does not discard the rest,
//! because losing nineteen good offline edits to one bad one is worse than
//! reporting a single failure.

use axum::{
    extract::{Extension, Json, Query, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::{CreateWorkOrderInput, WorkOrderService};
use sip_domain::{
    entity::work_order::{WorkOrderPriority, WorkOrderType},
    id::{AssetId, WorkOrderId},
    repository::WorkOrderRepository,
    tenant::TenantContext,
};
use sip_infrastructure::repositories::PgWorkOrderRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct PushRequest {
    pub operations: Vec<PushOperation>,
}

#[derive(Deserialize)]
pub struct PushOperation {
    /// `work_order` today; other entities are rejected per-item rather than
    /// ignoring the whole batch.
    pub entity: String,
    /// `create` today.
    pub op: String,
    /// Client-generated UUID. Reusing it is what makes the retry idempotent.
    pub id: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Deserialize)]
struct CreateWorkOrderPayload {
    asset_id: String,
    #[serde(default)]
    work_order_type: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    due_at: Option<DateTime<Utc>>,
    #[serde(default)]
    estimated_hours: Option<rust_decimal::Decimal>,
}

/// `POST /api/v1/sync/push`
///
/// Applies a batch of offline changes and reports the outcome of each one.
pub async fn push(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<PushRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let mut results = Vec::with_capacity(req.operations.len());

    for op in req.operations {
        results.push(apply_operation(&state, &ctx, op).await);
    }

    let applied = results
        .iter()
        .filter(|r| r["status"] == "applied" || r["status"] == "already_applied")
        .count();

    Ok(Json(json!({
        "data": results,
        "summary": {
            "total": results.len(),
            "applied": applied,
            "rejected": results.len() - applied,
        }
    })))
}

/// Apply one operation, always returning a result value — never an error, so one
/// bad record cannot abort the batch.
async fn apply_operation(
    state: &Arc<AppState>,
    ctx: &TenantContext,
    op: PushOperation,
) -> serde_json::Value {
    if op.entity != "work_order" {
        return rejected(&op.id, format!("unsupported entity: {}", op.entity));
    }
    if op.op != "create" {
        return rejected(&op.id, format!("unsupported op: {}", op.op));
    }

    let work_order_id = match op.id.parse::<WorkOrderId>() {
        Ok(id) => id,
        Err(e) => return rejected(&op.id, format!("id must be a UUID: {e}")),
    };

    let repo = PgWorkOrderRepository::new(state.pool.clone());

    // Already applied? Return the original instead of creating a second ticket.
    // This is the whole point of a client-generated id.
    match repo.get_work_order(ctx, work_order_id).await {
        Ok(Some(existing)) => return already_applied(work_order_id, &existing),
        Ok(None) => {}
        Err(e) => return rejected(&op.id, e.to_string()),
    }

    let payload: CreateWorkOrderPayload = match serde_json::from_value(op.payload) {
        Ok(p) => p,
        Err(e) => return rejected(&op.id, format!("invalid payload: {e}")),
    };

    let asset_id = match payload.asset_id.parse::<AssetId>() {
        Ok(id) => id,
        Err(e) => return rejected(&op.id, format!("invalid asset_id: {e}")),
    };

    let work_order_type = match payload
        .work_order_type
        .unwrap_or_default()
        .to_uppercase()
        .as_str()
    {
        "PREVENTIVE" => WorkOrderType::Preventive,
        "INSPECTION" => WorkOrderType::Inspection,
        "EMERGENCY" => WorkOrderType::Emergency,
        _ => WorkOrderType::Corrective,
    };
    let priority = match payload.priority.unwrap_or_default().to_uppercase().as_str() {
        "LOW" => WorkOrderPriority::Low,
        "HIGH" => WorkOrderPriority::High,
        "CRITICAL" => WorkOrderPriority::Critical,
        _ => WorkOrderPriority::Medium,
    };

    let service = WorkOrderService::new(PgWorkOrderRepository::new(state.pool.clone()));
    let input = CreateWorkOrderInput {
        id: Some(work_order_id),
        asset_id,
        work_order_type,
        priority,
        title: payload.title,
        description: payload.description.unwrap_or_default(),
        scheduled_start: None,
        scheduled_end: None,
        due_at: payload.due_at,
        estimated_hours: payload.estimated_hours,
    };

    match service.create(ctx, input).await {
        Ok(wo) => json!({
            "id": work_order_id.to_string(),
            "status": "applied",
            "version": wo.version,
            "display_number": wo.display_number,
        }),
        Err(e) => {
            // A racing duplicate would surface here as a unique violation. Re-check
            // so the client still gets a success rather than a spurious failure.
            if let Ok(Some(existing)) = repo.get_work_order(ctx, work_order_id).await {
                return already_applied(work_order_id, &existing);
            }
            rejected(&op.id, e.to_string())
        }
    }
}

fn already_applied(
    id: WorkOrderId,
    wo: &sip_domain::entity::work_order::WorkOrder,
) -> serde_json::Value {
    json!({
        "id": id.to_string(),
        "status": "already_applied",
        "version": wo.version,
        "display_number": wo.display_number,
    })
}

fn rejected(id: &str, reason: String) -> serde_json::Value {
    json!({
        "id": id,
        "status": "rejected",
        "reason": reason,
    })
}

#[derive(Deserialize)]
pub struct PullQuery {
    /// RFC 3339 timestamp. Absent means "from the beginning".
    pub since: Option<String>,
    /// Tie-breaker from the previous page's `next_cursor`.
    pub after_id: Option<String>,
    pub limit: Option<i64>,
}

/// `GET /api/v1/sync/pull?since=&after_id=&limit=`
///
/// Changes since a cursor, oldest first, including archived rows so the client can
/// remove them locally.
pub async fn pull(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Query(params): Query<PullQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let since = match params.since.as_deref() {
        Some(s) => match DateTime::parse_from_rfc3339(s) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(
                        json!({"error": {"code": "BAD_REQUEST", "message": format!("since must be RFC 3339: {e}")}}),
                    ),
                ))
            }
        },
        None => DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_else(Utc::now),
    };

    let after_id = match params.after_id.as_deref() {
        Some(s) => match s.parse::<uuid::Uuid>() {
            Ok(id) => Some(id),
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(
                        json!({"error": {"code": "BAD_REQUEST", "message": format!("after_id must be a UUID: {e}")}}),
                    ),
                ))
            }
        },
        None => None,
    };

    // Clamp rather than reject: a client asking for a huge page gets a bounded one.
    let limit = params.limit.unwrap_or(200).clamp(1, 500);

    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let orders = match repo.changed_since(&ctx, since, after_id, limit).await {
        Ok(rows) => rows,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
            ))
        }
    };

    let next_cursor = orders.last().map(|wo| {
        json!({
            // `Z`, not `+00:00`: a literal `+` in a query string decodes to a
            // space, so a client echoing this cursor back would get a 400.
            "since": wo.updated_at.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            "after_id": wo.id.to_string(),
        })
    });

    let data: Vec<serde_json::Value> = orders
        .iter()
        .map(|wo| {
            json!({
                "id": wo.id.to_string(),
                "asset_id": wo.asset_id.to_string(),
                "display_number": wo.display_number,
                "type": format!("{:?}", wo.work_order_type).to_uppercase(),
                "priority": format!("{:?}", wo.priority).to_uppercase(),
                "status": format!("{:?}", wo.status).to_uppercase(),
                "title": wo.title,
                "description": wo.description,
                "due_at": wo.due_at,
                "version": wo.version,
                "updated_at": wo.updated_at,
                // Tombstone: the client deletes its local copy when this is set.
                "archived": wo.archived_at.is_some(),
                "archived_at": wo.archived_at,
            })
        })
        .collect();

    Ok(Json(json!({
        "data": data,
        "next_cursor": next_cursor,
        "has_more": orders.len() as i64 == limit,
    })))
}
