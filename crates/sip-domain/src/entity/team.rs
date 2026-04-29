use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{OrganizationId, TeamId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub lead_id: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
