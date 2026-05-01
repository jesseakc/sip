use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sip_application::services::{AssetService, CreateAssetInput, WorkOrderService};
use sip_domain::{
    entity::asset::{AssetStatus, Criticality},
    id::AssetId,
    tenant::TenantContext,
};
use sip_infrastructure::repositories::{PgAssetRepository, PgWorkOrderRepository};
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateAssetRequest {
    pub asset_type_id: String,
    pub model_id: Option<String>,
    pub location_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub criticality: String,
    pub attributes: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize)]
pub struct AssetResponse {
    pub id: String,
    pub name: String,
    pub status: String,
}

pub async fn list_assets(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgAssetRepository::new(state.pool.clone());
    let service = AssetService::new(repo);
    match service.list(&ctx).await {
        Ok(assets) => Ok(Json(json!({
            "data": assets.iter().map(|a| json!({
                "id": a.id.to_string(),
                "name": a.name,
                "status": format!("{:?}", a.status).to_uppercase(),
                "criticality": format!("{:?}", a.criticality).to_uppercase(),
                "asset_type_id": a.asset_type_id.to_string(),
                "location_id": a.location_id.map(|id| id.to_string()),
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_asset(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_id = id.parse::<AssetId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgAssetRepository::new(state.pool.clone());
    let service = AssetService::new(repo);
    match service.get(&ctx, asset_id).await {
        Ok(Some(asset)) => Ok(Json(json!({
            "id": asset.id.to_string(),
            "name": asset.name,
            "description": asset.description,
            "status": format!("{:?}", asset.status).to_uppercase(),
            "criticality": format!("{:?}", asset.criticality).to_uppercase(),
            "asset_type_id": asset.asset_type_id.to_string(),
            "location_id": asset.location_id.map(|id| id.to_string()),
            "model_id": asset.model_id.map(|id| id.to_string()),
            "serial_number": asset.serial_number,
            "tags": asset.tags,
            "attributes": asset.attributes,
            "created_at": asset.created_at,
            "updated_at": asset.updated_at,
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "NOT_FOUND", "message": "Asset not found"}})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn create_asset(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateAssetRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_type_id: sip_domain::id::AssetTypeId =
        req.asset_type_id.parse().map_err(|e: uuid::Error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
            )
        })?;
    let model_id: Option<sip_domain::id::AssetModelId> = req
        .model_id
        .map(|s| s.parse())
        .transpose()
        .map_err(|e: uuid::Error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
            )
        })?;
    let location_id: Option<sip_domain::id::LocationId> = req
        .location_id
        .map(|s| s.parse())
        .transpose()
        .map_err(|e: uuid::Error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
            )
        })?;
    let criticality = match req.criticality.as_str() {
        "LOW" => Criticality::Low,
        "HIGH" => Criticality::High,
        "CRITICAL" => Criticality::Critical,
        _ => Criticality::Medium,
    };
    let input = CreateAssetInput {
        asset_type_id,
        model_id,
        location_id,
        name: req.name,
        description: req.description,
        serial_number: req.serial_number,
        criticality,
        attributes: req.attributes,
        tags: req.tags,
    };
    let repo = PgAssetRepository::new(state.pool.clone());
    let service = AssetService::new(repo);
    match service.create(&ctx, input).await {
        Ok(asset) => Ok(Json(
            json!({"id": asset.id.to_string(), "name": asset.name}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn update_asset_status(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_id = id.parse::<AssetId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let status_str = req.get("status").and_then(|v| v.as_str()).ok_or((
        StatusCode::BAD_REQUEST,
        Json(json!({"error": {"code": "BAD_REQUEST", "message": "status required"}})),
    ))?;
    let status = match status_str.to_uppercase().as_str() {
        "OPERATIONAL" => AssetStatus::Operational,
        "DEGRADED" => AssetStatus::Degraded,
        "DOWN" => AssetStatus::Down,
        "MAINTENANCE" => AssetStatus::Maintenance,
        "RETIRED" => AssetStatus::Retired,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({"error": {"code": "BAD_REQUEST", "message": "invalid status"}})),
            ))
        }
    };
    let repo = PgAssetRepository::new(state.pool.clone());
    let service = AssetService::new(repo);
    match service.update_status(&ctx, asset_id, status).await {
        Ok(asset) => Ok(Json(
            json!({"id": asset.id.to_string(), "status": format!("{:?}", asset.status).to_uppercase()}),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )),
    }
}

pub async fn list_asset_work_orders(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_id = id.parse::<AssetId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgWorkOrderRepository::new(state.pool.clone());
    let service = WorkOrderService::new(repo);
    match service.list_by_asset(&ctx, asset_id).await {
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
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn archive_asset(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_id = id.parse::<AssetId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgAssetRepository::new(state.pool.clone());
    let service = AssetService::new(repo);
    match service.archive(&ctx, asset_id).await {
        Ok(asset) => Ok(Json(
            json!({"data": {"id": asset.id.to_string(), "status": "RETIRED"}}),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )),
    }
}

pub async fn list_asset_children(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_id = id.parse::<AssetId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgAssetRepository::new(state.pool.clone());
    let service = AssetService::new(repo);
    match service.list_children(&ctx, asset_id).await {
        Ok(children) => Ok(Json(json!({
            "data": children.iter().map(|a| json!({
                "id": a.id.to_string(),
                "name": a.name,
                "status": format!("{:?}", a.status).to_uppercase(),
                "asset_type_id": a.asset_type_id.to_string(),
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}
