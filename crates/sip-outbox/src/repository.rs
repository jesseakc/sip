use async_trait::async_trait;
use sip_domain::error::SipError;
use sip_domain::tenant::TenantContext;

#[async_trait]
pub trait OutboxRepository: Send + Sync {
    async fn enqueue(
        &self,
        ctx: &TenantContext,
        event_type: String,
        payload: serde_json::Value,
    ) -> Result<(), SipError>;
}
