use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{ManufacturerId, OrganizationId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manufacturer {
    pub id: ManufacturerId,
    pub organization_id: Option<OrganizationId>,
    pub name: String,
    pub website: Option<String>,
    pub support_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
