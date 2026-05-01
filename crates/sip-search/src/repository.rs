use async_trait::async_trait;
use sip_domain::error::SipError;
use sip_domain::tenant::TenantContext;

#[async_trait]
pub trait SearchRepository: Send + Sync {
    async fn search(&self, ctx: &TenantContext, query: String) -> Result<Vec<String>, SipError>;
}
