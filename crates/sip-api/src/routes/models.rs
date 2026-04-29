use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::AssetModelService;
use sip_domain::{
    id::{AssetModelId, ManufacturerId},
    tenant::TenantContext,
};
use sip_infrastructure::repositories::PgAssetModelRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct ListModelsQuery {
    pub manufacturer_id: Option<String>,
}

pub async fn list_models(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Query(query): Query<ListModelsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let manufacturer_id = query.manufacturer_id.map(|s| s.parse::<ManufacturerId>()).transpose().map_err(|e: uuid::Error| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgAssetModelRepository::new(state.pool.clone());
    let service = AssetModelService::new(repo);
    match service.list(&ctx, manufacturer_id).await {
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
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn get_model(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let model_id = id.parse::<AssetModelId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgAssetModelRepository::new(state.pool.clone());
    let service = AssetModelService::new(repo);
    match service.get(&ctx, model_id).await {
        Ok(Some(m)) => Ok(Json(json!({
            "data": {
                "id": m.id.to_string(),
                "name": m.name,
                "model_number": m.model_number,
                "manufacturer_id": m.manufacturer_id.to_string(),
                "asset_type_id": m.asset_type_id.to_string(),
                "revision": m.revision,
                "lifecycle_status": format!("{:?}", m.lifecycle_status).to_uppercase(),
                "documentation_url": m.documentation_url,
                "default_attributes": m.default_attributes,
                "metadata": m.metadata,
                "created_at": m.created_at,
                "updated_at": m.updated_at,
            }
        }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": {"code": "NOT_FOUND", "message": "Model not found"}})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

#[derive(Deserialize)]
pub struct CreateModelRequest {
    pub manufacturer_id: String,
    pub name: String,
    pub model_number: String,
    pub asset_type_id: String,
}

pub async fn create_model(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateModelRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let manufacturer_id: ManufacturerId = req.manufacturer_id.parse().map_err(|e: uuid::Error| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let asset_type_id: sip_domain::id::AssetTypeId = req.asset_type_id.parse().map_err(|e: uuid::Error| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgAssetModelRepository::new(state.pool.clone());
    let service = AssetModelService::new(repo);
    match service.create(&ctx, manufacturer_id, req.name, req.model_number, asset_type_id).await {
        Ok(m) => Ok(Json(json!({"data": {"id": m.id.to_string(), "name": m.name}}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}
