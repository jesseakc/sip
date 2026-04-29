use sip_domain::entity::work_order::WorkOrderStatus;

pub fn is_terminal(status: WorkOrderStatus) -> bool {
    matches!(
        status,
        WorkOrderStatus::Closed | WorkOrderStatus::Cancelled
    )
}
