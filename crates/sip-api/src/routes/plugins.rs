use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

use crate::AppState;

/// GET /api/v1/plugins
/// List all enabled plugins with safe public metadata.
pub async fn list_plugins(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let plugins = registry.public_plugins();
    Json(json!({ "data": plugins }))
}

/// GET /api/v1/plugins/enabled
/// List enabled plugin IDs only.
pub async fn list_enabled_plugins(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let enabled: Vec<String> = registry
        .public_plugins()
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
pub async fn get_ui_navigation(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let nav = registry.aggregate_navigation();
    Json(json!({ "data": nav }))
}

/// GET /api/v1/ui/plugins
/// List UI plugins specifically, with navigation.
pub async fn list_ui_plugins(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let ui_plugins: Vec<serde_json::Value> = registry
        .public_plugins()
        .into_iter()
        .filter(|p| p.ui.is_some())
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "version": p.version,
                "description": p.description,
                "ui": p.ui,
                "navigation": p.navigation,
                "enabled": p.enabled,
            })
        })
        .collect();
    Json(json!({ "data": ui_plugins }))
}

/// GET /api/v1/ui/capabilities
/// UI-level capabilities from enabled plugins (resources, extension points).
pub async fn get_ui_capabilities(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let resources: Vec<serde_json::Value> = registry
        .all_resources()
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect();
    let extension_points: Vec<serde_json::Value> = registry
        .all_extension_points()
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
pub async fn get_extension_points(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let registry = state.plugin_registry.read().await;
    let known = sip_plugins::extension_points::known_extension_points();
    let known_list: Vec<serde_json::Value> = known
        .iter()
        .map(|ep| {
            json!({
                "id": ep.id,
                "description": ep.description,
                "type": ep.plugin_type,
            })
        })
        .collect();
    let declared: Vec<serde_json::Value> = registry
        .all_extension_points()
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

/// GET /api/v1/ui/theme
/// Returns theme metadata for the SIP Core UI.
pub async fn get_ui_theme(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({
        "data": {
            "theme": "sip-core",
            "mode": "system",
            "density": "comfortable",
            "supportsDarkMode": true,
            "supportsAccentColor": true,
            "tokensVersion": "1.0.0",
            "tokens": {
                "colors": {
                    "background": "#f9fafb",
                    "surface": "#ffffff",
                    "surfaceMuted": "#f3f4f6",
                    "border": "#e5e7eb",
                    "text": "#111827",
                    "textMuted": "#6b7280",
                    "primary": "#2563eb",
                    "primaryHover": "#1d4ed8",
                    "danger": "#dc2626",
                    "warning": "#d97706",
                    "success": "#16a34a"
                },
                "radius": {
                    "sm": "0.25rem",
                    "md": "0.375rem",
                    "lg": "0.5rem",
                    "xl": "0.75rem"
                },
                "spacing": {
                    "xs": "0.25rem",
                    "sm": "0.5rem",
                    "md": "1rem",
                    "lg": "1.5rem",
                    "xl": "2rem"
                }
            }
        }
    }))
}

// ─── Plugin application hosting ─────────────────────────────────────────────

/// Content type for a static asset, by extension.
///
/// Deliberately conservative: anything unrecognised is served as
/// `application/octet-stream` rather than guessed, so a plugin cannot get the
/// browser to interpret a file as something the manifest did not declare.
fn content_type_for(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "map" => "application/json; charset=utf-8",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// GET /api/v1/ui/plugins/{plugin_id}/app/{*path}
///
/// Serves a UI plugin's built bundle from `<SIP_PLUGIN_DIR>/<plugin_id>/dist/`.
///
/// This is what makes a plugin's interface reachable without the plugin having
/// to ship a container, and without its source having to be compiled into the
/// first-party frontend. A plugin declares where its app lives on a route via
/// `[[ui.routes]] mount = "..."`, and the SIP shell mounts it in an iframe.
///
/// The handler is on the unauthenticated router because it serves static build
/// output only; the plugin obtains the user's token through the shell's
/// postMessage handshake, never from this response.
pub async fn plugin_app_asset(Path((plugin_id, path)): Path<(String, String)>) -> Response {
    // Manifest ids are restricted to [a-z0-9_-]; re-check rather than trusting
    // the caller, because this value becomes a filesystem path segment.
    if !is_safe_plugin_id(&plugin_id) {
        return (StatusCode::BAD_REQUEST, "invalid plugin id").into_response();
    }

    // Reject traversal outright. Percent-encoded dots are already decoded by the
    // router before reaching here.
    if !is_safe_asset_path(&path) {
        return (StatusCode::FORBIDDEN, "invalid asset path").into_response();
    }

    let plugin_dir = std::env::var("SIP_PLUGIN_DIR").unwrap_or_else(|_| "plugins".to_string());
    let base = PathBuf::from(&plugin_dir).join(&plugin_id).join("dist");
    let mut target = base.join(&path);
    if !target.starts_with(&base) {
        return (StatusCode::FORBIDDEN, "asset path escapes plugin directory").into_response();
    }

    let meta = tokio::fs::metadata(&target).await;
    if matches!(meta, Ok(ref m) if m.is_dir()) {
        target = target.join("index.html");
    }

    match tokio::fs::read(&target).await {
        Ok(bytes) => {
            let content_type = content_type_for(&target);
            (
                [
                    (header::CONTENT_TYPE, content_type),
                    (header::CACHE_CONTROL, "no-cache"),
                ],
                bytes,
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            format!(
                "no built plugin app at {}/dist/{}. Run the plugin build first.",
                plugin_id, path
            ),
        )
            .into_response(),
    }
}

/// True when `plugin_id` is a single, safe path segment.
///
/// Duplicates the manifest's id rule (`[a-z0-9_-]`) on purpose: this value is
/// turned into a filesystem path, so it is validated at the point of use rather
/// than trusted from the registry.
pub(crate) fn is_safe_plugin_id(plugin_id: &str) -> bool {
    !plugin_id.is_empty()
        && plugin_id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// True when `path` is a safe relative asset path: no leading slash, no empty
/// segments, no `.` or `..`, and no backslashes.
pub(crate) fn is_safe_asset_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && path
            .split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != ".." && !seg.contains('\\'))
}

#[cfg(test)]
mod tests {
    use super::{is_safe_asset_path, is_safe_plugin_id};

    #[test]
    fn plugin_ids_must_be_a_single_safe_segment() {
        for ok in ["sip-test-bench", "sip_test_bench", "abc123", "a"] {
            assert!(is_safe_plugin_id(ok), "{ok} should be accepted");
        }
        for bad in [
            "",
            "SIP-Test-Bench", // uppercase
            "sip/test-bench", // path separator
            "../etc",
            "..",
            "sip test", // space
            "sip.test",
            "sip:bench",
        ] {
            assert!(!is_safe_plugin_id(bad), "{bad} should be rejected");
        }
    }

    #[test]
    fn asset_paths_cannot_escape_the_plugin_directory() {
        for ok in [
            "index.html",
            "assets/main-abc123.js",
            "assets/deep/nested/file.css",
            "a.b.c",
        ] {
            assert!(is_safe_asset_path(ok), "{ok} should be accepted");
        }
        for bad in [
            "",
            "/etc/passwd",
            "../secret",
            "assets/../../secret",
            "..",
            ".",
            "assets//main.js",
            "assets/./main.js",
            r"assets\\..\\secret",
            "a\\b",
        ] {
            assert!(!is_safe_asset_path(bad), "{bad:?} should be rejected");
        }
    }
}
