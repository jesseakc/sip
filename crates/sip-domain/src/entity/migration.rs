use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::id::{
    MigrationBatchId, MigrationCheckpointId, MigrationDuplicateCandidateId,
    MigrationExternalIdMapId, MigrationFieldMappingId, MigrationImportResultId, MigrationJobId,
    MigrationRunId, MigrationSourceRecordId, MigrationStagedRecordId, MigrationValidationIssueId,
    OrganizationId, UserId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationJobStatus {
    Draft,
    Uploaded,
    Mapped,
    Validated,
    ReadyForImport,
    Importing,
    Completed,
    CompletedWithWarnings,
    Failed,
    Cancelled,
    RolledBack,
}

impl MigrationJobStatus {
    pub fn can_transition_to(self, next: MigrationJobStatus) -> bool {
        matches!(
            (self, next),
            (MigrationJobStatus::Draft, MigrationJobStatus::Uploaded)
                | (MigrationJobStatus::Draft, MigrationJobStatus::Cancelled)
                | (MigrationJobStatus::Uploaded, MigrationJobStatus::Mapped)
                | (MigrationJobStatus::Uploaded, MigrationJobStatus::Cancelled)
                | (MigrationJobStatus::Mapped, MigrationJobStatus::Validated)
                | (MigrationJobStatus::Mapped, MigrationJobStatus::Cancelled)
                | (
                    MigrationJobStatus::Validated,
                    MigrationJobStatus::ReadyForImport
                )
                | (MigrationJobStatus::Validated, MigrationJobStatus::Cancelled)
                | (
                    MigrationJobStatus::ReadyForImport,
                    MigrationJobStatus::Importing
                )
                | (
                    MigrationJobStatus::ReadyForImport,
                    MigrationJobStatus::Cancelled
                )
                | (MigrationJobStatus::Importing, MigrationJobStatus::Completed)
                | (
                    MigrationJobStatus::Importing,
                    MigrationJobStatus::CompletedWithWarnings
                )
                | (MigrationJobStatus::Importing, MigrationJobStatus::Failed)
                | (MigrationJobStatus::Importing, MigrationJobStatus::Cancelled)
                | (
                    MigrationJobStatus::Completed,
                    MigrationJobStatus::RolledBack
                )
                | (
                    MigrationJobStatus::CompletedWithWarnings,
                    MigrationJobStatus::RolledBack
                )
                | (MigrationJobStatus::Failed, MigrationJobStatus::Draft)
                | (MigrationJobStatus::Cancelled, MigrationJobStatus::Draft)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationJob {
    pub id: MigrationJobId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub source_system: String,
    pub source_object_type: String,
    pub status: MigrationJobStatus,
    pub source_record_count: i32,
    pub valid_record_count: i32,
    pub imported_record_count: i32,
    pub error_count: i32,
    pub created_by: UserId,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationRunType {
    DryRun,
    Import,
    Rollback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationRunStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationRun {
    pub id: MigrationRunId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub run_type: MigrationRunType,
    pub status: MigrationRunStatus,
    pub records_processed: i32,
    pub records_created: i32,
    pub records_updated: i32,
    pub records_skipped: i32,
    pub records_failed: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationSourceRecord {
    pub id: MigrationSourceRecordId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub batch_id: Option<MigrationBatchId>,
    pub external_id: Option<String>,
    pub source_object_type: String,
    pub raw_data: serde_json::Value,
    pub status: String,
    pub row_number: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationStagedRecord {
    pub id: MigrationStagedRecordId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub source_record_id: MigrationSourceRecordId,
    pub target_entity_type: String,
    pub canonical_data: serde_json::Value,
    pub status: String,
    pub validation_errors: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationFieldMapping {
    pub id: MigrationFieldMappingId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub target_entity_type: String,
    pub source_field: String,
    pub target_field: String,
    pub transform_expression: Option<String>,
    pub default_value: Option<String>,
    pub is_required: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationImportResult {
    pub id: MigrationImportResultId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub run_id: MigrationRunId,
    pub staged_record_id: MigrationStagedRecordId,
    pub sip_entity_type: String,
    pub sip_entity_id: Option<Uuid>,
    pub action: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationExternalIdMap {
    pub id: MigrationExternalIdMapId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub run_id: MigrationRunId,
    pub source_system: String,
    pub source_object_type: String,
    pub source_external_id: String,
    pub sip_entity_type: String,
    pub sip_entity_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationBatch {
    pub id: MigrationBatchId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub batch_number: i32,
    pub record_count: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationValidationIssue {
    pub id: MigrationValidationIssueId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub staged_record_id: MigrationStagedRecordId,
    pub severity: String,
    pub field: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrationDuplicateCandidate {
    pub id: MigrationDuplicateCandidateId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub staged_record_id: MigrationStagedRecordId,
    pub sip_entity_type: String,
    pub sip_entity_id: Uuid,
    pub confidence_score: f64,
    pub match_reason: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationCheckpoint {
    pub id: MigrationCheckpointId,
    pub organization_id: OrganizationId,
    pub job_id: MigrationJobId,
    pub run_id: MigrationRunId,
    pub checkpoint_type: String,
    pub state_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ── Canonical Import DTOs ──

pub const CANONICAL_IMPORT_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalOrganizationImport {
    pub external_id: Option<String>,
    pub name: String,
    pub slug: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalCustomerImport {
    pub external_id: Option<String>,
    pub name: String,
    pub account_number: Option<String>,
    pub org_external_id: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalContactImport {
    pub external_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub customer_external_id: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalLocationImport {
    pub external_id: Option<String>,
    pub name: String,
    pub location_type: Option<String>,
    pub parent_external_id: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
    pub active: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAssetTypeImport {
    pub external_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub json_schema: Option<serde_json::Value>,
    pub icon: Option<String>,
    pub parent_type_external_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalManufacturerImport {
    pub external_id: Option<String>,
    pub name: String,
    pub website: Option<String>,
    pub support_phone: Option<String>,
    pub support_email: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAssetModelImport {
    pub external_id: Option<String>,
    pub name: String,
    pub manufacturer_external_id: Option<String>,
    pub asset_type_external_id: Option<String>,
    pub description: Option<String>,
    pub specifications: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAssetImport {
    pub external_id: Option<String>,
    pub name: String,
    pub asset_type_external_id: Option<String>,
    pub model_external_id: Option<String>,
    pub location_external_id: Option<String>,
    pub parent_external_id: Option<String>,
    pub serial_number: Option<String>,
    pub status: Option<String>,
    pub criticality: Option<String>,
    pub purchase_date: Option<String>,
    pub warranty_expiry: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub custom_attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalWorkOrderImport {
    pub external_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub asset_external_id: Option<String>,
    pub priority: Option<String>,
    #[serde(rename = "type")]
    pub work_order_type: Option<String>,
    pub status: Option<String>,
    pub assigned_to_email: Option<String>,
    pub due_date: Option<String>,
    pub completed_date: Option<String>,
    pub resolution_notes: Option<String>,
    pub parts_used: Option<Vec<CanonicalPartUsageImport>>,
    pub labor_hours: Option<f64>,
    pub source_external_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPartUsageImport {
    pub part_external_id: Option<String>,
    pub quantity: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalWorkOrderActivityImport {
    pub external_id: Option<String>,
    pub work_order_external_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: Option<String>,
    pub description: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPartImport {
    pub external_id: Option<String>,
    pub name: String,
    pub part_number: Option<String>,
    pub manufacturer_external_id: Option<String>,
    pub description: Option<String>,
    pub quantity_on_hand: Option<f64>,
    pub unit_of_measure: Option<String>,
    pub unit_cost: Option<f64>,
    pub reorder_point: Option<f64>,
    pub location: Option<String>,
    pub category: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDocumentImport {
    pub external_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub asset_external_id: Option<String>,
    pub work_order_external_id: Option<String>,
    pub document_type: Option<String>,
    pub url: Option<String>,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_job_status_transitions() {
        assert!(MigrationJobStatus::Draft.can_transition_to(MigrationJobStatus::Uploaded));
        assert!(MigrationJobStatus::Uploaded.can_transition_to(MigrationJobStatus::Mapped));
        assert!(MigrationJobStatus::Mapped.can_transition_to(MigrationJobStatus::Validated));
        assert!(MigrationJobStatus::Validated.can_transition_to(MigrationJobStatus::ReadyForImport));
        assert!(MigrationJobStatus::ReadyForImport.can_transition_to(MigrationJobStatus::Importing));
        assert!(MigrationJobStatus::Importing.can_transition_to(MigrationJobStatus::Completed));
        assert!(MigrationJobStatus::Importing
            .can_transition_to(MigrationJobStatus::CompletedWithWarnings));
        assert!(MigrationJobStatus::Importing.can_transition_to(MigrationJobStatus::Failed));
        assert!(MigrationJobStatus::Completed.can_transition_to(MigrationJobStatus::RolledBack));
        assert!(MigrationJobStatus::CompletedWithWarnings
            .can_transition_to(MigrationJobStatus::RolledBack));
        assert!(MigrationJobStatus::Failed.can_transition_to(MigrationJobStatus::Draft));
        assert!(MigrationJobStatus::Cancelled.can_transition_to(MigrationJobStatus::Draft));
    }

    #[test]
    fn test_canonical_import_version() {
        assert_eq!(CANONICAL_IMPORT_VERSION, "1.0");
    }

    #[test]
    fn test_canonical_location_import_serialization() {
        let loc = CanonicalLocationImport {
            external_id: Some("LOC-001".into()),
            name: "Main Building".into(),
            location_type: Some("building".into()),
            parent_external_id: None,
            address_line1: Some("123 Main St".into()),
            address_line2: None,
            city: Some("Springfield".into()),
            state: Some("IL".into()),
            postal_code: Some("62701".into()),
            country: Some("US".into()),
            latitude: Some(39.7817),
            longitude: Some(-89.6501),
            timezone: Some("America/Chicago".into()),
            active: Some(true),
            notes: None,
        };
        let json = serde_json::to_string(&loc).unwrap();
        let parsed: CanonicalLocationImport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Main Building");
        assert_eq!(parsed.external_id, Some("LOC-001".into()));
    }

    #[test]
    fn test_canonical_asset_import_serialization() {
        let asset = CanonicalAssetImport {
            external_id: Some("AST-001".into()),
            name: "Chiller Unit 3".into(),
            asset_type_external_id: Some("ATYPE-HVAC".into()),
            model_external_id: None,
            location_external_id: Some("LOC-001".into()),
            parent_external_id: None,
            serial_number: Some("SN-12345".into()),
            status: Some("Operational".into()),
            criticality: Some("High".into()),
            purchase_date: Some("2023-01-15".into()),
            warranty_expiry: Some("2026-01-15".into()),
            notes: Some("Installed in mechanical room B".into()),
            tags: Some(vec!["hvac".into(), "critical".into()]),
            custom_attributes: None,
        };
        let json = serde_json::to_string(&asset).unwrap();
        let parsed: CanonicalAssetImport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Chiller Unit 3");
        assert_eq!(parsed.serial_number, Some("SN-12345".into()));
    }

    #[test]
    fn test_canonical_work_order_import_serialization() {
        let wo = CanonicalWorkOrderImport {
            external_id: Some("WO-EXT-001".into()),
            title: "Replace bearing".into(),
            description: Some("Bearing making noise".into()),
            asset_external_id: Some("AST-001".into()),
            priority: Some("High".into()),
            work_order_type: Some("Corrective".into()),
            status: Some("Open".into()),
            assigned_to_email: None,
            due_date: Some("2026-05-15".into()),
            completed_date: None,
            resolution_notes: None,
            parts_used: Some(vec![CanonicalPartUsageImport {
                part_external_id: Some("PART-001".into()),
                quantity: 2.0,
                notes: Some("SKF bearings".into()),
            }]),
            labor_hours: Some(1.5),
            source_external_id: None,
        };
        let json = serde_json::to_string(&wo).unwrap();
        let parsed: CanonicalWorkOrderImport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, "Replace bearing");
        assert_eq!(parsed.parts_used.unwrap().len(), 1);
    }

    #[test]
    fn test_id_newtypes_display_and_from_uuid() {
        use uuid::Uuid;
        let uuid = Uuid::new_v4();
        let job_id: MigrationJobId = uuid.into();
        assert_eq!(job_id.to_string(), uuid.to_string());
        assert_eq!(Uuid::from(job_id), uuid);
    }
}
