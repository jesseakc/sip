use axum::{
    extract::{Extension, Json, State},
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sip_application::services::AuthService;
use sip_auth::passwords::hash_password;
use sip_domain::tenant::TenantContext;
use std::sync::Arc;
use uuid::Uuid;

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
        state
            .config
            .auth
            .as_ref()
            .map(|a| a.jwt_secret.clone())
            .unwrap_or_default(),
        state
            .config
            .auth
            .as_ref()
            .map(|a| a.jwt_expiration_seconds)
            .unwrap_or(900),
        state
            .config
            .auth
            .as_ref()
            .map(|a| a.refresh_expiration_seconds)
            .unwrap_or(604800),
    );
    match auth.login(&req.email, &req.password).await {
        Ok((token, refresh)) => Ok(Json(LoginResponse {
            token,
            refresh_token: refresh,
        })),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "UNAUTHORIZED", "message": e.to_string()}})),
        )),
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
        state
            .config
            .auth
            .as_ref()
            .map(|a| a.jwt_secret.clone())
            .unwrap_or_default(),
        state
            .config
            .auth
            .as_ref()
            .map(|a| a.jwt_expiration_seconds)
            .unwrap_or(900),
        state
            .config
            .auth
            .as_ref()
            .map(|a| a.refresh_expiration_seconds)
            .unwrap_or(604800),
    );
    match auth.refresh(&req.refresh_token).await {
        Ok(token) => Ok(Json(json!({"data": {"token": token}}))),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "UNAUTHORIZED", "message": e.to_string()}})),
        )),
    }
}

pub async fn logout() -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    Ok(Json(json!({"data": {}})))
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = match ctx.user_id {
        Some(id) => id,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"code": "UNAUTHORIZED", "message": "No user in token"}})),
            ))
        }
    };

    // Look up user from database to get name and email
    let row = sqlx::query_as::<_, (String, String, String)>(
        "SELECT email, name, role FROM users WHERE id = $1 AND organization_id = $2",
    )
    .bind(Uuid::from(user_id))
    .bind(Uuid::from(ctx.organization_id))
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )
    })?;

    match row {
        Some((email, name, role)) => Ok(Json(json!({
            "data": {
                "id": user_id.to_string(),
                "email": email,
                "name": name,
                "role": role,
                "organization_id": ctx.organization_id.to_string(),
                "permissions": ctx.permissions,
            }
        }))),
        None => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": {"code": "UNAUTHORIZED", "message": "User not found"}})),
        )),
    }
}

#[derive(Deserialize)]
pub struct BootstrapRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub organization_name: String,
}

/// Constant-time comparison, so a wrong token cannot be recovered byte by byte
/// from response timing.
fn token_matches(provided: &str, expected: &str) -> bool {
    let (a, b) = (provided.as_bytes(), expected.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn bootstrap_error(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(
            json!({"error": {"code": status.canonical_reason().unwrap_or("ERROR"), "message": message}}),
        ),
    )
}

/// `POST /api/v1/auth/bootstrap`
///
/// Creates the first organisation and its administrator on an empty database.
/// This exists because SIP no longer ships a seeded administrator: a fresh
/// deployment needs exactly one controlled way in.
///
/// Three independent guards, all failing closed:
///   1. `SIP_BOOTSTRAP_TOKEN` must be configured — unset means the endpoint is off.
///   2. The caller must present that token in `X-Bootstrap-Token`.
///   3. Bootstrap is only possible while no user exists, so it is single-use by
///      state rather than relying on someone remembering to unset the token.
pub async fn bootstrap(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<BootstrapRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let expected = match state.config.security.bootstrap_token.as_deref() {
        Some(token) if !token.trim().is_empty() => token.to_string(),
        _ => {
            return Err(bootstrap_error(
                StatusCode::FORBIDDEN,
                "bootstrap is disabled: SIP_BOOTSTRAP_TOKEN is not configured",
            ))
        }
    };

    let provided = headers
        .get("x-bootstrap-token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !token_matches(provided, &expected) {
        return Err(bootstrap_error(
            StatusCode::FORBIDDEN,
            "invalid bootstrap token",
        ));
    }

    let (existing,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| bootstrap_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;
    if existing > 0 {
        return Err(bootstrap_error(
            StatusCode::CONFLICT,
            "already initialised: this deployment has users, so bootstrap is closed",
        ));
    }

    let email = req.email.trim().to_lowercase();
    if !email.contains('@') || email.len() < 3 {
        return Err(bootstrap_error(
            StatusCode::BAD_REQUEST,
            "a valid email is required",
        ));
    }
    if req.password.chars().count() < 12 {
        return Err(bootstrap_error(
            StatusCode::BAD_REQUEST,
            "the administrator password must be at least 12 characters",
        ));
    }
    let org_name = req.organization_name.trim();
    if org_name.is_empty() {
        return Err(bootstrap_error(
            StatusCode::BAD_REQUEST,
            "organization_name is required",
        ));
    }

    let slug: String = org_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if slug.is_empty() {
        return Err(bootstrap_error(
            StatusCode::BAD_REQUEST,
            "organization_name must contain letters or digits",
        ));
    }

    let password_hash = hash_password(&req.password)
        .map_err(|e| bootstrap_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    let organization_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| bootstrap_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    sqlx::query("INSERT INTO organizations (id, name, slug) VALUES ($1, $2, $3)")
        .bind(organization_id)
        .bind(org_name)
        .bind(&slug)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            bootstrap_error(
                StatusCode::CONFLICT,
                &format!("could not create organization: {e}"),
            )
        })?;

    sqlx::query(
        "INSERT INTO users (id, organization_id, email, name, role, skills, certifications, password_hash, is_active)
         VALUES ($1, $2, $3, $4, 'ADMIN', ARRAY[]::text[], '[]'::jsonb, $5, true)",
    )
    .bind(user_id)
    .bind(organization_id)
    .bind(&email)
    .bind(req.name.trim())
    .bind(&password_hash)
    .execute(&mut *tx)
    .await
    .map_err(|e| bootstrap_error(StatusCode::CONFLICT, &format!("could not create user: {e}")))?;

    tx.commit()
        .await
        .map_err(|e| bootstrap_error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(json!({
        "data": {
            "organization_id": organization_id.to_string(),
            "organization_slug": slug,
            "user_id": user_id.to_string(),
            "email": email,
            "role": "ADMIN",
            "next": "sign in, then remove SIP_BOOTSTRAP_TOKEN from the environment",
        }
    })))
}
