use sqlx::{PgPool, Postgres, Transaction};
use sip_domain::id::OrganizationId;
use sip_domain::tenant::TenantContext;

pub async fn set_rls_org(
    executor: &mut sqlx::PgConnection,
    org_id: OrganizationId,
) -> Result<(), sqlx::Error> {
    let org_str = org_id.to_string();
    sqlx::query("SET LOCAL app.current_organization_id = $1")
        .bind(&org_str)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn set_rls_org_pool(pool: &PgPool, org_id: OrganizationId) -> Result<(), sqlx::Error> {
    let org_str = org_id.to_string();
    sqlx::query("SET app.current_organization_id = $1")
        .bind(&org_str)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn begin_tx_with_rls<'a>(
    pool: &'a PgPool,
    ctx: &'a TenantContext,
) -> Result<Transaction<'a, Postgres>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    set_rls_org(tx.as_mut(), ctx.organization_id).await?;
    Ok(tx)
}
