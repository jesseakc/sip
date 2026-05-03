use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{AssetModelId, AssetTypeId, ManufacturerId, OrganizationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleStatus {
    Active,
    Deprecated,
    EndOfSupport,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetModel {
    pub id: AssetModelId,
    pub organization_id: Option<OrganizationId>,
    pub manufacturer_id: ManufacturerId,
    pub name: String,
    pub model_number: String,
    pub revision: Option<String>,
    pub lifecycle_status: LifecycleStatus,
    pub asset_type_id: AssetTypeId,
    pub documentation_url: Option<String>,
    pub default_attributes: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_status_serde_roundtrip() {
        for status in [
            LifecycleStatus::Active,
            LifecycleStatus::Deprecated,
            LifecycleStatus::EndOfSupport,
            LifecycleStatus::Retired,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: LifecycleStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }
}
