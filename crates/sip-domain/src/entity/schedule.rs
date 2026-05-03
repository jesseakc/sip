use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{AssetId, OrganizationId, ScheduleId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleTriggerType {
    Cron,
    Meter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    pub id: ScheduleId,
    pub organization_id: OrganizationId,
    pub asset_id: AssetId,
    pub name: String,
    pub trigger_type: ScheduleTriggerType,
    pub trigger_config: Option<serde_json::Value>,
    pub work_order_template: Option<serde_json::Value>,
    pub next_due: Option<DateTime<Utc>>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
    pub archived_by_id: Option<UserId>,
    pub archive_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_trigger_type_serde_roundtrip() {
        for ty in [ScheduleTriggerType::Cron, ScheduleTriggerType::Meter] {
            let json = serde_json::to_string(&ty).unwrap();
            let back: ScheduleTriggerType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, ty);
        }
    }
}
