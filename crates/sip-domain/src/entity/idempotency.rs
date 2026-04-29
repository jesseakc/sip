use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{IdempotencyKeyId, OrganizationId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyKey {
    pub id: IdempotencyKeyId,
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub key: String,
    pub request_hash: String,
    pub response_status: i32,
    pub response_body: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
