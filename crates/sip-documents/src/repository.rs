use async_trait::async_trait;
use sip_domain::error::SipError;
use sip_domain::tenant::TenantContext;

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn create(&self, ctx: &TenantContext, name: String) -> Result<String, SipError>;
}
