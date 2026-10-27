use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sip_auth::{decode_jwt, TokenUse};
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

    if claims.token_use != TokenUse::Access {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":{"code":"UNAUTHORIZED","message":"Access token required"}})),
        )
            .into_response();
    }
    let org_id: sip_domain::id::OrganizationId = match claims.org_id.parse() {
        Ok(id) => id,
        Err(_) => return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "UNAUTHORIZED", "message": "Invalid org_id in token"}})),
        )
            .into_response(),
    };

    let user_id = match claims.sub.parse::<sip_domain::id::UserId>() {
        Ok(id) => id,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let row = sqlx::query_as::<_, (bool, i32, String)>(
        "SELECT is_active,auth_epoch,role FROM users WHERE id=$1 AND organization_id=$2",
    )
    .bind(uuid::Uuid::from(user_id))
    .bind(uuid::Uuid::from(org_id))
    .fetch_optional(&state.pool)
    .await;
    match row {
        Ok(Some((true, epoch, role))) if epoch == claims.auth_epoch && role == claims.role => {}
        Ok(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(
                    json!({"error":{"code":"UNAUTHORIZED","message":"Session is no longer valid"}}),
                ),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("session validation failed: {}", e);
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    }

    let ctx = TenantContext::new(org_id, Some(user_id)).with_permissions(claims.permissions);
    req.extensions_mut().insert(ctx);
    next.run(req).await
}
