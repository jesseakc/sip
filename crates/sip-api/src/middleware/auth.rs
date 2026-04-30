use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sip_auth::decode_jwt;
use sip_domain::tenant::TenantContext;
use std::sync::Arc;

use crate::AppState;

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let auth_header = req.headers().get(AUTHORIZATION).and_then(|h| h.to_str().ok());
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return (StatusCode::UNAUTHORIZED, Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Missing bearer token"}}))).into_response();
        }
    };

    let jwt_secret = state.config.auth.as_ref()
        .map(|a| a.jwt_secret.as_str())
        .unwrap_or("");

    let claims = match decode_jwt(token, jwt_secret) {
        Ok(c) => c,
        Err(_) => {
            return (StatusCode::UNAUTHORIZED, Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Invalid token"}}))).into_response();
        }
    };

    let org_id = match claims.org_id.parse() {
        Ok(id) => id,
        Err(_) => return (StatusCode::UNAUTHORIZED, Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Invalid org_id in token"}}))).into_response(),
    };

    let user_id = match claims.sub.parse() {
        Ok(id) => Some(id),
        Err(_) => None,
    };

    let ctx = TenantContext::new(org_id, user_id).with_permissions(claims.permissions);
    req.extensions_mut().insert(ctx);
    next.run(req).await
}
