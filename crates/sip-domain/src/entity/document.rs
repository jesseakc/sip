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
