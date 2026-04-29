use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{LocationId, OrganizationId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LocationType {
    Site,
    Building,
    Floor,
    Room,
    Area,
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub organization_id: OrganizationId,
    pub parent_id: Option<LocationId>,
    pub name: String,
    pub location_type: LocationType,
    pub geo_latitude: Option<f64>,
    pub geo_longitude: Option<f64>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
