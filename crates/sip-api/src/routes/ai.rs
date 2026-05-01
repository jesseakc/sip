use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::sse::{Event, Sse},
};
use serde::Deserialize;
use serde_json::json;
use sip_ai::create_provider;
use sip_application::services::AIService;
use sip_domain::tenant::TenantContext;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;

use crate::AppState;

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
}

pub async fn chat(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    axum::extract::Json(req): axum::extract::Json<ChatRequest>,
) -> Result<
    Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>,
    (StatusCode, axum::Json<serde_json::Value>),
> {
    let provider = create_provider(&state.config.ai);
    let ai = AIService::new(provider, state.pool.clone());

    match ai
        .chat_stream(&ctx, &req.message, req.conversation_id)
        .await
    {
        Ok(rx) => {
            let stream = UnboundedReceiverStream::new(rx).map(|event| {
                let data = serde_json::to_string(&event).unwrap_or_default();
                Ok::<_, Infallible>(Event::default().data(data))
            });
            Ok(Sse::new(stream))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({"error": {"code": "AI_ERROR", "message": e.to_string()}})),
        )),
    }
}
