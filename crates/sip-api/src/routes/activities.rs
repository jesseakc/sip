use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde_json::json;
use sip_application::services::ActivityService;
use sip_domain::tenant::TenantContext;
use sip_infrastructure::repositories::PgActivityRepository;
use std::sync::Arc;

use crate::AppState;

pub async fn list_activities(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgActivityRepository::new(state.pool.clone());
    let service = ActivityService::new(repo);
    match service.list(&ctx).await {
        Ok(activities) => Ok(Json(json!({
            "data": activities.iter().map(|a| json!({
                "id": a.id.to_string(),
                "actor_id": a.actor_id.map(|id| id.to_string()),
                "actor_type": format!("{:?}", a.actor_type).to_uppercase(),
                "entity_type": a.entity_type,
                "entity_id": a.entity_id.to_string(),
                "action": a.action,
                "metadata": a.metadata,
                "created_at": a.created_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn list_activities_by_entity(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let entity_uuid = entity_id.parse::<uuid::Uuid>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgActivityRepository::new(state.pool.clone());
    let service = ActivityService::new(repo);
    match service
        .list_by_entity(&ctx, &entity_type, entity_uuid)
        .await
    {
        Ok(activities) => Ok(Json(json!({
            "data": activities.iter().map(|a| json!({
                "id": a.id.to_string(),
                "actor_id": a.actor_id.map(|id| id.to_string()),
                "actor_type": format!("{:?}", a.actor_type).to_uppercase(),
                "entity_type": a.entity_type,
                "entity_id": a.entity_id.to_string(),
                "action": a.action,
                "metadata": a.metadata,
                "created_at": a.created_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}
