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

#[cfg(test)]
mod tests {
    use super::*;

    // ─── WorkOrderStatus::is_terminal ───

    #[test]
    fn test_terminal_states() {
        assert!(WorkOrderStatus::Closed.is_terminal());
        assert!(WorkOrderStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_non_terminal_states() {
        assert!(!WorkOrderStatus::Draft.is_terminal());
        assert!(!WorkOrderStatus::Open.is_terminal());
        assert!(!WorkOrderStatus::Assigned.is_terminal());
        assert!(!WorkOrderStatus::Accepted.is_terminal());
        assert!(!WorkOrderStatus::InProgress.is_terminal());
        assert!(!WorkOrderStatus::OnHold.is_terminal());
        assert!(!WorkOrderStatus::Completed.is_terminal());
        assert!(!WorkOrderStatus::Reviewed.is_terminal());
    }

    // ─── WorkOrderStatus serde ───

    #[test]
    fn test_work_order_status_serde_draft() {
        let json = serde_json::to_string(&WorkOrderStatus::Draft).unwrap();
        let parsed: WorkOrderStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, WorkOrderStatus::Draft);
    }

    #[test]
    fn test_work_order_status_serde_cancelled() {
        let json = serde_json::to_string(&WorkOrderStatus::Cancelled).unwrap();
        let parsed: WorkOrderStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, WorkOrderStatus::Cancelled);
    }

    #[test]
    fn test_work_order_status_all_variants_roundtrip() {
        for status in [
            WorkOrderStatus::Draft,
            WorkOrderStatus::Open,
            WorkOrderStatus::Assigned,
            WorkOrderStatus::Accepted,
            WorkOrderStatus::InProgress,
            WorkOrderStatus::OnHold,
            WorkOrderStatus::Completed,
            WorkOrderStatus::Reviewed,
            WorkOrderStatus::Closed,
            WorkOrderStatus::Cancelled,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: WorkOrderStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }

    // ─── WorkOrderType serde ───

    #[test]
    fn test_work_order_type_serde_roundtrip() {
        for ty in [
            WorkOrderType::Preventive,
            WorkOrderType::Corrective,
            WorkOrderType::Inspection,
            WorkOrderType::Emergency,
        ] {
            let json = serde_json::to_string(&ty).unwrap();
            let back: WorkOrderType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, ty);
        }
    }

    // ─── WorkOrderPriority serde ───

    #[test]
    fn test_work_order_priority_serde_roundtrip() {
        for p in [
            WorkOrderPriority::Low,
            WorkOrderPriority::Medium,
            WorkOrderPriority::High,
            WorkOrderPriority::Critical,
        ] {
            let json = serde_json::to_string(&p).unwrap();
            let back: WorkOrderPriority = serde_json::from_str(&json).unwrap();
            assert_eq!(back, p);
        }
    }

    // ─── WorkOrderSourceType serde ───

    #[test]
    fn test_work_order_source_type_serde_roundtrip() {
        for src in [
            WorkOrderSourceType::Manual,
            WorkOrderSourceType::Schedule,
            WorkOrderSourceType::AiAgent,
            WorkOrderSourceType::Api,
            WorkOrderSourceType::Import,
            WorkOrderSourceType::Plugin,
            WorkOrderSourceType::System,
        ] {
            let json = serde_json::to_string(&src).unwrap();
            let back: WorkOrderSourceType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, src);
        }
    }

    // ─── Assignment enums serde ───

    #[test]
    fn test_assignee_type_serde_roundtrip() {
        for ty in [
            AssigneeType::User,
            AssigneeType::Team,
            AssigneeType::Vendor,
            AssigneeType::AiAgent,
        ] {
            let json = serde_json::to_string(&ty).unwrap();
            let back: AssigneeType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, ty);
        }
    }

    #[test]
    fn test_assignment_role_serde_roundtrip() {
        for role in [
            AssignmentRole::Primary,
            AssignmentRole::Secondary,
            AssignmentRole::Observer,
            AssignmentRole::Approver,
            AssignmentRole::DispatchedTech,
            AssignmentRole::RemoteSupport,
        ] {
            let json = serde_json::to_string(&role).unwrap();
            let back: AssignmentRole = serde_json::from_str(&json).unwrap();
            assert_eq!(back, role);
        }
    }

    #[test]
    fn test_assignment_status_serde_roundtrip() {
        for status in [
            AssignmentStatus::Assigned,
            AssignmentStatus::Accepted,
            AssignmentStatus::Declined,
            AssignmentStatus::Removed,
            AssignmentStatus::Completed,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: AssignmentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }

    #[test]
    fn test_actor_type_serde_roundtrip() {
        for ty in [
            ActorType::Human,
            ActorType::AiAgent,
            ActorType::System,
            ActorType::Plugin,
        ] {
            let json = serde_json::to_string(&ty).unwrap();
            let back: ActorType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, ty);
        }
    }

    // ─── WorkOrderStatus transitions (boundary) ───

    #[test]
    fn test_work_order_cancelled_has_no_transitions() {
        assert!(!WorkOrderStatus::Cancelled.can_transition_to(WorkOrderStatus::Draft));
        assert!(!WorkOrderStatus::Cancelled.can_transition_to(WorkOrderStatus::Open));
        assert!(!WorkOrderStatus::Cancelled.can_transition_to(WorkOrderStatus::InProgress));
        assert!(!WorkOrderStatus::Cancelled.can_transition_to(WorkOrderStatus::Closed));
    }

    #[test]
    fn test_work_order_closed_can_reopen() {
        assert!(WorkOrderStatus::Closed.can_transition_to(WorkOrderStatus::Open));
        assert!(WorkOrderStatus::Closed.can_transition_to(WorkOrderStatus::InProgress));
        assert!(WorkOrderStatus::Closed.can_transition_to(WorkOrderStatus::Cancelled));
        assert!(!WorkOrderStatus::Closed.can_transition_to(WorkOrderStatus::Draft));
        assert!(!WorkOrderStatus::Closed.can_transition_to(WorkOrderStatus::Reviewed));
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
