use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::OrganizationId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub timezone: Option<String>,
    pub default_currency: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OrganizationSettings {
    pub feature_flags: Option<serde_json::Value>,
    pub ai_config: Option<serde_json::Value>,
    pub notification_preferences: Option<serde_json::Value>,
}
