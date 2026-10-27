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
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Missing bearer token"}})),
            )
                .into_response();
        }
    };

    let jwt_secret = state
        .config
        .auth
        .as_ref()
        .map(|a| a.jwt_secret.as_str())
        .unwrap_or("");

    let claims = match decode_jwt(token, jwt_secret) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Invalid token"}})),
            )
                .into_response();
        }
    };

    let org_id = match claims.org_id.parse() {
        Ok(id) => id,
        Err(_) => return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Invalid org_id in token"}})),
        )
            .into_response(),
    };

    // Tokens are stateless, so nothing revoked them before now: a deactivated user,
    // or one who signed out everywhere, kept working until the token aged out. This
    // is the check that makes revocation immediate. It fails closed — if the session
    // cannot be verified, the request is not authenticated.
    let user_uuid = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Invalid subject in token"}})),
        )
            .into_response(),
    };

    let users = sip_infrastructure::repositories::PgUserRepository::new(state.pool.clone());
    match users.get_session_state(user_uuid).await {
        Ok(Some((true, epoch))) if epoch.max(0) as u32 == claims.auth_epoch => {}
        Ok(Some((false, _))) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Account is inactive"}})),
            )
                .into_response()
        }
        Ok(Some(_)) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Session has been revoked; sign in again"}})),
            )
                .into_response()
        }
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Unknown user"}})),
            )
                .into_response()
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
            )
                .into_response()
        }
    }

    let user_id = claims.sub.parse().ok();

    let ctx = TenantContext::new(org_id, user_id).with_permissions(claims.permissions);
    req.extensions_mut().insert(ctx);
    next.run(req).await
}
