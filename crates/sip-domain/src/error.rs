use thiserror::Error;

use crate::entity::work_order::WorkOrderStatus;
use crate::entity::asset::AssetStatus;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SipError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("permission denied")]
    PermissionDenied,

    #[error("tenant scope violation")]
    TenantScopeViolation,

    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: String,
        to: String,
    },

    #[error("version conflict: current={current_version}, submitted={submitted_version}")]
    VersionConflict {
        current_version: i32,
        submitted_version: i32,
    },

    #[error("capability not available: {0}")]
    CapabilityNotAvailable(String),
}

impl SipError {
    pub fn invalid_state_transition_work_order(from: WorkOrderStatus, to: WorkOrderStatus) -> Self {
        SipError::InvalidStateTransition {
            from: format!("{:?}", from),
            to: format!("{:?}", to),
        }
    }

    pub fn invalid_state_transition_asset(from: AssetStatus, to: AssetStatus) -> Self {
        SipError::InvalidStateTransition {
            from: format!("{:?}", from),
            to: format!("{:?}", to),
        }
    }
}
