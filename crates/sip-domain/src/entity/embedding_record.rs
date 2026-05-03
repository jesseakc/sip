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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_source_type_serde_roundtrip() {
        for ty in [
            EmbeddingSourceType::Asset,
            EmbeddingSourceType::WorkOrder,
            EmbeddingSourceType::DocumentChunk,
            EmbeddingSourceType::InspectionFinding,
        ] {
            let json = serde_json::to_string(&ty).unwrap();
            let back: EmbeddingSourceType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, ty);
        }
    }
}
