use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::TeamService;
use sip_domain::{
    id::{TeamId, UserId},
    tenant::TenantContext,
};
use sip_infrastructure::repositories::PgTeamRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: Option<String>,
    pub lead_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTeamRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub lead_id: Option<String>,
}

pub async fn list_teams(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgTeamRepository::new(state.pool.clone());
    let service = TeamService::new(repo);
    match service.list(&ctx).await {
        Ok(teams) => Ok(Json(json!({
            "data": teams.iter().map(|t| json!({
                "id": t.id.to_string(),
                "name": t.name,
                "description": t.description,
                "lead_id": t.lead_id.map(|id| id.to_string()),
                "created_at": t.created_at,
                "updated_at": t.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn get_team(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let team_id = id.parse::<TeamId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgTeamRepository::new(state.pool.clone());
    let service = TeamService::new(repo);
    match service.get(&ctx, team_id).await {
        Ok(Some(t)) => Ok(Json(json!({
            "data": {
                "id": t.id.to_string(),
                "name": t.name,
                "description": t.description,
                "lead_id": t.lead_id.map(|id| id.to_string()),
                "created_at": t.created_at,
                "updated_at": t.updated_at,
            }
        }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": {"code": "NOT_FOUND", "message": "Team not found"}})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn create_team(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let lead_id = req.lead_id.map(|s| s.parse::<UserId>()).transpose().map_err(|e: uuid::Error| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgTeamRepository::new(state.pool.clone());
    let service = TeamService::new(repo);
    match service.create(&ctx, req.name, req.description, lead_id).await {
        Ok(t) => Ok(Json(json!({"data": {"id": t.id.to_string(), "name": t.name}}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn update_team(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTeamRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let team_id = id.parse::<TeamId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let lead_id = req.lead_id.map(|s| s.parse::<UserId>()).transpose().map_err(|e: uuid::Error| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgTeamRepository::new(state.pool.clone());
    let service = TeamService::new(repo);
    match service.update(&ctx, team_id, req.name, req.description, lead_id).await {
        Ok(t) => Ok(Json(json!({"data": {"id": t.id.to_string(), "name": t.name}}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}
