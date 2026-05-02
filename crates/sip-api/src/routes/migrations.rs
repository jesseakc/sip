use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use sip_application::services::MigrationService;
use sip_domain::{
    entity::migration::MigrationFieldMapping, error::SipError, id::MigrationJobId,
    tenant::TenantContext,
};
use std::str::FromStr;
use std::sync::Arc;

use crate::AppState;

fn migration_error(
    status: StatusCode,
    code: &str,
    message: String,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(json!({"error": {"code": code, "message": message}})),
    )
}

fn map_migration_error(e: SipError) -> (StatusCode, Json<serde_json::Value>) {
    match &e {
        SipError::PermissionDenied => {
            migration_error(StatusCode::FORBIDDEN, "FORBIDDEN", e.to_string())
        }
        SipError::Validation(msg) => {
            if msg.contains("not found")
                || msg.contains("job not found")
                || msg.contains("Job not found")
            {
                migration_error(StatusCode::NOT_FOUND, "NOT_FOUND", e.to_string())
            } else if msg.contains("duplicate")
                || msg.contains("already exists")
                || msg.contains("conflict")
            {
                migration_error(StatusCode::CONFLICT, "CONFLICT", e.to_string())
            } else {
                migration_error(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "VALIDATION_ERROR",
                    e.to_string(),
                )
            }
        }
        SipError::InvalidStateTransition { .. } => {
            migration_error(StatusCode::CONFLICT, "CONFLICT", e.to_string())
        }
        SipError::VersionConflict { .. } => {
            migration_error(StatusCode::CONFLICT, "CONFLICT", e.to_string())
        }
        SipError::TenantScopeViolation => {
            migration_error(StatusCode::FORBIDDEN, "FORBIDDEN", e.to_string())
        }
        SipError::CapabilityNotAvailable(_) => migration_error(
            StatusCode::NOT_IMPLEMENTED,
            "CAPABILITY_UNAVAILABLE",
            e.to_string(),
        ),
    }
}

#[derive(Deserialize)]
pub struct CreateMigrationJobRequest {
    pub name: String,
    pub source_system: String,
    pub source_object_type: String,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct AddSourceRecordsRequest {
    pub records: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct SaveFieldMappingsRequest {
    pub mappings: Vec<MigrationFieldMapping>,
}

pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = MigrationService::new(state.pool.clone());
    match service.list_jobs(&ctx).await {
        Ok(jobs) => Ok(Json(json!({
            "data": jobs.iter().map(|j| json!({
                "id": j.id.to_string(),
                "name": j.name,
                "source_system": j.source_system,
                "source_object_type": j.source_object_type,
                "status": serde_json::to_value(j.status).unwrap_or_default().as_str().unwrap_or("unknown"),
                "source_record_count": j.source_record_count,
                "valid_record_count": j.valid_record_count,
                "imported_record_count": j.imported_record_count,
                "error_count": j.error_count,
                "created_at": j.created_at,
                "updated_at": j.updated_at,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn create_job(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Json(body): Json<CreateMigrationJobRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let service = MigrationService::new(state.pool.clone());
    match service
        .create_job(
            &ctx,
            body.name,
            body.source_system,
            body.source_object_type,
            body.description,
        )
        .await
    {
        Ok(job) => Ok(Json(json!({
            "data": {
                "id": job.id.to_string(),
                "name": job.name,
                "source_system": job.source_system,
                "source_object_type": job.source_object_type,
                "status": serde_json::to_value(job.status).unwrap_or_default().as_str().unwrap_or("unknown"),
            }
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn get_job(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.get_job(&ctx, id).await {
        Ok(Some(job)) => Ok(Json(json!({
            "data": {
                "id": job.id.to_string(),
                "name": job.name,
                "description": job.description,
                "source_system": job.source_system,
                "source_object_type": job.source_object_type,
                "status": serde_json::to_value(job.status).unwrap_or_default().as_str().unwrap_or("unknown"),
                "source_record_count": job.source_record_count,
                "valid_record_count": job.valid_record_count,
                "imported_record_count": job.imported_record_count,
                "error_count": job.error_count,
                "created_at": job.created_at,
                "updated_at": job.updated_at,
            }
        }))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": {"code": "NOT_FOUND", "message": "Job not found"}})),
        )),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn get_source_records(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.get_source_records(&ctx, id).await {
        Ok(records) => Ok(Json(json!({
            "data": records.iter().map(|r| json!({
                "id": r.id.to_string(),
                "external_id": r.external_id,
                "source_object_type": r.source_object_type,
                "raw_data": r.raw_data,
                "status": r.status,
                "row_number": r.row_number,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn add_source_records(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
    Json(body): Json<AddSourceRecordsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.add_source_records(&ctx, id, body.records).await {
        Ok(records) => Ok(Json(json!({
            "data": {
                "record_count": records.len(),
                "status": "uploaded",
            }
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn save_field_mappings(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
    Json(body): Json<SaveFieldMappingsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.save_field_mappings(&ctx, id, body.mappings).await {
        Ok(()) => Ok(Json(json!({
            "data": {
                "status": "mapped",
            }
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn get_field_mappings(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.get_field_mappings(&ctx, id).await {
        Ok(mappings) => Ok(Json(json!({
            "data": mappings.iter().map(|m| json!({
                "id": m.id.to_string(),
                "target_entity_type": m.target_entity_type,
                "source_field": m.source_field,
                "target_field": m.target_field,
                "transform_expression": m.transform_expression,
                "default_value": m.default_value,
                "is_required": m.is_required,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn validate_job(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.validate_job(&ctx, id).await {
        Ok(issues) => Ok(Json(json!({
            "data": {
                "status": "validated",
                "issue_count": issues.len(),
                "issues": issues.iter().map(|i| json!({
                    "severity": i.severity,
                    "field": i.field,
                    "message": i.message,
                })).collect::<Vec<_>>(),
            }
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn dry_run(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.dry_run(&ctx, id).await {
        Ok(run) => Ok(Json(json!({
            "data": {
                "run_id": run.id.to_string(),
                "run_type": serde_json::to_value(run.run_type).unwrap_or_default().as_str().unwrap_or("unknown"),
                "status": serde_json::to_value(run.status).unwrap_or_default().as_str().unwrap_or("unknown"),
                "records_processed": run.records_processed,
                "records_to_create": run.records_created,
                "records_with_errors": run.records_failed,
            }
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn execute_import(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.execute_import(&ctx, id).await {
        Ok(run) => Ok(Json(json!({
            "data": {
                "run_id": run.id.to_string(),
                "status": serde_json::to_value(run.status).unwrap_or_default().as_str().unwrap_or("unknown"),
                "records_processed": run.records_processed,
                "records_created": run.records_created,
                "records_updated": run.records_updated,
                "records_skipped": run.records_skipped,
                "records_failed": run.records_failed,
            }
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn rollback_job(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.rollback_job(&ctx, id).await {
        Ok(run) => Ok(Json(json!({
            "data": {
                "run_id": run.id.to_string(),
                "status": serde_json::to_value(run.status).unwrap_or_default().as_str().unwrap_or("unknown"),
                "records_processed": run.records_processed,
            }
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.cancel_job(&ctx, id).await {
        Ok(()) => Ok(Json(json!({
            "data": {
                "status": "cancelled",
            }
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn get_validation_issues(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.get_validation_issues(&ctx, id).await {
        Ok(issues) => Ok(Json(json!({
            "data": issues.iter().map(|i| json!({
                "id": i.id.to_string(),
                "severity": i.severity,
                "field": i.field,
                "message": i.message,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn get_duplicates(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.get_duplicates(&ctx, id).await {
        Ok(candidates) => Ok(Json(json!({
            "data": candidates.iter().map(|d| json!({
                "id": d.id.to_string(),
                "sip_entity_type": d.sip_entity_type,
                "sip_entity_id": d.sip_entity_id.to_string(),
                "confidence_score": d.confidence_score,
                "match_reason": d.match_reason,
                "status": d.status,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn get_external_id_maps(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.get_external_id_maps(&ctx, id).await {
        Ok(maps) => Ok(Json(json!({
            "data": maps.iter().map(|m| json!({
                "id": m.id.to_string(),
                "source_system": m.source_system,
                "source_object_type": m.source_object_type,
                "source_external_id": m.source_external_id,
                "sip_entity_type": m.sip_entity_type,
                "sip_entity_id": m.sip_entity_id.to_string(),
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn get_staged_records(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.get_staged_records(&ctx, id).await {
        Ok(records) => Ok(Json(json!({
            "data": records.iter().map(|r| json!({
                "id": r.id.to_string(),
                "target_entity_type": r.target_entity_type,
                "canonical_data": r.canonical_data,
                "status": r.status,
                "validation_errors": r.validation_errors,
            })).collect::<Vec<_>>()
        }))),
        Err(e) => Err(map_migration_error(e)),
    }
}

pub async fn get_report(
    State(state): State<Arc<AppState>>,
    Extension(ctx): Extension<TenantContext>,
    Path(job_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let id = MigrationJobId::from_str(&job_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": "BAD_REQUEST", "message": "Invalid job ID"}})),
        )
    })?;
    let service = MigrationService::new(state.pool.clone());
    match service.get_report(&ctx, id).await {
        Ok(report) => Ok(Json(json!({"data": report}))),
        Err(e) => Err(map_migration_error(e)),
    }
}
