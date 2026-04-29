use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{OrganizationId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Manager,
    Technician,
    Viewer,
    Vendor,
    Auditor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub organization_id: OrganizationId,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub skills: Vec<String>,
    pub certifications: Option<Vec<serde_json::Value>>,
    pub working_hours: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
