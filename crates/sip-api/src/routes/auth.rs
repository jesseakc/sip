use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sip_application::services::AuthService;
use sip_domain::tenant::TenantContext;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub refresh_token: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<serde_json::Value>)> {
    let auth = AuthService::new(
        state.pool.clone(),
        state.config.auth.as_ref().map(|a| a.jwt_secret.clone()).unwrap_or_default(),
        state.config.auth.as_ref().map(|a| a.jwt_expiration_seconds).unwrap_or(900),
        state.config.auth.as_ref().map(|a| a.refresh_expiration_seconds).unwrap_or(604800),
    );
    match auth.login(&req.email, &req.password).await {
        Ok((token, refresh)) => Ok(Json(LoginResponse { token, refresh_token: refresh })),
        Err(e) => Err((StatusCode::UNAUTHORIZED, Json(json!({"error": {"code": "UNAUTHORIZED", "message": e.to_string()}})))),
    }
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let auth = AuthService::new(
        state.pool.clone(),
        state.config.auth.as_ref().map(|a| a.jwt_secret.clone()).unwrap_or_default(),
        state.config.auth.as_ref().map(|a| a.jwt_expiration_seconds).unwrap_or(900),
        state.config.auth.as_ref().map(|a| a.refresh_expiration_seconds).unwrap_or(604800),
    );
    match auth.refresh(&req.refresh_token).await {
        Ok(token) => Ok(Json(json!({"data": {"token": token}}))),
        Err(e) => Err((StatusCode::UNAUTHORIZED, Json(json!({"error": {"code": "UNAUTHORIZED", "message": e.to_string()}})))),
    }
}

pub async fn logout() -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(json!({"data": {}})))
}

pub async fn me(
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(json!({
        "id": ctx.user_id.map(|id| id.to_string()).unwrap_or_default(),
        "organization_id": ctx.organization_id.to_string(),
        "role": "UNKNOWN",
        "permissions": ctx.permissions,
    })))
}
