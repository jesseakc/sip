use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{AIConversationId, AIMessageId, AIRetrievalTraceId, OrganizationId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIConversation {
    pub id: AIConversationId,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiMessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrieverType {
    SQL,
    Vector,
    RAG,
    GRAPH,
    TEMPORAL,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIMessage {
    pub id: AIMessageId,
    pub conversation_id: AIConversationId,
    pub role: String,
    pub content: String,
    pub sources: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AIRetrievalTrace {
    pub id: AIRetrievalTraceId,
    pub organization_id: OrganizationId,
    pub message_id: AIMessageId,
    pub retriever_type: RetrieverType,
    pub source_type: String,
    pub source_id: uuid::Uuid,
    pub query: String,
    pub strategy: String,
    pub records_queried: i32,
    pub records_returned: i32,
    pub duration_ms: i32,
    pub score: Option<rust_decimal::Decimal>,
    pub included_in_context: bool,
    pub source_scope: Option<String>,
    pub verified_by_sql: bool,
    pub rank: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    #[serde(rename = "type")]
    pub source_type: String,
    pub title: String,
    pub relevance: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieval_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_by_sql: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}
