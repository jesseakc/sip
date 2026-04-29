use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{AgentIdentityId, OrganizationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Knowledge,
    Scheduling,
    Diagnostic,
    Dispatch,
    Compliance,
    Inventory,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub id: AgentIdentityId,
    pub organization_id: Option<OrganizationId>,
    pub name: String,
    pub agent_type: AgentType,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub permissions: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}
