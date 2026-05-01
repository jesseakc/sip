use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::{CreatePartInput, PartService, UpdatePartInput};
use sip_domain::{id::PartId, tenant::TenantContext};
use sip_infrastructure::repositories::PgPartRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreatePartRequest {
    pub name: String,
    pub part_number: Option<String>,
    pub description: Option<String>,
    pub quantity_on_hand: Option<rust_decimal::Decimal>,
    pub quantity_minimum: Option<rust_decimal::Decimal>,
    pub unit: Option<String>,
    pub unit_cost: Option<rust_decimal::Decimal>,
    pub storage_location: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePartRequest {
    pub name: Option<String>,
    pub part_number: Option<String>,
    pub description: Option<String>,
    pub quantity_on_hand: Option<rust_decimal::Decimal>,
    pub quantity_minimum: Option<rust_decimal::Decimal>,
    pub unit: Option<String>,
    pub unit_cost: Option<rust_decimal::Decimal>,
    pub storage_location: Option<String>,
}

pub async fn list_parts(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgPartRepository::new(state.pool.clone());
    let service = PartService::new(repo);
    match service.list(&ctx).await {
        Ok(parts) => Ok(Json(json!({
            "data": parts.iter().map(|p| json!({
                "id": p.id.to_string(),
                "name": p.name,
                "part_number": p.part_number,
                "description": p.description,
                "quantity_on_hand": p.quantity_on_hand,
                "quantity_minimum": p.quantity_minimum,
                "unit": p.unit,
                "unit_cost": p.unit_cost,
                "storage_location": p.storage_location,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn create_part(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreatePartRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let input = CreatePartInput {
        name: req.name,
        part_number: req.part_number,
        description: req.description,
        quantity_on_hand: req.quantity_on_hand.unwrap_or(rust_decimal::Decimal::ZERO),
        quantity_minimum: req.quantity_minimum,
        unit: req.unit.unwrap_or_else(|| "ea".to_string()),
        unit_cost: req.unit_cost,
        storage_location: req.storage_location,
    };
    let repo = PgPartRepository::new(state.pool.clone());
    let service = PartService::new(repo);
    match service.create(&ctx, input).await {
        Ok(part) => Ok(Json(
            json!({"data": {"id": part.id.to_string(), "name": part.name}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_part(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let part_id = id.parse::<PartId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;

    let repo = PgPartRepository::new(state.pool.clone());
    let service = PartService::new(repo);
    match service.get(&ctx, part_id).await {
        Ok(Some(part)) => Ok(Json(json!({
            "data": {
                "id": part.id.to_string(),
                "name": part.name,
                "part_number": part.part_number,
                "description": part.description,
                "quantity_on_hand": part.quantity_on_hand,
                "quantity_minimum": part.quantity_minimum,
                "unit": part.unit,
                "unit_cost": part.unit_cost,
                "storage_location": part.storage_location,
                "created_at": part.created_at,
                "updated_at": part.updated_at,
            }
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "NOT_FOUND", "message": "Part not found"}})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn update_part(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePartRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let part_id = id.parse::<PartId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;

    let input = UpdatePartInput {
        name: req.name,
        part_number: req.part_number,
        description: req.description,
        quantity_on_hand: req.quantity_on_hand,
        quantity_minimum: req.quantity_minimum,
        unit: req.unit,
        unit_cost: req.unit_cost,
        storage_location: req.storage_location,
    };
    let repo = PgPartRepository::new(state.pool.clone());
    let service = PartService::new(repo);
    match service.update(&ctx, part_id, input).await {
        Ok(part) => Ok(Json(
            json!({"data": {"id": part.id.to_string(), "name": part.name}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}
