use axum::{extract::State, http::StatusCode, Json};
use serde_json::json;
use std::sync::Arc;

use crate::AppState;

pub async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

pub async fn ready(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match sqlx::query("SELECT 1").fetch_one(&state.pool).await {
        Ok(_) => Ok(Json(json!({"status": "ready", "database": "ok"}))),
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": "not_ready", "database": e.to_string()})),
        )),
    }
}
