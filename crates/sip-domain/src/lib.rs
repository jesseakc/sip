pub mod entity;
pub mod error;
pub mod id;
pub mod repository;
pub mod tenant;

#[cfg(test)]
mod tests {
    use crate::entity::asset::AssetStatus;
    use crate::entity::work_order::WorkOrderStatus;

    #[test]
    fn work_order_status_transitions_are_valid() {
        assert!(WorkOrderStatus::Draft.can_transition_to(WorkOrderStatus::Open));
        assert!(WorkOrderStatus::Open.can_transition_to(WorkOrderStatus::Assigned));
        assert!(WorkOrderStatus::Assigned.can_transition_to(WorkOrderStatus::Accepted));
        assert!(WorkOrderStatus::Accepted.can_transition_to(WorkOrderStatus::InProgress));
        assert!(WorkOrderStatus::InProgress.can_transition_to(WorkOrderStatus::OnHold));
        assert!(WorkOrderStatus::OnHold.can_transition_to(WorkOrderStatus::InProgress));
        assert!(WorkOrderStatus::InProgress.can_transition_to(WorkOrderStatus::Completed));
        assert!(WorkOrderStatus::Completed.can_transition_to(WorkOrderStatus::Reviewed));
        assert!(WorkOrderStatus::Reviewed.can_transition_to(WorkOrderStatus::Closed));
        assert!(WorkOrderStatus::Closed.can_transition_to(WorkOrderStatus::Open));
        assert!(WorkOrderStatus::Closed.can_transition_to(WorkOrderStatus::InProgress));

        assert!(!WorkOrderStatus::Draft.can_transition_to(WorkOrderStatus::Completed));
        assert!(!WorkOrderStatus::Open.can_transition_to(WorkOrderStatus::Closed));
        assert!(!WorkOrderStatus::Cancelled.can_transition_to(WorkOrderStatus::Open));
    }

    #[test]
    fn asset_status_transitions_are_valid() {
        assert!(AssetStatus::Operational.can_transition_to(AssetStatus::Degraded));
        assert!(AssetStatus::Operational.can_transition_to(AssetStatus::Down));
        assert!(AssetStatus::Operational.can_transition_to(AssetStatus::Maintenance));
        assert!(AssetStatus::Degraded.can_transition_to(AssetStatus::Operational));
        assert!(AssetStatus::Degraded.can_transition_to(AssetStatus::Down));
        assert!(AssetStatus::Degraded.can_transition_to(AssetStatus::Maintenance));
        assert!(AssetStatus::Down.can_transition_to(AssetStatus::Operational));
        assert!(AssetStatus::Down.can_transition_to(AssetStatus::Maintenance));
        assert!(AssetStatus::Maintenance.can_transition_to(AssetStatus::Operational));
        assert!(AssetStatus::Maintenance.can_transition_to(AssetStatus::Down));
        assert!(AssetStatus::Maintenance.can_transition_to(AssetStatus::Retired));
        assert!(AssetStatus::Degraded.can_transition_to(AssetStatus::Retired));
        assert!(AssetStatus::Down.can_transition_to(AssetStatus::Retired));

        assert!(!AssetStatus::Operational.can_transition_to(AssetStatus::Retired));
        assert!(!AssetStatus::Retired.can_transition_to(AssetStatus::Operational));
    }

    #[test]
    fn id_display_and_from_uuid_roundtrip() {
        use uuid::Uuid;
        use crate::id::{AssetId, WorkOrderId};

        let uuid = Uuid::new_v4();
        let asset_id: AssetId = uuid.into();
        assert_eq!(asset_id.to_string(), uuid.to_string());
        assert_eq!(Uuid::from(asset_id), uuid);

        let wo_id = WorkOrderId::from(uuid);
        assert_eq!(Uuid::from(wo_id), uuid);
    }

    #[test]
    fn tenant_context_permissions() {
        use crate::id::OrganizationId;
        use crate::tenant::TenantContext;

        let ctx = TenantContext::new(OrganizationId::new(), None)
            .with_permissions(vec!["asset:read".to_string()]);
        assert!(ctx.has_permission("asset:read"));
        assert!(!ctx.has_permission("asset:write"));
    }
}
