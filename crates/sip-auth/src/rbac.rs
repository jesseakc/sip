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
                "assets:read",
                "assets:create",
                "assets:update",
                "assets:delete",
                "work_orders:read",
                "work_orders:create",
                "work_orders:update",
                "work_orders:delete",
                "users:read",
                "users:create",
                "users:update",
                "users:delete",
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
                "assets:read",
                "assets:create",
                "assets:update",
                "assets:delete",
                "work_orders:read",
                "work_orders:create",
                "work_orders:update",
                "work_orders:delete",
                "users:read",
                "users:create",
                "users:update",
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
                "assets:read",
                "work_orders:read",
                "work_orders:create",
                "work_orders:update",
                "parts:read",
                "parts:create",
                "parts:update",
                "activity:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            Role::Viewer => vec![
                "assets:read",
                "work_orders:read",
                "parts:read",
                "reports:read",
                "activity:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            Role::Vendor => vec![
                "assets:read_assigned",
                "work_orders:read_assigned",
                "parts:read",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            Role::Auditor => vec![
                "activity:read",
                "reports:read",
                "assets:read",
                "work_orders:read",
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
