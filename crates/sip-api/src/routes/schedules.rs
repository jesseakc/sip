use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::{CreateScheduleInput, ScheduleService, UpdateScheduleInput};
use sip_domain::{
    entity::schedule::ScheduleTriggerType,
    id::{AssetId, ScheduleId},
    tenant::TenantContext,
};
use sip_infrastructure::repositories::PgScheduleRepository;
use sip_scheduler::due_date::calculate_next_due;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateScheduleRequest {
    pub asset_id: String,
    pub name: String,
    pub trigger_type: String,
    pub cron_expression: Option<String>,
    pub trigger_config: Option<serde_json::Value>,
    pub work_order_template: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateScheduleRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub cron_expression: Option<String>,
    pub trigger_config: Option<serde_json::Value>,
    pub work_order_template: Option<serde_json::Value>,
}

pub async fn list_schedules(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgScheduleRepository::new(state.pool.clone());
    let service = ScheduleService::new(repo);
    match service.list(&ctx).await {
        Ok(schedules) => Ok(Json(json!({
            "data": schedules.iter().map(|s| json!({
                "id": s.id.to_string(),
                "asset_id": s.asset_id.to_string(),
                "name": s.name,
                "trigger_type": format!("{:?}", s.trigger_type).to_uppercase(),
                "trigger_config": s.trigger_config,
                "work_order_template": s.work_order_template,
                "next_due": s.next_due,
                "last_triggered": s.last_triggered,
                "enabled": s.enabled,
                "archived_at": s.archived_at,
                "created_at": s.created_at,
                "updated_at": s.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let schedule_id = id.parse::<ScheduleId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgScheduleRepository::new(state.pool.clone());
    let service = ScheduleService::new(repo);
    match service.get(&ctx, schedule_id).await {
        Ok(Some(s)) => Ok(Json(json!({
            "data": {
                "id": s.id.to_string(),
                "asset_id": s.asset_id.to_string(),
                "name": s.name,
                "trigger_type": format!("{:?}", s.trigger_type).to_uppercase(),
                "trigger_config": s.trigger_config,
                "work_order_template": s.work_order_template,
                "next_due": s.next_due,
                "last_triggered": s.last_triggered,
                "enabled": s.enabled,
                "archived_at": s.archived_at,
                "created_at": s.created_at,
                "updated_at": s.updated_at,
            }
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "NOT_FOUND", "message": "Schedule not found"}})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn create_schedule(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateScheduleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_id: AssetId = req.asset_id.parse().map_err(|e: uuid::Error| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let trigger_type = match req.trigger_type.to_uppercase().as_str() {
        "METER" => ScheduleTriggerType::Meter,
        _ => ScheduleTriggerType::Cron,
    };
    let next_due = req
        .cron_expression
        .as_ref()
        .and_then(|expr| calculate_next_due(expr, chrono::Utc::now()));
    let input = CreateScheduleInput {
        asset_id,
        name: req.name,
        trigger_type,
        trigger_config: req.trigger_config,
        work_order_template: req.work_order_template,
        next_due,
    };
    let repo = PgScheduleRepository::new(state.pool.clone());
    let service = ScheduleService::new(repo);
    match service.create(&ctx, input).await {
        Ok(schedule) => Ok(Json(
            json!({"data": {"id": schedule.id.to_string(), "name": schedule.name}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn update_schedule(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<UpdateScheduleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let schedule_id = id.parse::<ScheduleId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let next_due = req
        .cron_expression
        .as_ref()
        .and_then(|expr| calculate_next_due(expr, chrono::Utc::now()));
    let input = UpdateScheduleInput {
        name: req.name,
        enabled: req.enabled,
        trigger_config: req.trigger_config,
        work_order_template: req.work_order_template,
        next_due,
    };
    let repo = PgScheduleRepository::new(state.pool.clone());
    let service = ScheduleService::new(repo);
    match service.update(&ctx, schedule_id, input).await {
        Ok(schedule) => Ok(Json(
            json!({"data": {"id": schedule.id.to_string(), "name": schedule.name, "enabled": schedule.enabled}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn archive_schedule(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let schedule_id = id.parse::<ScheduleId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgScheduleRepository::new(state.pool.clone());
    let service = ScheduleService::new(repo);
    match service.archive(&ctx, schedule_id).await {
        Ok(schedule) => Ok(Json(
            json!({"data": {"id": schedule.id.to_string(), "archived_at": schedule.archived_at}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}
