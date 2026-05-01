use serde::{Deserialize, Serialize};

use crate::id::{AssetId, AssetPartId, OrganizationId, PartId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetPart {
    pub id: AssetPartId,
    pub organization_id: OrganizationId,
    pub asset_id: AssetId,
    pub part_id: PartId,
    pub quantity: i32,
    pub unit: String,
    pub notes: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
