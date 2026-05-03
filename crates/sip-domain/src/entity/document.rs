use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{DocumentChunkId, DocumentId, DocumentLinkId, OrganizationId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    Manual,
    Procedure,
    Diagram,
    Warranty,
    Certificate,
    Photo,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentSourceType {
    Upload,
    Api,
    Plugin,
    System,
    Vendor,
    PublicImport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    PrivateTenant,
    SharedVendor,
    Public,
    SystemDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Pending,
    Extracting,
    Extracted,
    Chunking,
    Embedding,
    Indexed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub document_type: DocumentType,
    pub mime_type: String,
    pub size_bytes: i64,
    pub document_version: Option<String>,
    pub checksum: String,
    pub source_type: DocumentSourceType,
    pub source_system: Option<String>,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
    pub storage_path: String,
    pub visibility: Visibility,
    pub processing_status: ProcessingStatus,
    pub processing_error: Option<String>,
    pub extracted_text_path: Option<String>,
    pub text_content: Option<String>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub expiration_date: Option<chrono::NaiveDate>,
    pub supersedes_document_id: Option<DocumentId>,
    pub version: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub archived_by_id: Option<UserId>,
    pub archive_reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub uploaded_by_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkEntityType {
    Asset,
    WorkOrder,
    AssetModel,
    Manufacturer,
    Part,
    Inspection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    ManualFor,
    PhotoOf,
    WarrantyFor,
    ProcedureFor,
    EvidenceFor,
    Attachment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentLink {
    pub id: DocumentLinkId,
    pub organization_id: OrganizationId,
    pub document_id: DocumentId,
    pub entity_type: LinkEntityType,
    pub entity_id: uuid::Uuid,
    pub relationship_type: RelationshipType,
    pub created_by_id: UserId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: DocumentChunkId,
    pub organization_id: OrganizationId,
    pub document_id: DocumentId,
    pub chunk_index: i32,
    pub content: String,
    pub token_count: Option<i32>,
    pub page_number: Option<i32>,
    pub section_title: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_type_serde_roundtrip() {
        for ty in [
            DocumentType::Manual,
            DocumentType::Procedure,
            DocumentType::Diagram,
            DocumentType::Warranty,
            DocumentType::Certificate,
            DocumentType::Photo,
            DocumentType::Other,
        ] {
            let json = serde_json::to_string(&ty).unwrap();
            let back: DocumentType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, ty);
        }
    }

    #[test]
    fn test_document_source_type_serde_roundtrip() {
        for src in [
            DocumentSourceType::Upload,
            DocumentSourceType::Api,
            DocumentSourceType::Plugin,
            DocumentSourceType::System,
            DocumentSourceType::Vendor,
            DocumentSourceType::PublicImport,
        ] {
            let json = serde_json::to_string(&src).unwrap();
            let back: DocumentSourceType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, src);
        }
    }

    #[test]
    fn test_visibility_serde_roundtrip() {
        for v in [
            Visibility::PrivateTenant,
            Visibility::SharedVendor,
            Visibility::Public,
            Visibility::SystemDefault,
        ] {
            let json = serde_json::to_string(&v).unwrap();
            let back: Visibility = serde_json::from_str(&json).unwrap();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn test_processing_status_serde_roundtrip() {
        for ps in [
            ProcessingStatus::Pending,
            ProcessingStatus::Extracting,
            ProcessingStatus::Extracted,
            ProcessingStatus::Chunking,
            ProcessingStatus::Embedding,
            ProcessingStatus::Indexed,
            ProcessingStatus::Failed,
        ] {
            let json = serde_json::to_string(&ps).unwrap();
            let back: ProcessingStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, ps);
        }
    }

    #[test]
    fn test_link_entity_type_serde_roundtrip() {
        for et in [
            LinkEntityType::Asset,
            LinkEntityType::WorkOrder,
            LinkEntityType::AssetModel,
            LinkEntityType::Manufacturer,
            LinkEntityType::Part,
            LinkEntityType::Inspection,
        ] {
            let json = serde_json::to_string(&et).unwrap();
            let back: LinkEntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, et);
        }
    }

    #[test]
    fn test_relationship_type_serde_roundtrip() {
        for rt in [
            RelationshipType::ManualFor,
            RelationshipType::PhotoOf,
            RelationshipType::WarrantyFor,
            RelationshipType::ProcedureFor,
            RelationshipType::EvidenceFor,
            RelationshipType::Attachment,
        ] {
            let json = serde_json::to_string(&rt).unwrap();
            let back: RelationshipType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, rt);
        }
    }
}
