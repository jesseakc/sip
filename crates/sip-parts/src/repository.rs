use async_trait::async_trait;
use sip_domain::error::SipError;
use sip_domain::id::AssetId;
use sip_domain::tenant::TenantContext;

#[async_trait]
pub trait PartRepository: Send + Sync {
    async fn list_by_asset(
        &self,
        ctx: &TenantContext,
        asset_id: AssetId,
    ) -> Result<Vec<String>, SipError>;
}
