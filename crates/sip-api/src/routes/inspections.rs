use axum::{
    extract::{Extension, Json, Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::{InspectionService, UpdateChecklistItemInput};
use sip_domain::{
    entity::inspection::ChecklistResult,
    id::{InspectionId, InspectionChecklistItemId, WorkOrderId},
    tenant::TenantContext,
};
use sip_infrastructure::repositories::PgInspectionRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct ListInspectionsQuery {
    pub work_order_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateChecklistItem {
    pub id: String,
    pub result: Option<String>,
    pub actual_value: Option<String>,
    pub finding: Option<String>,
    pub photo_url: Option<String>,
}

pub async fn list_inspections(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Query(query): Query<ListInspectionsQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let work_order_id = query.work_order_id
        .map(|s| s.parse::<WorkOrderId>())
        .transpose()
        .map_err(|e: uuid::Error| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;

    let repo = PgInspectionRepository::new(state.pool.clone());
    let service = InspectionService::new(repo);
    match service.list(&ctx, work_order_id).await {
        Ok(inspections) => Ok(Json(json!({
            "data": inspections.iter().map(|i| json!({
                "id": i.id.to_string(),
                "work_order_id": i.work_order_id.to_string(),
                "template_name": i.template_name,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn get_inspection(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let inspection_id = id.parse::<InspectionId>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;

    let repo = PgInspectionRepository::new(state.pool.clone());
    let service = InspectionService::new(repo);
    match service.get(&ctx, inspection_id).await {
        Ok(Some(inspection)) => Ok(Json(json!({
            "data": {
                "id": inspection.id.to_string(),
                "work_order_id": inspection.work_order_id.to_string(),
                "template_name": inspection.template_name,
            }
        }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": {"code": "NOT_FOUND", "message": "Inspection not found"}})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn update_checklist_items(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<Vec<UpdateChecklistItem>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let inspection_id = id.parse::<InspectionId>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;

    let items: Vec<UpdateChecklistItemInput> = req.iter().map(|item| {
        let item_id = item.id.parse::<InspectionChecklistItemId>()
            .map_err(|e: uuid::Error| format!("invalid item id: {}", e))?;
        let result = item.result.as_ref().map(|s| match s.to_uppercase().as_str() {
            "PASS" | "PASS_FAIL" => ChecklistResult::Pass,
            "N_A" | "NOT_APPLICABLE" | "NOTAPPLICABLE" => ChecklistResult::NotApplicable,
            _ => ChecklistResult::Fail,
        });
        Ok(UpdateChecklistItemInput {
            id: item_id,
            result,
            actual_value: item.actual_value.clone(),
            finding: item.finding.clone(),
            photo_url: item.photo_url.clone(),
        })
    }).collect::<Result<Vec<_>, String>>()
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e}}))))?;

    let repo = PgInspectionRepository::new(state.pool.clone());
    let service = InspectionService::new(repo);
    match service.update_items(&ctx, inspection_id, items).await {
        Ok(inspection) => Ok(Json(json!({
            "data": {
                "id": inspection.id.to_string(),
                "work_order_id": inspection.work_order_id.to_string(),
                "template_name": inspection.template_name,
            }
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}
