use serde::{Deserialize, Serialize};

use crate::id::{AgentIdentityId, OrganizationId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub id: AgentIdentityId,
    pub organization_id: Option<OrganizationId>,
    pub name: String,
    pub agent_type: String,
    pub provider: String,
    pub model: Option<String>,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
