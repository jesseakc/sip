use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::id::{AssetId, OrganizationId, ScheduleId, UserId, WorkOrderId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkOrderStatus {
    Draft,
    Open,
    Assigned,
    Accepted,
    InProgress,
    OnHold,
    Completed,
    Reviewed,
    Closed,
    Cancelled,
}

impl WorkOrderStatus {
    pub fn can_transition_to(self, next: WorkOrderStatus) -> bool {
        matches!(
            (self, next),
            (WorkOrderStatus::Draft, WorkOrderStatus::Open)
                | (WorkOrderStatus::Draft, WorkOrderStatus::Cancelled)
                | (WorkOrderStatus::Open, WorkOrderStatus::Assigned)
                | (WorkOrderStatus::Open, WorkOrderStatus::Cancelled)
                | (WorkOrderStatus::Assigned, WorkOrderStatus::Accepted)
                | (WorkOrderStatus::Assigned, WorkOrderStatus::Cancelled)
                | (WorkOrderStatus::Accepted, WorkOrderStatus::InProgress)
                | (WorkOrderStatus::Accepted, WorkOrderStatus::Cancelled)
                | (WorkOrderStatus::InProgress, WorkOrderStatus::OnHold)
                | (WorkOrderStatus::InProgress, WorkOrderStatus::Completed)
                | (WorkOrderStatus::InProgress, WorkOrderStatus::Cancelled)
                | (WorkOrderStatus::OnHold, WorkOrderStatus::InProgress)
                | (WorkOrderStatus::OnHold, WorkOrderStatus::Cancelled)
                | (WorkOrderStatus::Completed, WorkOrderStatus::Reviewed)
                | (WorkOrderStatus::Reviewed, WorkOrderStatus::Closed)
                | (WorkOrderStatus::Closed, WorkOrderStatus::Open)
                | (WorkOrderStatus::Closed, WorkOrderStatus::InProgress)
                | (WorkOrderStatus::Closed, WorkOrderStatus::Cancelled)
        )
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, WorkOrderStatus::Closed | WorkOrderStatus::Cancelled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkOrderType {
    Preventive,
    Corrective,
    Inspection,
    Emergency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkOrderPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkOrderSourceType {
    Manual,
    Schedule,
    AiAgent,
    Api,
    Import,
    Plugin,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkOrder {
    pub id: WorkOrderId,
    pub organization_id: OrganizationId,
    pub asset_id: AssetId,
    pub parent_id: Option<WorkOrderId>,
    pub schedule_id: Option<ScheduleId>,
    pub work_order_type: WorkOrderType,
    pub priority: WorkOrderPriority,
    pub status: WorkOrderStatus,
    pub title: String,
    pub display_number: String,
    pub description: String,
    pub scheduled_start: Option<DateTime<Utc>>,
    pub scheduled_end: Option<DateTime<Utc>>,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,
    pub estimated_hours: Option<rust_decimal::Decimal>,
    pub actual_hours: Option<rust_decimal::Decimal>,
    pub resolution_notes: Option<String>,
    pub failure_code: Option<String>,
    pub root_cause: Option<String>,
    pub created_by_id: UserId,
    pub source_type: WorkOrderSourceType,
    pub source_system: Option<String>,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
    pub reopened_count: i32,
    pub last_reopened_at: Option<DateTime<Utc>>,
    pub last_reopened_by_id: Option<UserId>,
    pub version: i32,
    pub archived_at: Option<DateTime<Utc>>,
    pub archived_by_id: Option<UserId>,
    pub archive_reason: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssigneeType {
    User,
    Team,
    Vendor,
    AiAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentRole {
    Primary,
    Secondary,
    Observer,
    Approver,
    DispatchedTech,
    RemoteSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentStatus {
    Assigned,
    Accepted,
    Declined,
    Removed,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkOrderAssignment {
    pub id: crate::id::WorkOrderAssignmentId,
    pub organization_id: OrganizationId,
    pub work_order_id: WorkOrderId,
    pub assignee_type: AssigneeType,
    pub assignee_id: crate::id::UserId,
    pub role: AssignmentRole,
    pub assigned_at: DateTime<Utc>,
    pub assigned_by: UserId,
    pub accepted_at: Option<DateTime<Utc>>,
    pub removed_at: Option<DateTime<Utc>>,
    pub status: AssignmentStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorType {
    Human,
    AiAgent,
    System,
    Plugin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkOrderStatusHistory {
    pub id: crate::id::WorkOrderStatusHistoryId,
    pub organization_id: OrganizationId,
    pub work_order_id: WorkOrderId,
    pub from_status: Option<WorkOrderStatus>,
    pub to_status: WorkOrderStatus,
    pub changed_by_id: Option<UserId>,
    pub actor_type: ActorType,
    pub agent_identity_id: Option<crate::id::AgentIdentityId>,
    pub plugin_id: Option<crate::id::PluginId>,
    pub reason: Option<String>,
    pub created_at: DateTime<Utc>,
}
