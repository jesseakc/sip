use thiserror::Error;

#[derive(Error, Debug)]
pub enum TenancyError {
    #[error("missing tenant context")]
    MissingTenantContext,
    #[error("tenant scope violation")]
    TenantScopeViolation,
}
