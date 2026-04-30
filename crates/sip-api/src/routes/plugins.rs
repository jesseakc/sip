use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::AppState;

/// GET /api/v1/plugins
/// List all enabled plugins with safe public metadata.
pub async fn list_plugins(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let plugins = registry.public_plugins();
    Json(json!({ "data": plugins }))
}

/// GET /api/v1/plugins/enabled
/// List enabled plugin IDs only.
pub async fn list_enabled_plugins(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let enabled: Vec<String> = registry.public_plugins()
        .into_iter()
        .filter(|p| p.enabled)
        .map(|p| p.id)
        .collect();
    Json(json!({ "data": enabled }))
}

/// GET /api/v1/plugins/:plugin_id
/// Get public metadata for a single plugin.
pub async fn get_plugin(
    State(state): State<Arc<AppState>>,
    Path(plugin_id): Path<String>,
) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    match registry.public_plugin(&plugin_id) {
        Some(info) => Json(json!({ "data": info })),
        None => Json(json!({ "data": null, "error": "Plugin not found" })),
    }
}

/// GET /api/v1/ui/navigation
/// Aggregated sidebar navigation from all enabled UI plugins.
pub async fn get_ui_navigation(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let nav = registry.aggregate_navigation();
    Json(json!({ "data": nav }))
}

/// GET /api/v1/ui/plugins
/// List UI plugins specifically, with navigation.
pub async fn list_ui_plugins(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let ui_plugins: Vec<serde_json::Value> = registry.public_plugins()
        .into_iter()
        .filter(|p| p.ui.is_some())
        .map(|p| json!({
            "id": p.id,
            "name": p.name,
            "version": p.version,
            "description": p.description,
            "ui": p.ui,
            "navigation": p.navigation,
            "enabled": p.enabled,
        }))
        .collect();
    Json(json!({ "data": ui_plugins }))
}

/// GET /api/v1/ui/capabilities
/// UI-level capabilities from enabled plugins (resources, extension points).
pub async fn get_ui_capabilities(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let resources: Vec<serde_json::Value> = registry.all_resources()
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect();
    let extension_points: Vec<serde_json::Value> = registry.all_extension_points()
        .into_iter()
        .map(|ep| serde_json::to_value(ep).unwrap_or_default())
        .collect();
    Json(json!({
        "data": {
            "resources": resources,
            "extension_points": extension_points,
        }
    }))
}

/// GET /api/v1/ui/extension-points
/// List all available UI extension points (known + plugin-declared).
pub async fn get_extension_points(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let known = sip_plugins::extension_points::known_extension_points();
    let known_list: Vec<serde_json::Value> = known.iter().map(|ep| json!({
        "id": ep.id,
        "description": ep.description,
        "type": ep.plugin_type,
    })).collect();
    let declared: Vec<serde_json::Value> = registry.all_extension_points()
        .into_iter()
        .map(|ep| serde_json::to_value(ep).unwrap_or_default())
        .collect();
    Json(json!({
        "data": {
            "known": known_list,
            "declared": declared,
        }
    }))
}
