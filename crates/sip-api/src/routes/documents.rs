use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::{CreateDocumentInput, DocumentService};
use sip_domain::{
    entity::document::{DocumentSourceType, DocumentType, Visibility},
    id::DocumentId,
    tenant::TenantContext,
};
use sip_infrastructure::repositories::PgDocumentRepository;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CreateDocumentRequest {
    pub name: String,
    pub document_type: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub document_version: Option<String>,
    pub checksum: String,
    pub source_type: String,
    pub source_system: Option<String>,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
    pub storage_path: String,
    pub visibility: String,
    pub effective_date: Option<chrono::NaiveDate>,
    pub expiration_date: Option<chrono::NaiveDate>,
    pub supersedes_document_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

pub async fn list_documents(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let repo = PgDocumentRepository::new(state.pool.clone());
    let service = DocumentService::new(repo);
    match service.list(&ctx).await {
        Ok(docs) => Ok(Json(json!({
            "data": docs.iter().map(|d| json!({
                "id": d.id.to_string(),
                "name": d.name,
                "document_type": format!("{:?}", d.document_type).to_uppercase(),
                "mime_type": d.mime_type,
                "size_bytes": d.size_bytes,
                "processing_status": format!("{:?}", d.processing_status).to_uppercase(),
                "created_at": d.created_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn get_document(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let doc_id = id.parse::<DocumentId>().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
        )
    })?;
    let repo = PgDocumentRepository::new(state.pool.clone());
    let service = DocumentService::new(repo);
    match service.get(&ctx, doc_id).await {
        Ok(Some(doc)) => Ok(Json(json!({
            "data": {
                "id": doc.id.to_string(),
                "name": doc.name,
                "document_type": format!("{:?}", doc.document_type).to_uppercase(),
                "mime_type": doc.mime_type,
                "size_bytes": doc.size_bytes,
                "checksum": doc.checksum,
                "source_type": format!("{:?}", doc.source_type).to_uppercase(),
                "storage_path": doc.storage_path,
                "visibility": format!("{:?}", doc.visibility).to_uppercase(),
                "processing_status": format!("{:?}", doc.processing_status).to_uppercase(),
                "version": doc.version,
                "uploaded_by_id": doc.uploaded_by_id.to_string(),
                "created_at": doc.created_at,
                "updated_at": doc.updated_at,
            }
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "NOT_FOUND", "message": "Document not found"}})),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn create_document(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let document_type = match req.document_type.to_uppercase().as_str() {
        "MANUAL" => DocumentType::Manual,
        "PROCEDURE" => DocumentType::Procedure,
        "DIAGRAM" => DocumentType::Diagram,
        "WARRANTY" => DocumentType::Warranty,
        "CERTIFICATE" => DocumentType::Certificate,
        "PHOTO" => DocumentType::Photo,
        _ => DocumentType::Other,
    };
    let source_type = match req.source_type.to_uppercase().as_str() {
        "API" => DocumentSourceType::Api,
        "PLUGIN" => DocumentSourceType::Plugin,
        "SYSTEM" => DocumentSourceType::System,
        "VENDOR" => DocumentSourceType::Vendor,
        "PUBLIC_IMPORT" => DocumentSourceType::PublicImport,
        _ => DocumentSourceType::Upload,
    };
    let visibility = match req.visibility.to_uppercase().as_str() {
        "PRIVATE_TENANT" => Visibility::PrivateTenant,
        "SHARED_VENDOR" => Visibility::SharedVendor,
        "PUBLIC" => Visibility::Public,
        _ => Visibility::SystemDefault,
    };
    let supersedes_document_id = req
        .supersedes_document_id
        .map(|s| s.parse())
        .transpose()
        .map_err(|e: uuid::Error| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": {"code": "BAD_REQUEST", "message": e.to_string()}})),
            )
        })?;
    let input = CreateDocumentInput {
        name: req.name,
        document_type,
        mime_type: req.mime_type,
        size_bytes: req.size_bytes,
        document_version: req.document_version,
        checksum: req.checksum,
        source_type,
        source_system: req.source_system,
        external_id: req.external_id,
        external_url: req.external_url,
        storage_path: req.storage_path,
        visibility,
        effective_date: req.effective_date,
        expiration_date: req.expiration_date,
        supersedes_document_id,
        metadata: req.metadata,
    };
    let repo = PgDocumentRepository::new(state.pool.clone());
    let service = DocumentService::new(repo);
    match service.create(&ctx, input).await {
        Ok(doc) => Ok(Json(
            json!({"data": {"id": doc.id.to_string(), "name": doc.name}}),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}

pub async fn archive_document(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(id): Path<uuid::Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let doc_id = DocumentId::from(id);
    let reason = body.get("reason").and_then(|v| v.as_str()).unwrap_or("");
    let repo = PgDocumentRepository::new(state.pool.clone());
    let service = DocumentService::new(repo);
    match service.archive(&ctx, doc_id, reason).await {
        Ok(doc) => Ok(Json(json!({
            "data": {
                "id": doc.id.to_string(),
                "name": doc.name,
                "archived_at": doc.archived_at,
                "archive_reason": doc.archive_reason,
            }
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": {"code": "INTERNAL_ERROR", "message": e.to_string()}})),
        )),
    }
}
