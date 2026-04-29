use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::OutboxEventId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutboxStatus {
    Pending,
    Processing,
    Published,
    Failed,
    DeadLetter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxEvent {
    pub id: OutboxEventId,
    pub aggregate_type: String,
    pub aggregate_id: uuid::Uuid,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: OutboxStatus,
    pub attempts: i32,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub next_attempt_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}
