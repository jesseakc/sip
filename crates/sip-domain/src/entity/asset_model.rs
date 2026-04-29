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
