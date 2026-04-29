use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{OrganizationId, PartId, PartUsageId, UserId, WorkOrderId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    pub id: PartId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub part_number: Option<String>,
    pub description: Option<String>,
    pub quantity_on_hand: rust_decimal::Decimal,
    pub quantity_minimum: Option<rust_decimal::Decimal>,
    pub unit: String,
    pub unit_cost: Option<rust_decimal::Decimal>,
    pub storage_location: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartUsage {
    pub id: PartUsageId,
    pub work_order_id: WorkOrderId,
    pub part_id: PartId,
    pub quantity: rust_decimal::Decimal,
    pub used_by_id: UserId,
}
