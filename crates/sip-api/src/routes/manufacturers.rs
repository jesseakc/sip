use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::{AssetModelService, ManufacturerService};
use sip_domain::{id::ManufacturerId, tenant::TenantContext};
use sip_infrastructure::repositories::{PgAssetModelRepository, PgManufacturerRepository};
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateManufacturerRequest {
    pub name: String,
    pub website: Option<String>,
    pub support_url: Option<String>,
}

pub async fn list_manufacturers(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgManufacturerRepository::new(state.pool.clone());
    let service = ManufacturerService::new(repo);
    match service.list(&ctx).await {
        Ok(items) => Ok(Json(json!({
            "data": items.iter().map(|m| json!({
                "id": m.id.to_string(),
                "name": m.name,
                "website": m.website,
                "support_url": m.support_url,
                "created_at": m.created_at,
                "updated_at": m.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_manufacturer(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let manufacturer_id = id.parse::<ManufacturerId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgManufacturerRepository::new(state.pool.clone());
    let service = ManufacturerService::new(repo);
    match service.get(&ctx, manufacturer_id).await {
        Ok(Some(m)) => Ok(Json(json!({
            "data": {
                "id": m.id.to_string(),
                "name": m.name,
                "website": m.website,
                "support_url": m.support_url,
                "created_at": m.created_at,
                "updated_at": m.updated_at,
            }
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "NOT_FOUND", "message": "Manufacturer not found"}})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn create_manufacturer(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateManufacturerRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgManufacturerRepository::new(state.pool.clone());
    let service = ManufacturerService::new(repo);
    match service
        .create(&ctx, req.name, req.website, req.support_url)
        .await
    {
        Ok(m) => Ok(Json(
            json!({"data": {"id": m.id.to_string(), "name": m.name}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn list_manufacturer_models(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let manufacturer_id = id.parse::<ManufacturerId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgAssetModelRepository::new(state.pool.clone());
    let service = AssetModelService::new(repo);
    match service.list(&ctx, Some(manufacturer_id)).await {
        Ok(items) => Ok(Json(json!({
            "data": items.iter().map(|m| json!({
                "id": m.id.to_string(),
                "name": m.name,
                "model_number": m.model_number,
                "manufacturer_id": m.manufacturer_id.to_string(),
                "asset_type_id": m.asset_type_id.to_string(),
                "lifecycle_status": format!("{:?}", m.lifecycle_status).to_uppercase(),
                "created_at": m.created_at,
                "updated_at": m.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}
