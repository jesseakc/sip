use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
}

impl TenantContext {
    pub fn new(organization_id: Uuid) -> Self {
        Self {
            organization_id,
            user_id: None,
        }
    }
}
