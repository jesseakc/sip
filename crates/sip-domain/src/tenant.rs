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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::OrganizationId;

    fn make_ctx() -> TenantContext {
        TenantContext::new(OrganizationId::new(), None)
    }

    #[test]
    fn test_has_permission_exact_match() {
        let ctx = make_ctx().with_permissions(vec!["asset:read".to_string()]);
        assert!(ctx.has_permission("asset:read"));
    }

    #[test]
    fn test_has_permission_not_found() {
        let ctx = make_ctx().with_permissions(vec!["asset:read".to_string()]);
        assert!(!ctx.has_permission("asset:delete"));
    }

    #[test]
    fn test_has_permission_wildcard() {
        let ctx = make_ctx().with_permissions(vec!["*".to_string()]);
        assert!(ctx.has_permission("asset:delete"));
        assert!(ctx.has_permission("anything:any"));
    }

    #[test]
    fn test_has_permission_empty() {
        let ctx = make_ctx();
        assert!(!ctx.has_permission("asset:read"));
    }

    #[test]
    fn test_has_permission_multiple_with_wildcard() {
        let ctx = make_ctx().with_permissions(vec!["asset:read".to_string(), "*".to_string()]);
        assert!(ctx.has_permission("asset:read"));
        assert!(ctx.has_permission("asset:delete"));
    }

    #[test]
    fn test_require_permission_success() {
        let ctx = make_ctx().with_permissions(vec!["asset:read".to_string()]);
        assert!(ctx.require_permission("asset:read").is_ok());
    }

    #[test]
    fn test_require_permission_denied() {
        let ctx = make_ctx().with_permissions(vec!["asset:read".to_string()]);
        let err = ctx.require_permission("asset:delete").unwrap_err();
        assert_eq!(err, SipError::PermissionDenied);
    }

    #[test]
    fn test_with_permissions_replaces() {
        let ctx = make_ctx()
            .with_permissions(vec!["a:read".to_string()])
            .with_permissions(vec!["b:write".to_string()]);
        assert!(ctx.has_permission("b:write"));
        assert!(!ctx.has_permission("a:read"));
    }
}
