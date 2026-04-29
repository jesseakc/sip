use serde::{Deserialize, Serialize};

use crate::id::OrganizationId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub id: OrganizationId,
    pub timezone: String,
    pub default_currency: String,
    pub feature_flags: Option<serde_json::Value>,
    pub ai_config: Option<serde_json::Value>,
    pub notification_preferences: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
