use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::AssetTypeService;
use sip_domain::{id::AssetTypeId, tenant::TenantContext};
use sip_infrastructure::repositories::PgAssetTypeRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateAssetTypeRequest {
    pub name: String,
    pub category: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateAssetTypeRequest {
    pub name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
}

pub async fn list_asset_types(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgAssetTypeRepository::new(state.pool.clone());
    let service = AssetTypeService::new(repo);
    match service.list(&ctx).await {
        Ok(items) => Ok(Json(json!({
            "data": items.iter().map(|at| json!({
                "id": at.id.to_string(),
                "name": at.name,
                "category": at.category,
                "description": at.description,
                "icon": at.icon,
                "is_system": at.is_system,
                "created_at": at.created_at,
                "updated_at": at.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_asset_type(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_type_id = id.parse::<AssetTypeId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgAssetTypeRepository::new(state.pool.clone());
    let service = AssetTypeService::new(repo);
    match service.get(&ctx, asset_type_id).await {
        Ok(Some(at)) => Ok(Json(json!({
            "data": {
                "id": at.id.to_string(),
                "name": at.name,
                "category": at.category,
                "description": at.description,
                "schema": at.schema,
                "default_pm_schedules": at.default_pm_schedules,
                "default_inspection_template": at.default_inspection_template,
                "icon": at.icon,
                "is_system": at.is_system,
                "created_at": at.created_at,
                "updated_at": at.updated_at,
            }
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "NOT_FOUND", "message": "Asset type not found"}})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn create_asset_type(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateAssetTypeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgAssetTypeRepository::new(state.pool.clone());
    let service = AssetTypeService::new(repo);
    match service
        .create(&ctx, req.name, req.category, req.description)
        .await
    {
        Ok(at) => Ok(Json(
            json!({"data": {"id": at.id.to_string(), "name": at.name}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn update_asset_type(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAssetTypeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let asset_type_id = id.parse::<AssetTypeId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgAssetTypeRepository::new(state.pool.clone());
    let service = AssetTypeService::new(repo);
    match service
        .update(&ctx, asset_type_id, req.name, req.category, req.description)
        .await
    {
        Ok(at) => Ok(Json(
            json!({"data": {"id": at.id.to_string(), "name": at.name}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}
