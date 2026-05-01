use async_trait::async_trait;
use sip_domain::entity::work_order::WorkOrder;
use sip_domain::error::SipError;
use sip_domain::id::WorkOrderId;
use sip_domain::tenant::TenantContext;

#[async_trait]
pub trait WorkOrderRepository: Send + Sync {
    async fn create(&self, ctx: &TenantContext, wo: WorkOrder) -> Result<WorkOrder, SipError>;

    async fn get(
        &self,
        ctx: &TenantContext,
        id: WorkOrderId,
    ) -> Result<Option<WorkOrder>, SipError>;
}
