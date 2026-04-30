use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sip_ai::create_provider;
use sip_application::services::AIService;
use sip_domain::{
    id::{AIConversationId, AIMessageId},
    tenant::TenantContext,
};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;

#[derive(Deserialize)]
pub struct FeedbackRequest {
    pub rating: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct VerificationTraceRow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub message_id: Uuid,
    pub claim: String,
    pub verification_status: String,
    pub verified_by: Option<String>,
    pub contradiction: Option<String>,
    pub supporting_evidence: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_conversations(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = match ctx.user_id {
        Some(uid) => uid,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"code": "UNAUTHORIZED", "message": "User required"}})),
            ))
        }
    };
    let ai = AIService::new(
        create_provider(&state.config.ai),
        state.pool.clone(),
    );
    match ai.list_conversations(&ctx, user_id).await {
        Ok(conversations) => Ok(Json(json!({
            "data": conversations.iter().map(|c| json!({
                "id": c.id.to_string(),
                "title": c.title,
                "created_at": c.created_at,
                "updated_at": c.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_conversation(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let conv_id = AIConversationId::from(id);
    let ai = AIService::new(
        create_provider(&state.config.ai),
        state.pool.clone(),
    );
    match ai.get_conversation(&ctx, conv_id).await {
        Ok(Some((conversation, messages))) => Ok(Json(json!({
            "data": {
                "id": conversation.id.to_string(),
                "title": conversation.title,
                "created_at": conversation.created_at,
                "updated_at": conversation.updated_at,
                "messages": messages.iter().map(|m| json!({
                    "id": m.id.to_string(),
                    "role": m.role,
                    "content": m.content,
                    "sources": m.sources,
                    "created_at": m.created_at,
                })).collect::<Vec<_>>()
            }
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "NOT_FOUND", "message": "Conversation not found"}})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_retrieval_trace(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let msg_id = AIMessageId::from(id);
    let ai = AIService::new(
        create_provider(&state.config.ai),
        state.pool.clone(),
    );
    match ai.get_retrieval_trace(&ctx, msg_id).await {
        Ok(traces) => Ok(Json(json!({
            "data": traces.iter().map(|t| json!({
                "id": t.id.to_string(),
                "message_id": t.message_id.to_string(),
                "query": t.query,
                "strategy": t.strategy,
                "records_queried": t.records_queried,
                "records_returned": t.records_returned,
                "duration_ms": t.duration_ms,
                "created_at": t.created_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<Uuid>,
    Json(body): Json<FeedbackRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let msg_id = AIMessageId::from(id);
    let ai = AIService::new(
        create_provider(&state.config.ai),
        state.pool.clone(),
    );
    match ai.submit_feedback(&ctx, msg_id, body.rating, body.comment).await {
        Ok(()) => Ok(Json(json!({"data": {"status": "ok"}}))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_verification_trace(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(message_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let trace = sqlx::query_as::<_, VerificationTraceRow>(
        "SELECT * FROM verification_traces WHERE message_id = $1 AND organization_id = $2 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(message_id)
    .bind(Uuid::from(ctx.organization_id))
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        let err_msg = e.to_string();
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": {"code": "INTERNAL_ERROR", "message": err_msg}})))
    })?;

    match trace {
        Some(t) => Ok(Json(serde_json::json!({"data": t}))),
        None => Ok(Json(serde_json::json!({"data": null, "message": "No verification trace found"}))),
    }
}
