use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::UserService;
use sip_auth::hash_password;
use sip_domain::{
    entity::user::UserRole,
    id::UserId,
    tenant::TenantContext,
};
use sip_infrastructure::repositories::PgUserRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub role: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgUserRepository::new(state.pool.clone());
    let service = UserService::new(repo);
    match service.list(&ctx).await {
        Ok(users) => Ok(Json(json!({
            "data": users.iter().map(|u| json!({
                "id": u.id.to_string(),
                "email": u.email,
                "name": u.name,
                "role": format!("{:?}", u.role).to_uppercase(),
                "is_active": u.is_active,
                "created_at": u.created_at,
                "updated_at": u.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = id.parse::<UserId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let repo = PgUserRepository::new(state.pool.clone());
    let service = UserService::new(repo);
    match service.get(&ctx, user_id).await {
        Ok(Some(u)) => Ok(Json(json!({
            "data": {
                "id": u.id.to_string(),
                "email": u.email,
                "name": u.name,
                "role": format!("{:?}", u.role).to_uppercase(),
                "skills": u.skills,
                "certifications": u.certifications,
                "working_hours": u.working_hours,
                "is_active": u.is_active,
                "created_at": u.created_at,
                "updated_at": u.updated_at,
            }
        }))),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": {"code": "NOT_FOUND", "message": "User not found"}})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let role = match req.role.to_uppercase().as_str() {
        "ADMIN" => UserRole::Admin,
        "MANAGER" => UserRole::Manager,
        "TECHNICIAN" => UserRole::Technician,
        "VIEWER" => UserRole::Viewer,
        "VENDOR" => UserRole::Vendor,
        _ => UserRole::Auditor,
    };
    let password = req.password;
    let password_hash = hash_password(&password).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}}))))?;
    let repo = PgUserRepository::new(state.pool.clone());
    let service = UserService::new(repo);
    match service.create(&ctx, req.email, req.name, role, password_hash).await {
        Ok(u) => Ok(Json(json!({"data": {"id": u.id.to_string(), "name": u.name, "email": u.email}}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}

pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = id.parse::<UserId>().map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}}))))?;
    let role = req.role.map(|r| match r.to_uppercase().as_str() {
        "ADMIN" => UserRole::Admin,
        "MANAGER" => UserRole::Manager,
        "TECHNICIAN" => UserRole::Technician,
        "VIEWER" => UserRole::Viewer,
        "VENDOR" => UserRole::Vendor,
        _ => UserRole::Auditor,
    });
    let repo = PgUserRepository::new(state.pool.clone());
    let service = UserService::new(repo);
    match service.update(&ctx, user_id, req.name, role, req.is_active).await {
        Ok(u) => Ok(Json(json!({"data": {"id": u.id.to_string(), "name": u.name, "role": format!("{:?}", u.role).to_uppercase()}}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})))),
    }
}
