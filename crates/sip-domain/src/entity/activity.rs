use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{ActivityId, AgentIdentityId, OrganizationId, PluginId};
use crate::entity::work_order::ActorType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivitySource {
    Api,
    Ui,
    Ai,
    Automation,
    Plugin,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    pub id: ActivityId,
    pub organization_id: OrganizationId,
    pub actor_id: Option<crate::id::UserId>,
    pub actor_type: ActorType,
    pub agent_identity_id: Option<AgentIdentityId>,
    pub plugin_id: Option<PluginId>,
    pub entity_type: String,
    pub entity_id: uuid::Uuid,
    pub action: String,
    pub changes: Option<serde_json::Value>,
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
    pub source: ActivitySource,
    pub reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}
