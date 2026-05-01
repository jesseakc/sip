use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::LocationService;
use sip_domain::{entity::location::LocationType, id::LocationId, tenant::TenantContext};
use sip_infrastructure::repositories::PgLocationRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateLocationRequest {
    pub parent_id: Option<String>,
    pub name: String,
    pub location_type: String,
}

#[derive(Deserialize)]
pub struct UpdateLocationRequest {
    pub name: Option<String>,
    pub location_type: Option<String>,
}

pub async fn list_locations(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgLocationRepository::new(state.pool.clone());
    let service = LocationService::new(repo);
    match service.list(&ctx).await {
        Ok(locations) => Ok(Json(json!({
            "data": locations.iter().map(|l| json!({
                "id": l.id.to_string(),
                "name": l.name,
                "type": format!("{:?}", l.location_type).to_uppercase(),
                "parent_id": l.parent_id.map(|id| id.to_string()),
                "created_at": l.created_at,
                "updated_at": l.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_location(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let location_id = id.parse::<LocationId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgLocationRepository::new(state.pool.clone());
    let service = LocationService::new(repo);
    match service.get(&ctx, location_id).await {
        Ok(Some(location)) => Ok(Json(json!({
            "data": {
                "id": location.id.to_string(),
                "name": location.name,
                "type": format!("{:?}", location.location_type).to_uppercase(),
                "parent_id": location.parent_id.map(|id| id.to_string()),
                "metadata": location.metadata,
                "created_at": location.created_at,
                "updated_at": location.updated_at,
            }
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "NOT_FOUND", "message": "Location not found"}})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn create_location(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let parent_id = req
        .parent_id
        .map(|s| s.parse::<LocationId>())
        .transpose()
        .map_err(|e: uuid::Error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
            )
        })?;
    let location_type = match req.location_type.to_uppercase().as_str() {
        "SITE" => LocationType::Site,
        "BUILDING" => LocationType::Building,
        "FLOOR" => LocationType::Floor,
        "ROOM" => LocationType::Room,
        "AREA" => LocationType::Area,
        _ => LocationType::Other,
    };
    let repo = PgLocationRepository::new(state.pool.clone());
    let service = LocationService::new(repo);
    match service
        .create(&ctx, parent_id, req.name, location_type)
        .await
    {
        Ok(location) => Ok(Json(
            json!({"data": {"id": location.id.to_string(), "name": location.name}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn update_location(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<UpdateLocationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let location_id = id.parse::<LocationId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let location_type = req.location_type.map(|t| match t.to_uppercase().as_str() {
        "SITE" => LocationType::Site,
        "BUILDING" => LocationType::Building,
        "FLOOR" => LocationType::Floor,
        "ROOM" => LocationType::Room,
        "AREA" => LocationType::Area,
        _ => LocationType::Other,
    });
    let repo = PgLocationRepository::new(state.pool.clone());
    let service = LocationService::new(repo);
    match service
        .update(&ctx, location_id, req.name, location_type)
        .await
    {
        Ok(location) => Ok(Json(
            json!({"data": {"id": location.id.to_string(), "name": location.name}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn list_location_children(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let location_id = id.parse::<LocationId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgLocationRepository::new(state.pool.clone());
    let service = LocationService::new(repo);
    match service.list_children(&ctx, location_id).await {
        Ok(children) => Ok(Json(json!({
            "data": children.iter().map(|l| json!({
                "id": l.id.to_string(),
                "name": l.name,
                "type": format!("{:?}", l.location_type).to_uppercase(),
                "parent_id": l.parent_id.map(|id| id.to_string()),
                "created_at": l.created_at,
                "updated_at": l.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn list_location_assets(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let location_id = id.parse::<LocationId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgLocationRepository::new(state.pool.clone());
    let service = LocationService::new(repo);
    match service.list_assets(&ctx, location_id).await {
        Ok(assets) => Ok(Json(json!({
            "data": assets.iter().map(|a| json!({
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
