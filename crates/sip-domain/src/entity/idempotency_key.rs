use serde::{Deserialize, Serialize};

use crate::id::{IdempotencyKeyId, OrganizationId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdempotencyKey {
    pub id: IdempotencyKeyId,
    pub key: String,
    pub organization_id: OrganizationId,
    pub response_status: i32,
    pub response_body: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
