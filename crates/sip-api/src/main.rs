pub mod middleware;
pub mod response;
pub mod routes;

use axum::{
    http::{header, Method},
    routing::{delete, get, patch, post},
    Router,
};
use sip_config::{apply_backward_compat, config_status, load_config, validate_config, AppConfig};
use sip_infrastructure::db::create_pool;
use sip_observability::init_tracing;
use sip_plugins::PluginRegistry;
use sip_scheduler::CronEngine;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub plugin_registry: RwLock<PluginRegistry>,
}

async fn capabilities_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    let cfg = &state.config;
    let ai_available = cfg.ai.enabled && cfg.features.ai_chat_enabled && cfg!(feature = "ai");

    let model_name = match cfg.ai.provider {
        sip_config::LlmProviderType::Ollama => Some(cfg.ai.ollama.model.clone()),
        sip_config::LlmProviderType::OpenAI => cfg.ai.openai.as_ref().map(|o| o.model.clone()),
        sip_config::LlmProviderType::Anthropic => {
            cfg.ai.anthropic.as_ref().map(|a| a.model.clone())
        }
        sip_config::LlmProviderType::OpenRouter => {
            cfg.ai.openrouter.as_ref().map(|o| o.model.clone())
        }
        sip_config::LlmProviderType::AzureOpenAI => {
            cfg.ai.azure_openai.as_ref().map(|a| a.deployment.clone())
        }
        sip_config::LlmProviderType::Bedrock => cfg.ai.bedrock.as_ref().map(|b| b.model_id.clone()),
        sip_config::LlmProviderType::CustomHttp => {
            cfg.ai.custom_http.as_ref().map(|c| c.model.clone())
        }
        sip_config::LlmProviderType::Disabled => None,
    };

    axum::Json(serde_json::json!({
        "data": {
            "version": "0.1.0",
            "env": cfg.env.to_string(),
            "features": {
                "assets": true,
                "work_orders": true,
                "ai_chat": ai_available,
                "ai_enabled": cfg.ai.enabled,
                "documents": cfg.features.document_ingestion_enabled && cfg!(feature = "documents"),
                "export": cfg!(feature = "export"),
                "plugins": cfg.features.plugin_system_enabled && cfg!(feature = "plugins"),
                "rag": cfg.features.rag_enabled,
                "semantic_search": cfg.features.semantic_search_enabled,
                "graph": cfg.features.graph_enabled,
                "dispatch": cfg.features.dispatch_enabled,
                "parts_inventory": cfg.features.parts_inventory_enabled,
            },
            "ai": {
                "enabled": cfg.ai.enabled,
                "provider": cfg.ai.provider.to_string(),
                "model": model_name,
                "models": if ai_available { vec![model_name.unwrap_or_else(|| "unknown".into())] } else { vec![] as Vec<String> }
            },
            "plugins": {
                "enabled": cfg.features.plugin_system_enabled && cfg!(feature = "plugins")
            }
        }
    }))
}

async fn config_status_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    let status = config_status(&state.config);
    axum::Json(serde_json::json!({
        "data": status
    }))
}

/// Register a default set of navigation items for the first-party UI.
/// Used when the TOML manifest is unavailable (plugin system disabled or file missing).
fn register_default_navigation(registry: &mut PluginRegistry) {
    use sip_plugins::manifest::*;

    let manifest = PluginManifest {
        id: "sip-core-ui".into(),
        name: "SIP Core UI".into(),
        version: "0.1.0".into(),
        description: Some("Official first-party web interface (embedded default).".into()),
        author: Some("SIP Project".into()),
        license: Some("AGPL-3.0-or-later".into()),
        sip_version: "0.1.0".into(),
        plugin_type: PluginType::Ui,
        enabled_by_default: true,
        ui: Some(UiConfig {
            kind: UiKind::Web,
            framework: Some(UiFramework::Nextjs),
            entrypoint: Some("/".into()),
            dev_url: Some("http://localhost:3000".into()),
            production_mount: Some("/".into()),
            api_base_env: Some("NEXT_PUBLIC_API_URL".into()),
            enabled: true,
            ..Default::default()
        }),
        navigation: vec![
            nav_item(
                "dashboard",
                "Dashboard",
                "/",
                "layout-dashboard",
                "dashboard:read",
                10,
            ),
            nav_item("assets", "Assets", "/assets", "packages", "asset:read", 20),
            nav_item(
                "work-orders",
                "Work Orders",
                "/work-orders",
                "clipboard-list",
                "work_order:read",
                30,
            ),
            nav_item(
                "schedules",
                "Schedules",
                "/schedules",
                "calendar",
                "schedule:read",
                40,
            ),
            nav_item(
                "inspections",
                "Inspections",
                "/inspections",
                "clipboard-check",
                "inspection:read",
                50,
            ),
            nav_item("parts", "Parts", "/parts", "wrench", "part:read", 60),
            nav_item(
                "ai-chat",
                "AI Chat",
                "/ai-chat",
                "message-square",
                "ai:query",
                70,
            ),
            nav_item(
                "locations",
                "Locations",
                "/locations",
                "map-pin",
                "location:read",
                80,
            ),
            nav_item("teams", "Teams", "/teams", "users", "team:read", 90),
            nav_item("users", "Users", "/users", "user", "user:read", 100),
            nav_item(
                "documents",
                "Documents",
                "/documents",
                "file-text",
                "document:read",
                110,
            ),
            nav_item(
                "settings",
                "Settings",
                "/settings",
                "settings",
                "org:manage",
                120,
            ),
        ],
        ..Default::default()
    };

    registry.register(manifest);
}

fn nav_item(
    id: &str,
    label: &str,
    path: &str,
    icon: &str,
    permission: &str,
    order: i32,
) -> sip_plugins::manifest::NavigationItem {
    sip_plugins::manifest::NavigationItem {
        id: id.into(),
        label: label.into(),
        path: path.into(),
        icon: Some(icon.into()),
        permission: Some(permission.into()),
        order: Some(order),
        feature_flag: None,
        children: vec![],
    }
}

#[cfg(not(feature = "plugins"))]
async fn ui_navigation_handler_no_plugins(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let nav = registry.aggregate_navigation();
    axum::Json(serde_json::json!({ "data": nav }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = init_tracing();

    let mut config = load_config().map_err(|e| anyhow::anyhow!("Config load error: {}", e))?;

    // Apply backward compatibility for old flat env vars
    apply_backward_compat(&mut config);

    // Validate config
    let validation = validate_config(&config);
    if !validation.errors.is_empty() {
        for e in &validation.errors {
            tracing::error!("Config validation error: {}", e);
        }
        if config.env.is_strict() {
            anyhow::bail!(
                "Configuration errors detected in {} mode:\n{}",
                config.env,
                validation
            );
        }
    }
    for w in &validation.warnings {
        tracing::warn!("Config warning: {}", w);
    }

    tracing::info!(
        "Starting SIP in {} mode. AI: {} (provider: {}, model: {})",
        config.env,
        if config.ai.enabled {
            "enabled"
        } else {
            "disabled"
        },
        config.ai.provider,
        if config.ai.provider == sip_config::LlmProviderType::Ollama {
            config.ai.ollama.model.as_str()
        } else if config.ai.provider == sip_config::LlmProviderType::OpenAI {
            config
                .ai
                .openai
                .as_ref()
                .map(|o| o.model.as_str())
                .unwrap_or("unknown")
        } else {
            "unknown"
        }
    );

    let db_url = config
        .database
        .as_ref()
        .map(|d| d.url.clone())
        .unwrap_or_else(|| std::env::var("SIP_DATABASE_URL").unwrap_or_default());
    let pool = create_pool(&db_url).await?;

    // Initialize plugin registry
    let plugin_dir = std::env::var("SIP_PLUGIN_DIR").unwrap_or_else(|_| "plugins".to_string());
    let mut plugin_registry = PluginRegistry::new(&plugin_dir);

    // Load built-in sip-core-ui manifest if the plugin system is enabled
    if config.features.plugin_system_enabled {
        // Try loading from plugins directory
        if let Err(e) = plugin_registry.load_all().await {
            tracing::warn!("Failed to load plugins from {}: {:?}", plugin_dir, e);
        }

        // Always register the first-party sip-core-ui plugin
        // (load manifest embedded or from file)
        match sip_plugins::registry::load_manifest_from_file(std::path::Path::new(
            "plugins/sip-core-ui/plugin.toml",
        )) {
            Ok(manifest) => {
                tracing::info!("Registered plugin: {} v{}", manifest.id, manifest.version);
                plugin_registry.register(manifest);
            }
            Err(e) => {
                // Fall back to embedded default navigation if TOML file is missing
                tracing::warn!(
                    "Could not load plugins/sip-core-ui/plugin.toml: {}. Using embedded defaults.",
                    e
                );
                register_default_navigation(&mut plugin_registry);
            }
        }
    } else {
        // Plugin system disabled — still register default navigation for the first-party UI
        register_default_navigation(&mut plugin_registry);
    }

    let state = Arc::new(AppState {
        pool: pool.clone(),
        config: config.clone(),
        plugin_registry: RwLock::new(plugin_registry),
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
                    "config_status": "/admin/config/status"
                },
                "links": {
                    "openapi": "/docs/openapi.yaml",
                }
            }))
        }));

    // Plugin discovery endpoints (public, no auth needed)
    #[cfg(feature = "plugins")]
    let public = public
        .route("/api/v1/plugins", get(routes::plugins::list_plugins))
        .route(
            "/api/v1/plugins/enabled",
            get(routes::plugins::list_enabled_plugins),
        )
        .route(
            "/api/v1/plugins/{plugin_id}",
            get(routes::plugins::get_plugin),
        )
        .route("/api/v1/ui/plugins", get(routes::plugins::list_ui_plugins))
        .route(
            "/api/v1/ui/capabilities",
            get(routes::plugins::get_ui_capabilities),
        )
        .route(
            "/api/v1/ui/extension-points",
            get(routes::plugins::get_extension_points),
        )
        .route(
            "/api/v1/ui/navigation",
            get(routes::plugins::get_ui_navigation),
        )
        .route("/api/v1/ui/theme", get(routes::plugins::get_ui_theme));

    // Navigation endpoint always available (without plugins feature, falls back to embedded defaults)
    #[cfg(not(feature = "plugins"))]
    let public = public.route(
        "/api/v1/ui/navigation",
        get(ui_navigation_handler_no_plugins),
    );

    let mut protected = Router::new()
        .route("/api/v1/auth/logout", post(routes::auth::logout))
        .route("/api/v1/auth/me", get(routes::auth::me))
        .route(
            "/api/v1/organizations",
            post(routes::organizations::create_organization),
        )
        .route(
            "/api/v1/organizations/me",
            get(routes::organizations::get_organization)
                .patch(routes::organizations::update_organization),
        )
        .route(
            "/api/v1/locations",
            get(routes::locations::list_locations).post(routes::locations::create_location),
        )
        .route(
            "/api/v1/locations/{id}",
            get(routes::locations::get_location).patch(routes::locations::update_location),
        )
        .route(
            "/api/v1/locations/{id}/children",
            get(routes::locations::list_location_children),
        )
        .route(
            "/api/v1/locations/{id}/assets",
            get(routes::locations::list_location_assets),
        )
        .route(
            "/api/v1/asset-types",
            get(routes::asset_types::list_asset_types).post(routes::asset_types::create_asset_type),
        )
        .route(
            "/api/v1/asset-types/{id}",
            get(routes::asset_types::get_asset_type).patch(routes::asset_types::update_asset_type),
        )
        .route(
            "/api/v1/manufacturers",
            get(routes::manufacturers::list_manufacturers)
                .post(routes::manufacturers::create_manufacturer),
        )
        .route(
            "/api/v1/manufacturers/{id}",
            get(routes::manufacturers::get_manufacturer),
        )
        .route(
            "/api/v1/manufacturers/{id}/models",
            get(routes::manufacturers::list_manufacturer_models),
        )
        .route(
            "/api/v1/models",
            get(routes::models::list_models).post(routes::models::create_model),
        )
        .route("/api/v1/models/{id}", get(routes::models::get_model))
        .route(
            "/api/v1/users",
            get(routes::users::list_users).post(routes::users::create_user),
        )
        .route(
            "/api/v1/users/{id}",
            get(routes::users::get_user).patch(routes::users::update_user),
        )
        .route(
            "/api/v1/teams",
            get(routes::teams::list_teams).post(routes::teams::create_team),
        )
        .route(
            "/api/v1/teams/{id}",
            get(routes::teams::get_team).patch(routes::teams::update_team),
        )
        .route(
            "/api/v1/assets",
            get(routes::assets::list_assets).post(routes::assets::create_asset),
        )
        .route(
            "/api/v1/assets/{id}",
            get(routes::assets::get_asset).patch(routes::assets::update_asset_status),
        )
        .route(
            "/api/v1/assets/{id}/archive",
            post(routes::assets::archive_asset),
        )
        .route(
            "/api/v1/assets/{id}/children",
            get(routes::assets::list_asset_children),
        )
        .route(
            "/api/v1/assets/{id}/work-orders",
            get(routes::assets::list_asset_work_orders),
        )
        .route(
            "/api/v1/work-orders",
            get(routes::work_orders::list_work_orders).post(routes::work_orders::create_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}",
            get(routes::work_orders::get_work_order)
                .patch(routes::work_orders::transition_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/publish",
            post(routes::work_orders::publish_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/start",
            post(routes::work_orders::start_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/hold",
            post(routes::work_orders::hold_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/resume",
            post(routes::work_orders::resume_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/complete",
            post(routes::work_orders::complete_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/review",
            post(routes::work_orders::review_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/close",
            post(routes::work_orders::close_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/cancel",
            post(routes::work_orders::cancel_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/archive",
            post(routes::work_orders::archive_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/reopen",
            post(routes::work_orders::reopen_work_order),
        )
        .route(
            "/api/v1/work-orders/{id}/assignments",
            get(routes::work_orders::list_assignments).post(routes::work_orders::create_assignment),
        )
        .route(
            "/api/v1/work-orders/{id}/assignments/{assignment_id}",
            delete(routes::work_orders::delete_assignment)
                .patch(routes::work_orders::update_assignment),
        )
        .route(
            "/api/v1/work-orders/{id}/parts",
            get(routes::work_orders::list_parts).post(routes::work_orders::create_part_usage),
        )
        .route(
            "/api/v1/schedules",
            get(routes::schedules::list_schedules).post(routes::schedules::create_schedule),
        )
        .route(
            "/api/v1/schedules/{id}",
            get(routes::schedules::get_schedule).patch(routes::schedules::update_schedule),
        )
        .route(
            "/api/v1/schedules/{id}/archive",
            post(routes::schedules::archive_schedule),
        )
        .route(
            "/api/v1/inspections",
            get(routes::inspections::list_inspections),
        )
        .route(
            "/api/v1/inspections/{id}",
            get(routes::inspections::get_inspection),
        )
        .route(
            "/api/v1/inspections/{id}/items",
            patch(routes::inspections::update_checklist_items),
        )
        .route(
            "/api/v1/parts",
            get(routes::parts::list_parts).post(routes::parts::create_part),
        )
        .route(
            "/api/v1/parts/{id}",
            get(routes::parts::get_part).patch(routes::parts::update_part),
        );

    #[cfg(feature = "documents")]
    {
        protected = protected
            .route(
                "/api/v1/documents",
                get(routes::documents::list_documents).post(routes::documents::create_document),
            )
            .route(
                "/api/v1/documents/{id}",
                get(routes::documents::get_document),
            )
            .route(
                "/api/v1/documents/{id}/archive",
                post(routes::documents::archive_document),
            );
    }

    protected = protected
        .route(
            "/api/v1/activities",
            get(routes::activities::list_activities),
        )
        .route(
            "/api/v1/activities/{entity_type}/{entity_id}",
            get(routes::activities::list_activities_by_entity),
        );

    #[cfg(feature = "export")]
    {
        protected = protected
            .route(
                "/api/v1/export/work-orders",
                post(routes::export::export_work_orders),
            )
            .route("/api/v1/export", get(routes::export::export_all));
    }

    #[cfg(feature = "plugins")]
    {
        protected = protected
            .route(
                "/api/v1/migrations",
                get(routes::migrations::list_jobs).post(routes::migrations::create_job),
            )
            .route(
                "/api/v1/migrations/{job_id}",
                get(routes::migrations::get_job),
            )
            .route(
                "/api/v1/migrations/{job_id}/source-records",
                get(routes::migrations::get_source_records)
                    .post(routes::migrations::add_source_records),
            )
            .route(
                "/api/v1/migrations/{job_id}/staged-records",
                get(routes::migrations::get_staged_records),
            )
            .route(
                "/api/v1/migrations/{job_id}/mappings",
                get(routes::migrations::get_field_mappings)
                    .post(routes::migrations::save_field_mappings),
            )
            .route(
                "/api/v1/migrations/{job_id}/validate",
                post(routes::migrations::validate_job),
            )
            .route(
                "/api/v1/migrations/{job_id}/dry-run",
                post(routes::migrations::dry_run),
            )
            .route(
                "/api/v1/migrations/{job_id}/import",
                post(routes::migrations::execute_import),
            )
            .route(
                "/api/v1/migrations/{job_id}/rollback",
                post(routes::migrations::rollback_job),
            )
            .route(
                "/api/v1/migrations/{job_id}/cancel",
                post(routes::migrations::cancel_job),
            )
            .route(
                "/api/v1/migrations/{job_id}/validation-issues",
                get(routes::migrations::get_validation_issues),
            )
            .route(
                "/api/v1/migrations/{job_id}/duplicates",
                get(routes::migrations::get_duplicates),
            )
            .route(
                "/api/v1/migrations/{job_id}/external-id-maps",
                get(routes::migrations::get_external_id_maps),
            )
            .route(
                "/api/v1/migrations/{job_id}/report",
                get(routes::migrations::get_report),
            );
    }

    #[cfg(feature = "ai")]
    {
        // Only register AI routes if feature flag for AI chat is enabled in config
        if state.config.features.ai_chat_enabled {
            protected = protected
                .route("/api/v1/ai/chat", post(routes::ai::chat))
                .route(
                    "/api/v1/ai/conversations",
                    get(routes::ai_conversations::list_conversations),
                )
                .route(
                    "/api/v1/ai/conversations/{id}",
                    get(routes::ai_conversations::get_conversation),
                )
                .route(
                    "/api/v1/ai/messages/{id}/retrieval-trace",
                    get(routes::ai_conversations::get_retrieval_trace),
                )
                .route(
                    "/api/v1/ai/messages/{id}/verification-trace",
                    get(routes::ai_conversations::get_verification_trace),
                )
                .route(
                    "/api/v1/ai/messages/{id}/feedback",
                    post(routes::ai_conversations::submit_feedback),
                );
        }
    }

    // Config inspection endpoint — only in non-production environments
    if !state.config.env.is_production() {
        protected = protected.route("/admin/config/status", get(config_status_handler));
    }

    protected = protected.layer(axum::middleware::from_fn_with_state(
        state.clone(),
        middleware::auth::auth_middleware,
    ));

    let cors = if config.security.cors_allowed_origins.is_empty() {
        if config.env.is_production() {
            tracing::warn!("CORS origins empty in production — using permissive CORS. Set SIP_CORS_ALLOWED_ORIGINS.");
        }
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(
                config
                    .security
                    .cors_allowed_origins
                    .iter()
                    .map(|o| o.parse().unwrap())
                    .collect::<Vec<_>>(),
            )
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
            .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
    };

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .layer(cors)
        .with_state(state);

    if config.features.dispatch_enabled {
        let cron_engine = CronEngine::new(pool.clone());
        let _cron_handle = cron_engine.start();
        tracing::info!("Cron engine started");
    } else {
        tracing::info!("Cron engine disabled (dispatch feature off)");
    }

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("SIP API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
