use serde::{Deserialize, Serialize};

use crate::id::{DocumentChunkId, DocumentId, OrganizationId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: DocumentChunkId,
    pub organization_id: OrganizationId,
    pub document_id: DocumentId,
    pub chunk_index: i32,
    pub content: String,
    pub token_count: i32,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
