use async_trait::async_trait;
use sip_domain::error::SipError;
use sip_domain::tenant::TenantContext;

#[async_trait]
pub trait ActivityRepository: Send + Sync {
    async fn log(&self, ctx: &TenantContext, event: String) -> Result<(), SipError>;
}
