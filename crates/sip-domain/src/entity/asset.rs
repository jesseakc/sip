use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{AssetId, AssetModelId, AssetTypeId, LocationId, OrganizationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetStatus {
    Operational,
    Degraded,
    Down,
    Maintenance,
    Retired,
}

impl AssetStatus {
    pub fn can_transition_to(self, next: AssetStatus) -> bool {
        matches!(
            (self, next),
            (AssetStatus::Operational, AssetStatus::Degraded)
                | (AssetStatus::Operational, AssetStatus::Down)
                | (AssetStatus::Operational, AssetStatus::Maintenance)
                | (AssetStatus::Degraded, AssetStatus::Operational)
                | (AssetStatus::Degraded, AssetStatus::Down)
                | (AssetStatus::Degraded, AssetStatus::Maintenance)
                | (AssetStatus::Degraded, AssetStatus::Retired)
                | (AssetStatus::Down, AssetStatus::Operational)
                | (AssetStatus::Down, AssetStatus::Maintenance)
                | (AssetStatus::Down, AssetStatus::Retired)
                | (AssetStatus::Maintenance, AssetStatus::Operational)
                | (AssetStatus::Maintenance, AssetStatus::Down)
                | (AssetStatus::Maintenance, AssetStatus::Degraded)
                | (AssetStatus::Maintenance, AssetStatus::Retired)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Criticality {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetId,
    pub organization_id: OrganizationId,
    pub location_id: Option<LocationId>,
    pub parent_id: Option<AssetId>,
    pub asset_type_id: AssetTypeId,
    pub model_id: Option<AssetModelId>,
    pub name: String,
    pub description: Option<String>,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub software_version: Option<String>,
    pub hardware_revision: Option<String>,
    pub status: AssetStatus,
    pub version: i32,
    pub criticality: Criticality,
    pub installed_date: Option<chrono::NaiveDate>,
    pub warranty_expiry: Option<chrono::NaiveDate>,
    pub attributes: Option<serde_json::Value>,
    pub tags: Vec<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
