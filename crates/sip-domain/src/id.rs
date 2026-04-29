use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl From<$name> for Uuid {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::from_str(s)?))
            }
        }
    };
}

define_id!(OrganizationId);
define_id!(LocationId);
define_id!(AssetId);
define_id!(AssetTypeId);
define_id!(ManufacturerId);
define_id!(AssetModelId);
define_id!(WorkOrderId);
define_id!(UserId);
define_id!(TeamId);
define_id!(DocumentId);
define_id!(PartId);
define_id!(ScheduleId);
define_id!(InspectionId);
define_id!(InspectionChecklistItemId);
define_id!(PartUsageId);
define_id!(DocumentLinkId);
define_id!(DocumentChunkId);
define_id!(EmbeddingRecordId);
define_id!(ActivityId);
define_id!(AIConversationId);
define_id!(AIMessageId);
define_id!(AIRetrievalTraceId);
define_id!(AgentIdentityId);
define_id!(IdempotencyKeyId);
define_id!(OrganizationSettingsId);
define_id!(AssetPartId);
define_id!(OutboxEventId);
define_id!(WorkOrderAssignmentId);
define_id!(WorkOrderStatusHistoryId);
define_id!(PluginId);
