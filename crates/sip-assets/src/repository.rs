use async_trait::async_trait;
use sip_domain::entity::asset::Asset;
use sip_domain::error::SipError;
use sip_domain::id::AssetId;
use sip_domain::tenant::TenantContext;

#[async_trait]
pub trait AssetRepository: Send + Sync {
    async fn create(&self, ctx: &TenantContext, asset: Asset) -> Result<Asset, SipError>;

    async fn get(&self, ctx: &TenantContext, id: AssetId) -> Result<Option<Asset>, SipError>;
}
