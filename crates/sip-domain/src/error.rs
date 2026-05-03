use thiserror::Error;

use crate::entity::asset::AssetStatus;
use crate::entity::work_order::WorkOrderStatus;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum SipError {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("permission denied")]
    PermissionDenied,

    #[error("tenant scope violation")]
    TenantScopeViolation,

    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: String, to: String },

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::asset::AssetStatus;
    use crate::entity::work_order::WorkOrderStatus;

    #[test]
    fn test_validation_display() {
        let err = SipError::Validation("bad input".into());
        assert_eq!(err.to_string(), "validation error: bad input");
    }

    #[test]
    fn test_permission_denied_display() {
        assert_eq!(SipError::PermissionDenied.to_string(), "permission denied");
    }

    #[test]
    fn test_tenant_scope_violation_display() {
        assert_eq!(SipError::TenantScopeViolation.to_string(), "tenant scope violation");
    }

    #[test]
    fn test_invalid_state_transition_display() {
        let err = SipError::InvalidStateTransition { from: "Draft".into(), to: "Closed".into() };
        assert_eq!(err.to_string(), "invalid state transition from \"Draft\" to \"Closed\"");
    }

    #[test]
    fn test_version_conflict_display() {
        let err = SipError::VersionConflict { current_version: 5, submitted_version: 3 };
        assert_eq!(err.to_string(), "version conflict: current=5, submitted=3");
    }

    #[test]
    fn test_capability_not_available_display() {
        let err = SipError::CapabilityNotAvailable("gemini".into());
        assert_eq!(err.to_string(), "capability not available: gemini");
    }

    #[test]
    fn test_invalid_state_transition_work_order_constructor() {
        let err = SipError::invalid_state_transition_work_order(WorkOrderStatus::Draft, WorkOrderStatus::Closed);
        assert!(matches!(err, SipError::InvalidStateTransition { .. }));
    }

    #[test]
    fn test_invalid_state_transition_asset_constructor() {
        let err = SipError::invalid_state_transition_asset(AssetStatus::Operational, AssetStatus::Retired);
        assert!(matches!(err, SipError::InvalidStateTransition { .. }));
    }

    #[test]
    fn test_error_equality() {
        assert_eq!(SipError::Validation("x".into()), SipError::Validation("x".into()));
        assert_ne!(SipError::Validation("x".into()), SipError::Validation("y".into()));
        assert_eq!(SipError::PermissionDenied, SipError::PermissionDenied);
        assert_ne!(SipError::PermissionDenied, SipError::TenantScopeViolation);
    }
}
