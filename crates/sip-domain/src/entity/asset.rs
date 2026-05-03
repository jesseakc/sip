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

    pub fn is_terminal(self) -> bool {
        matches!(self, AssetStatus::Retired)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── AssetStatus is_terminal ───

    #[test]
    fn test_asset_retired_is_terminal() {
        assert!(AssetStatus::Retired.is_terminal());
    }

    #[test]
    fn test_asset_non_terminal_states() {
        assert!(!AssetStatus::Operational.is_terminal());
        assert!(!AssetStatus::Degraded.is_terminal());
        assert!(!AssetStatus::Down.is_terminal());
        assert!(!AssetStatus::Maintenance.is_terminal());
    }

    // ─── AssetStatus transitions (boundary) ───

    #[test]
    fn test_asset_retired_has_no_transitions() {
        assert!(!AssetStatus::Retired.can_transition_to(AssetStatus::Operational));
        assert!(!AssetStatus::Retired.can_transition_to(AssetStatus::Degraded));
        assert!(!AssetStatus::Retired.can_transition_to(AssetStatus::Down));
        assert!(!AssetStatus::Retired.can_transition_to(AssetStatus::Maintenance));
    }

    #[test]
    fn test_asset_operational_cannot_jump_to_retired() {
        assert!(!AssetStatus::Operational.can_transition_to(AssetStatus::Retired));
    }

    // ─── AssetStatus serde ───

    #[test]
    fn test_asset_status_serde_all_roundtrip() {
        for status in [
            AssetStatus::Operational,
            AssetStatus::Degraded,
            AssetStatus::Down,
            AssetStatus::Maintenance,
            AssetStatus::Retired,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: AssetStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }

    // ─── Criticality serde ───

    #[test]
    fn test_criticality_serde_roundtrip() {
        for c in [
            Criticality::Low,
            Criticality::Medium,
            Criticality::High,
            Criticality::Critical,
        ] {
            let json = serde_json::to_string(&c).unwrap();
            let back: Criticality = serde_json::from_str(&json).unwrap();
            assert_eq!(back, c);
        }
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
