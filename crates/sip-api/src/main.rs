pub mod middleware;
pub mod response;
pub mod routes;

use axum::{
    routing::{get, post, patch, delete},
    Router,
};
use sip_infrastructure::db::create_pool;
use sip_observability::init_tracing;
use sip_config::load_config;
use sip_scheduler::CronEngine;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use sqlx::PgPool;

pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub refresh_expiration: u64,
    pub ollama_url: String,
    pub ollama_model: String,
}

async fn capabilities_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "data": {
            "features": {
                "assets": true,
                "work_orders": true,
                "ai_chat": cfg!(feature = "ai"),
                "documents": cfg!(feature = "documents"),
                "export": cfg!(feature = "export"),
                "plugins": cfg!(feature = "plugins"),
            },
            "ai": {
                "enabled": cfg!(feature = "ai"),
                "models": if cfg!(feature = "ai") { vec!["llama3.1:8b"] } else { vec![] as Vec<&str> }
            },
            "plugins": {
                "enabled": cfg!(feature = "plugins")
            }
        }
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = init_tracing();

    let config = load_config().map_err(|e| anyhow::anyhow!("Config load error: {}", e))?;
    let pool = create_pool(&config.database_url).await?;

    let state = Arc::new(AppState {
        pool: pool.clone(),
        jwt_secret: config.jwt_secret.clone(),
        jwt_expiration: config.jwt_expiration_seconds,
        refresh_expiration: config.refresh_expiration_seconds,
        ollama_url: config.ollama_url.clone(),
        ollama_model: config.ollama_model.clone(),
    });

    let public = Router::new()
        .route("/api/v1/health", get(routes::health::health))
        .route("/api/v1/health/ready", get(routes::health::ready))
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/refresh", post(routes::auth::refresh))
        .route("/api/v1/capabilities", get(capabilities_handler))
        .route("/.well-known/sip-api", get(|| async {
            axum::Json(serde_json::json!({
                "name": "SIP API",
                "version": "0.1.0",
                "description": "Service Intelligence Platform — machine-readable maintenance intelligence",
                "auth": {
                    "type": "bearer_jwt",
                    "login_url": "/api/v1/auth/login",
                    "refresh_url": "/api/v1/auth/refresh",
                },
                "documentation": "/docs/openapi.yaml",
                "endpoints": {
                    "health": "/api/v1/health",
                    "capabilities": "/api/v1/capabilities",
                },
                "links": {
                    "openapi": "/docs/openapi.yaml",
                }
            }))
        }));

    let mut protected = Router::new()
        .route("/api/v1/auth/logout", post(routes::auth::logout))
        .route("/api/v1/auth/me", get(routes::auth::me))
        .route("/api/v1/organizations", post(routes::organizations::create_organization))
        .route("/api/v1/organizations/me", get(routes::organizations::get_organization).patch(routes::organizations::update_organization))
        .route("/api/v1/locations", get(routes::locations::list_locations).post(routes::locations::create_location))
        .route("/api/v1/locations/:id", get(routes::locations::get_location).patch(routes::locations::update_location))
        .route("/api/v1/locations/:id/children", get(routes::locations::list_location_children))
        .route("/api/v1/locations/:id/assets", get(routes::locations::list_location_assets))
        .route("/api/v1/asset-types", get(routes::asset_types::list_asset_types).post(routes::asset_types::create_asset_type))
        .route("/api/v1/asset-types/:id", get(routes::asset_types::get_asset_type).patch(routes::asset_types::update_asset_type))
        .route("/api/v1/manufacturers", get(routes::manufacturers::list_manufacturers).post(routes::manufacturers::create_manufacturer))
        .route("/api/v1/manufacturers/:id", get(routes::manufacturers::get_manufacturer))
        .route("/api/v1/manufacturers/:id/models", get(routes::manufacturers::list_manufacturer_models))
        .route("/api/v1/models", get(routes::models::list_models).post(routes::models::create_model))
        .route("/api/v1/models/:id", get(routes::models::get_model))
        .route("/api/v1/users", get(routes::users::list_users).post(routes::users::create_user))
        .route("/api/v1/users/:id", get(routes::users::get_user).patch(routes::users::update_user))
        .route("/api/v1/teams", get(routes::teams::list_teams).post(routes::teams::create_team))
        .route("/api/v1/teams/:id", get(routes::teams::get_team).patch(routes::teams::update_team))
        .route("/api/v1/assets", get(routes::assets::list_assets).post(routes::assets::create_asset))
        .route("/api/v1/assets/:id", get(routes::assets::get_asset).patch(routes::assets::update_asset_status))
        .route("/api/v1/assets/:id/archive", post(routes::assets::archive_asset))
        .route("/api/v1/assets/:id/children", get(routes::assets::list_asset_children))
        .route("/api/v1/assets/:id/work-orders", get(routes::assets::list_asset_work_orders))
        .route("/api/v1/work-orders", get(routes::work_orders::list_work_orders).post(routes::work_orders::create_work_order))
        .route("/api/v1/work-orders/:id", get(routes::work_orders::get_work_order).patch(routes::work_orders::transition_work_order))
        .route("/api/v1/work-orders/:id/publish", post(routes::work_orders::publish_work_order))
        .route("/api/v1/work-orders/:id/start", post(routes::work_orders::start_work_order))
        .route("/api/v1/work-orders/:id/hold", post(routes::work_orders::hold_work_order))
        .route("/api/v1/work-orders/:id/resume", post(routes::work_orders::resume_work_order))
        .route("/api/v1/work-orders/:id/complete", post(routes::work_orders::complete_work_order))
        .route("/api/v1/work-orders/:id/review", post(routes::work_orders::review_work_order))
        .route("/api/v1/work-orders/:id/close", post(routes::work_orders::close_work_order))
        .route("/api/v1/work-orders/:id/cancel", post(routes::work_orders::cancel_work_order))
        .route("/api/v1/work-orders/:id/archive", post(routes::work_orders::archive_work_order))
        .route("/api/v1/work-orders/:id/reopen", post(routes::work_orders::reopen_work_order))
        .route("/api/v1/work-orders/:id/assignments", get(routes::work_orders::list_assignments).post(routes::work_orders::create_assignment))
        .route("/api/v1/work-orders/:id/assignments/:assignment_id", delete(routes::work_orders::delete_assignment).patch(routes::work_orders::update_assignment))
        .route("/api/v1/work-orders/:id/parts", get(routes::work_orders::list_parts).post(routes::work_orders::create_part_usage))
        .route("/api/v1/schedules", get(routes::schedules::list_schedules).post(routes::schedules::create_schedule))
        .route("/api/v1/schedules/:id", get(routes::schedules::get_schedule).patch(routes::schedules::update_schedule))
        .route("/api/v1/schedules/:id/archive", post(routes::schedules::archive_schedule))
        .route("/api/v1/inspections", get(routes::inspections::list_inspections))
        .route("/api/v1/inspections/:id", get(routes::inspections::get_inspection))
        .route("/api/v1/inspections/:id/items", patch(routes::inspections::update_checklist_items))
        .route("/api/v1/parts", get(routes::parts::list_parts).post(routes::parts::create_part))
        .route("/api/v1/parts/:id", get(routes::parts::get_part).patch(routes::parts::update_part));

    #[cfg(feature = "documents")]
    {
        protected = protected
            .route("/api/v1/documents", get(routes::documents::list_documents).post(routes::documents::create_document))
            .route("/api/v1/documents/:id", get(routes::documents::get_document))
            .route("/api/v1/documents/:id/archive", post(routes::documents::archive_document));
    }

    protected = protected
        .route("/api/v1/activities", get(routes::activities::list_activities))
        .route("/api/v1/activities/:entity_type/:entity_id", get(routes::activities::list_activities_by_entity));

    #[cfg(feature = "export")]
    {
        protected = protected
            .route("/api/v1/export/work-orders", post(routes::export::export_work_orders))
            .route("/api/v1/export", get(routes::export::export_all));
    }

    #[cfg(feature = "ai")]
    {
        protected = protected
            .route("/api/v1/ai/chat", post(routes::ai::chat))
            .route("/api/v1/ai/conversations", get(routes::ai_conversations::list_conversations))
            .route("/api/v1/ai/conversations/:id", get(routes::ai_conversations::get_conversation))
            .route("/api/v1/ai/messages/:id/retrieval-trace", get(routes::ai_conversations::get_retrieval_trace))
            .route("/api/v1/ai/messages/:id/verification-trace", get(routes::ai_conversations::get_verification_trace))
            .route("/api/v1/ai/messages/:id/feedback", post(routes::ai_conversations::submit_feedback));
    }

    protected = protected
        .layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::auth_middleware));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let cron_engine = CronEngine::new(pool.clone());
    let _cron_handle = cron_engine.start();

    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port).parse()?;
    tracing::info!("SIP API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
