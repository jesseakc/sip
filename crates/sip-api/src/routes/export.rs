use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::{AssetService, LocationService, UserService, WorkOrderService};
use sip_domain::tenant::TenantContext;
use sip_infrastructure::repositories::{
    PgAssetRepository, PgLocationRepository, PgUserRepository, PgWorkOrderRepository,
};
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct ExportWorkOrdersRequest {
    pub format: Option<String>,
}

pub async fn export_work_orders(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(_req): Json<ExportWorkOrdersRequest>,
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
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn export_all(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let wo_repo = PgWorkOrderRepository::new(state.pool.clone());
    let wo_service = WorkOrderService::new(wo_repo);
    let asset_repo = PgAssetRepository::new(state.pool.clone());
    let asset_service = AssetService::new(asset_repo);
    let loc_repo = PgLocationRepository::new(state.pool.clone());
    let loc_service = LocationService::new(loc_repo);
    let user_repo = PgUserRepository::new(state.pool.clone());
    let user_service = UserService::new(user_repo);

    let wos = wo_service.list(&ctx).await.unwrap_or_default();
    let assets = asset_service.list(&ctx).await.unwrap_or_default();
    let locations = loc_service.list(&ctx).await.unwrap_or_default();
    let users = user_service.list(&ctx).await.unwrap_or_default();

    Ok(Json(json!({
        "data": {
            "work_orders": wos.iter().map(|wo| json!({
                "id": wo.id.to_string(),
                "display_number": wo.display_number,
                "title": wo.title,
                "status": format!("{:?}", wo.status).to_uppercase(),
                "priority": format!("{:?}", wo.priority).to_uppercase(),
                "asset_id": wo.asset_id.to_string(),
                "created_at": wo.created_at,
            })).collect::<Vec<_>>(),
            "assets": assets.iter().map(|a| json!({
                "id": a.id.to_string(),
                "name": a.name,
                "status": format!("{:?}", a.status).to_uppercase(),
                "asset_type_id": a.asset_type_id.to_string(),
                "location_id": a.location_id.map(|id| id.to_string()),
            })).collect::<Vec<_>>(),
            "locations": locations.iter().map(|l| json!({
                "id": l.id.to_string(),
                "name": l.name,
                "type": format!("{:?}", l.location_type).to_uppercase(),
                "parent_id": l.parent_id.map(|id| id.to_string()),
            })).collect::<Vec<_>>(),
            "users": users.iter().map(|u| json!({
                "id": u.id.to_string(),
                "email": u.email,
                "name": u.name,
                "role": format!("{:?}", u.role).to_uppercase(),
            })).collect::<Vec<_>>(),
        }
    })))
}
