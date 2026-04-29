use serde::{Deserialize, Serialize};

use crate::id::OutboxEventId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: OutboxEventId,
    pub organization_id: crate::id::OrganizationId,
    pub aggregate_type: String,
    pub aggregate_id: uuid::Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
}
