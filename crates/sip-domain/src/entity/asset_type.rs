use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{AssetTypeId, OrganizationId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetType {
    pub id: AssetTypeId,
    pub organization_id: Option<OrganizationId>,
    pub name: String,
    pub category: String,
    pub description: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub default_pm_schedules: Option<Vec<serde_json::Value>>,
    pub default_inspection_template: Option<serde_json::Value>,
    pub icon: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
