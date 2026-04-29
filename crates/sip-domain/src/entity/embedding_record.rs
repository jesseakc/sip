use serde::{Deserialize, Serialize};

use crate::id::{DocumentChunkId, EmbeddingRecordId, OrganizationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbeddingSourceType {
    Asset,
    WorkOrder,
    DocumentChunk,
    InspectionFinding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingRecord {
    pub id: EmbeddingRecordId,
    pub organization_id: OrganizationId,
    pub source_type: EmbeddingSourceType,
    pub source_id: uuid::Uuid,
    pub document_chunk_id: Option<DocumentChunkId>,
    pub collection: String,
    pub content: String,
    pub embedding_model: String,
    pub model_name: String,
    pub dimensions: i32,
    pub source_scope: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
