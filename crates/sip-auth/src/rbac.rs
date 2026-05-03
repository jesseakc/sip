use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Permission(pub String);

impl From<&str> for Permission {
    fn from(value: &str) -> Self {
        Permission(value.to_string())
    }
}

impl From<String> for Permission {
    fn from(value: String) -> Self {
        Permission(value)
    }
}

pub fn has_permission(user_permissions: &[String], required: &str) -> bool {
    user_permissions.iter().any(|p| p == required || p == "*")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    Manager,
    Technician,
    Viewer,
    Vendor,
    Auditor,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "ADMIN",
            Role::Manager => "MANAGER",
            Role::Technician => "TECHNICIAN",
            Role::Viewer => "VIEWER",
            Role::Vendor => "VENDOR",
            Role::Auditor => "AUDITOR",
        }
    }
}

impl TryFrom<&str> for Role {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "ADMIN" => Ok(Role::Admin),
            "MANAGER" => Ok(Role::Manager),
            "TECHNICIAN" => Ok(Role::Technician),
            "VIEWER" => Ok(Role::Viewer),
            "VENDOR" => Ok(Role::Vendor),
            "AUDITOR" => Ok(Role::Auditor),
            other => Err(format!("unknown role: {other}")),
        }
    }
}

pub struct RolePermissions;

impl RolePermissions {
    pub fn permissions_for(role: Role) -> Vec<String> {
        match role {
            Role::Admin => vec![
                "asset:read",
                "asset:create",
                "asset:update",
                "asset:delete",
                "work_order:read",
                "work_order:create",
                "work_order:update",
                "work_order:delete",
                "user:read",
                "user:create",
                "user:update",
                "user:delete",
                "reports:read",
                "reports:create",
                "settings:read",
                "settings:update",
                "activity:read",
                "*",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            Role::Manager => vec![
                "asset:read",
                "asset:create",
                "asset:update",
                "asset:delete",
                "work_order:read",
                "work_order:create",
                "work_order:update",
                "work_order:delete",
                "user:read",
                "user:create",
                "user:update",
                "reports:read",
                "reports:create",
                "settings:read",
                "settings:update",
                "activity:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            Role::Technician => vec![
                "asset:read",
                "work_order:read",
                "work_order:create",
                "work_order:update",
                "part:read",
                "part:create",
                "part:update",
                "activity:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            Role::Viewer => vec![
                "asset:read",
                "work_order:read",
                "part:read",
                "reports:read",
                "activity:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            Role::Vendor => vec![
                "assets:read_assigned",
                "work_orders:read_assigned",
                "part:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            Role::Auditor => vec![
                "activity:read",
                "reports:read",
                "asset:read",
                "work_order:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        }
    }

    pub fn default_map() -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        for role in [
            Role::Admin,
            Role::Manager,
            Role::Technician,
            Role::Viewer,
            Role::Vendor,
            Role::Auditor,
        ] {
            map.insert(role.as_str().to_string(), Self::permissions_for(role));
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::Admin.as_str(), "ADMIN");
        assert_eq!(Role::Manager.as_str(), "MANAGER");
        assert_eq!(Role::Technician.as_str(), "TECHNICIAN");
        assert_eq!(Role::Viewer.as_str(), "VIEWER");
        assert_eq!(Role::Vendor.as_str(), "VENDOR");
        assert_eq!(Role::Auditor.as_str(), "AUDITOR");
    }

    #[test]
    fn test_role_try_from_valid() {
        assert_eq!(Role::try_from("ADMIN").unwrap(), Role::Admin);
        assert_eq!(Role::try_from("Manager").unwrap(), Role::Manager);
        assert_eq!(Role::try_from("technician").unwrap(), Role::Technician);
        assert_eq!(Role::try_from("Viewer").unwrap(), Role::Viewer);
        assert_eq!(Role::try_from("VENDOR").unwrap(), Role::Vendor);
        assert_eq!(Role::try_from("auditor").unwrap(), Role::Auditor);
    }

    #[test]
    fn test_role_try_from_invalid() {
        let err = Role::try_from("SUPERUSER").unwrap_err();
        assert!(err.contains("unknown role"));
    }

    #[test]
    fn test_permission_from_str() {
        let p: Permission = "asset:read".into();
        assert_eq!(p.0, "asset:read");
    }

    #[test]
    fn test_permission_from_string() {
        let p: Permission = String::from("work_order:create").into();
        assert_eq!(p.0, "work_order:create");
    }

    #[test]
    fn test_has_permission_exact_match() {
        let perms = vec!["asset:read".to_string(), "work_order:update".to_string()];
        assert!(has_permission(&perms, "asset:read"));
        assert!(has_permission(&perms, "work_order:update"));
    }

    #[test]
    fn test_has_permission_not_found() {
        let perms = vec!["asset:read".to_string()];
        assert!(!has_permission(&perms, "asset:delete"));
    }

    #[test]
    fn test_has_permission_wildcard() {
        let perms = vec!["*".to_string()];
        assert!(has_permission(&perms, "asset:delete"));
        assert!(has_permission(&perms, "anything:any"));
    }

    #[test]
    fn test_has_permission_empty() {
        let perms: Vec<String> = vec![];
        assert!(!has_permission(&perms, "asset:read"));
    }

    #[test]
    fn test_admin_has_wildcard() {
        let perms = RolePermissions::permissions_for(Role::Admin);
        assert!(perms.contains(&"*".to_string()));
    }

    #[test]
    fn test_admin_has_all_crud() {
        let perms = RolePermissions::permissions_for(Role::Admin);
        assert!(perms.contains(&"asset:create".to_string()));
        assert!(perms.contains(&"asset:delete".to_string()));
        assert!(perms.contains(&"user:delete".to_string()));
    }

    #[test]
    fn test_manager_permissions() {
        let perms = RolePermissions::permissions_for(Role::Manager);
        assert!(perms.contains(&"asset:create".to_string()));
        assert!(perms.contains(&"asset:delete".to_string()));
        assert!(!perms.contains(&"user:delete".to_string()));
        assert!(!perms.contains(&"*".to_string()));
    }

    #[test]
    fn test_technician_permissions() {
        let perms = RolePermissions::permissions_for(Role::Technician);
        assert!(perms.contains(&"work_order:create".to_string()));
        assert!(!perms.contains(&"user:create".to_string()));
        assert!(!perms.contains(&"asset:delete".to_string()));
    }

    #[test]
    fn test_viewer_is_read_only() {
        let perms = RolePermissions::permissions_for(Role::Viewer);
        assert!(perms.iter().all(|p| p.ends_with(":read")));
    }

    #[test]
    fn test_vendor_permissions() {
        let perms = RolePermissions::permissions_for(Role::Vendor);
        assert!(perms.contains(&"assets:read_assigned".to_string()));
        assert!(!perms.contains(&"user:read".to_string()));
    }

    #[test]
    fn test_auditor_permissions() {
        let perms = RolePermissions::permissions_for(Role::Auditor);
        assert!(perms.contains(&"activity:read".to_string()));
        assert!(!perms.contains(&"asset:create".to_string()));
    }

    #[test]
    fn test_default_map_contains_all_roles() {
        let map = RolePermissions::default_map();
        assert_eq!(map.len(), 6);
        assert!(map.contains_key("ADMIN"));
        assert!(map.contains_key("MANAGER"));
        assert!(map.contains_key("TECHNICIAN"));
        assert!(map.contains_key("VIEWER"));
        assert!(map.contains_key("VENDOR"));
        assert!(map.contains_key("AUDITOR"));
    }

    #[test]
    fn test_default_map_matches_permissions_for() {
        let map = RolePermissions::default_map();
        for role in [
            Role::Admin,
            Role::Manager,
            Role::Technician,
            Role::Viewer,
            Role::Vendor,
            Role::Auditor,
        ] {
            assert_eq!(
                map.get(role.as_str()).unwrap(),
                &RolePermissions::permissions_for(role)
            );
        }
    }
}
