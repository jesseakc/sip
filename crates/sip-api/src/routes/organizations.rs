use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::OrganizationService;
use sip_domain::tenant::TenantContext;
use sip_infrastructure::repositories::PgOrganizationRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub timezone: Option<String>,
    pub default_currency: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub slug: String,
    pub timezone: Option<String>,
    pub default_currency: Option<String>,
}

pub async fn get_organization(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgOrganizationRepository::new(state.pool.clone());
    let service = OrganizationService::new(repo);
    match service.get(&ctx).await {
        Ok(org) => Ok(Json(json!({
            "data": {
                "id": org.id.to_string(),
                "name": org.name,
                "slug": org.slug,
                "timezone": org.timezone,
                "default_currency": org.default_currency,
                "created_at": org.created_at,
                "updated_at": org.updated_at,
            }
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn update_organization(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<UpdateOrganizationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgOrganizationRepository::new(state.pool.clone());
    let service = OrganizationService::new(repo);
    match service.update(&ctx, req.name, req.timezone, req.default_currency).await {
        Ok(org) => Ok(Json(json!({
            "data": {
                "id": org.id.to_string(),
                "name": org.name,
                "slug": org.slug,
                "timezone": org.timezone,
                "default_currency": org.default_currency,
                "created_at": org.created_at,
                "updated_at": org.updated_at,
            }
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn create_organization(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateOrganizationRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgOrganizationRepository::new(state.pool.clone());
    let service = OrganizationService::new(repo);
    match service.create(&ctx, &req.name, &req.slug, req.timezone.as_deref(), req.default_currency.as_deref()).await {
        Ok(org) => Ok(Json(json!({
            "data": {
                "id": org.id.to_string(),
                "name": org.name,
                "slug": org.slug,
                "timezone": org.timezone,
                "default_currency": org.default_currency,
                "created_at": org.created_at,
                "updated_at": org.updated_at,
            }
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}
