use serde::{Deserialize, Serialize};

use crate::error::SipError;
use crate::id::{OrganizationId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantContext {
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub permissions: Vec<String>,
}

impl TenantContext {
    pub fn new(organization_id: OrganizationId, user_id: Option<UserId>) -> Self {
        Self {
            organization_id,
            user_id,
            permissions: Vec::new(),
        }
    }

    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&"*".to_string())
            || self.permissions.contains(&permission.to_string())
    }

    pub fn require_permission(&self, permission: &str) -> Result<(), SipError> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(SipError::PermissionDenied)
        }
    }
}
