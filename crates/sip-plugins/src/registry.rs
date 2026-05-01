use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::manifest::{PluginManifest, PublicPluginInfo};
use super::validation::{validate_manifest, validate_plugin_set, ValidationResult};

/// The plugin registry holds all registered plugins and provides
/// methods to query them for navigation, capabilities, extension points, etc.
pub struct PluginRegistry {
    /// All loaded plugin manifests, indexed by plugin ID.
    manifests: HashMap<String, PluginManifest>,
    /// Which plugins are currently enabled.
    enabled: HashMap<String, bool>,
    /// The directory plugins are loaded from.
    plugin_dir: PathBuf,
}

impl PluginRegistry {
    pub fn new(plugin_dir: impl Into<PathBuf>) -> Self {
        Self {
            manifests: HashMap::new(),
            enabled: HashMap::new(),
            plugin_dir: plugin_dir.into(),
        }
    }

    /// Create an empty registry (useful for tests or when plugins are disabled).
    pub fn empty() -> Self {
        Self {
            manifests: HashMap::new(),
            enabled: HashMap::new(),
            plugin_dir: PathBuf::from("plugins"),
        }
    }

    // ── Loading ─────────────────────────────────────────────────────────

    /// Scan the plugin directory for `plugin.toml` files and load them.
    /// Each plugin lives in its own subdirectory: `plugins/{plugin-id}/plugin.toml`
    pub async fn load_all(&mut self) -> Result<(Vec<String>, Vec<String>), String> {
        let mut loaded = Vec::new();
        let mut failed = Vec::new();

        if !self.plugin_dir.exists() {
            return Ok((loaded, failed));
        }

        let mut entries = tokio::fs::read_dir(&self.plugin_dir)
            .await
            .map_err(|e| format!("Failed to read plugin dir {:?}: {}", self.plugin_dir, e))?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.toml");
            if !manifest_path.exists() {
                continue;
            }

            match load_manifest_from_file(&manifest_path) {
                Ok(manifest) => {
                    let id = manifest.id.clone();
                    let enabled = manifest.enabled_by_default;
                    self.manifests.insert(id.clone(), manifest);
                    self.enabled.insert(id.clone(), enabled);
                    loaded.push(id);
                }
                Err(e) => {
                    let dir_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
                    tracing::warn!("Failed to load plugin '{}': {}", dir_name, e);
                    failed.push(dir_name.to_string());
                }
            }
        }

        Ok((loaded, failed))
    }

    /// Register a manifest directly (used for the built-in first-party plugin
    /// and for tests).
    pub fn register(&mut self, manifest: PluginManifest) {
        let enabled = manifest.enabled_by_default;
        self.enabled.insert(manifest.id.clone(), enabled);
        self.manifests.insert(manifest.id.clone(), manifest);
    }

    // ── Enable / Disable ────────────────────────────────────────────────

    /// Enable a plugin by ID.
    pub fn enable(&mut self, plugin_id: &str) -> bool {
        if self.manifests.contains_key(plugin_id) {
            self.enabled.insert(plugin_id.to_string(), true);
            true
        } else {
            false
        }
    }

    /// Disable a plugin by ID.
    pub fn disable(&mut self, plugin_id: &str) {
        self.enabled.insert(plugin_id.to_string(), false);
    }

    /// Set which plugins are enabled from a list of IDs.
    /// Any plugin not in the list is disabled.
    pub fn set_enabled_list(&mut self, enabled_ids: &[String]) {
        for id in self.manifests.keys() {
            self.enabled.insert(id.clone(), enabled_ids.contains(id));
        }
    }

    // ── Querying ────────────────────────────────────────────────────────

    /// Returns true if a plugin is registered and enabled.
    pub fn is_enabled(&self, plugin_id: &str) -> bool {
        self.enabled.get(plugin_id).copied().unwrap_or(false)
    }

    /// Get all registered plugin IDs.
    pub fn plugin_ids(&self) -> Vec<&str> {
        self.manifests.keys().map(|k| k.as_str()).collect()
    }

    /// Get a manifest by ID.
    pub fn get(&self, plugin_id: &str) -> Option<&PluginManifest> {
        self.manifests.get(plugin_id)
    }

    /// Get all enabled UI plugins (type Ui or Hybrid).
    pub fn enabled_ui_plugins(&self) -> Vec<&PluginManifest> {
        self.manifests
            .values()
            .filter(|m| {
                self.is_enabled(&m.id)
                    && (m.plugin_type == super::manifest::PluginType::Ui
                        || m.plugin_type == super::manifest::PluginType::Hybrid)
            })
            .filter(|m| m.ui.is_some())
            .collect()
    }

    /// Get the aggregated navigation from all enabled UI plugins.
    /// Navigation items are ordered by their `order` field, then by plugin registration order.
    pub fn aggregate_navigation(&self) -> Vec<super::manifest::NavigationItem> {
        let mut nav_items: Vec<(i32, String, super::manifest::NavigationItem)> = Vec::new();

        for p in self.enabled_ui_plugins() {
            for item in &p.navigation {
                let order = item.order.unwrap_or(999);
                nav_items.push((order, p.id.clone(), item.clone()));
            }
        }

        // Sort by order, then by plugin ID (stability)
        nav_items.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        nav_items.into_iter().map(|(_, _, item)| item).collect()
    }

    /// Get public info for all enabled plugins (safe for API exposure).
    pub fn public_plugins(&self) -> Vec<PublicPluginInfo> {
        self.manifests
            .iter()
            .filter(|(id, _)| self.is_enabled(id))
            .map(|(_, m)| {
                let mut info = m.public_view();
                info.enabled = self.is_enabled(&m.id);
                info
            })
            .collect()
    }

    /// Get public info for a single plugin.
    pub fn public_plugin(&self, plugin_id: &str) -> Option<PublicPluginInfo> {
        self.manifests.get(plugin_id).map(|m| {
            let mut info = m.public_view();
            info.enabled = self.is_enabled(plugin_id);
            info
        })
    }

    /// Collect all extension points declared by enabled plugins.
    pub fn all_extension_points(&self) -> Vec<super::manifest::ExtensionPointDecl> {
        self.manifests
            .iter()
            .filter(|(id, _)| self.is_enabled(id))
            .flat_map(|(_, m)| m.extension_points.iter().cloned())
            .collect()
    }

    /// Get resource definitions from all enabled plugins.
    pub fn all_resources(&self) -> Vec<super::manifest::ResourceDef> {
        self.manifests
            .iter()
            .filter(|(id, _)| self.is_enabled(id))
            .flat_map(|(_, m)| m.resources.iter().cloned())
            .collect()
    }

    // ── Validation ──────────────────────────────────────────────────────

    /// Validate all loaded manifests.
    pub fn validate_all(&self) -> ValidationResult {
        let manifests: Vec<&PluginManifest> = self.manifests.values().collect();
        let mut result = ValidationResult::ok();

        for m in &manifests {
            result.merge(validate_manifest(m));
        }

        result.merge(validate_plugin_set(
            &manifests.iter().map(|m| (*m).clone()).collect::<Vec<_>>(),
        ));

        result
    }
}

// ─── File loading ───────────────────────────────────────────────────────────

/// Load a plugin manifest from a `plugin.toml` file.
pub fn load_manifest_from_file(path: &Path) -> Result<PluginManifest, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

    let manifest: PluginManifest =
        toml::from_str(&content).map_err(|e| format!("Failed to parse {:?}: {}", path, e))?;

    Ok(manifest)
}

/// Load a plugin manifest from a TOML string (useful for tests and embedded manifests).
pub fn load_manifest_from_str(toml_str: &str) -> Result<PluginManifest, String> {
    toml::from_str(toml_str).map_err(|e| format!("Failed to parse manifest: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_core_ui_manifest() -> PluginManifest {
        crate::manifest::PluginManifest {
            id: "sip-core-ui".into(),
            name: "SIP Core UI".into(),
            version: "0.1.0".into(),
            description: Some("Official first-party web interface for SIP.".into()),
            author: Some("SIP Project".into()),
            license: Some("AGPL-3.0-or-later".into()),
            sip_version: "0.1.0".into(),
            plugin_type: crate::manifest::PluginType::Ui,
            enabled_by_default: true,
            ui: Some(crate::manifest::UiConfig {
                kind: crate::manifest::UiKind::Web,
                framework: Some(crate::manifest::UiFramework::Nextjs),
                entrypoint: Some("/".into()),
                dev_url: Some("http://localhost:3000".into()),
                production_mount: Some("/".into()),
                api_base_env: Some("NEXT_PUBLIC_API_URL".into()),
                enabled: true,
                ..Default::default()
            }),
            navigation: vec![
                crate::manifest::NavigationItem {
                    id: "dashboard".into(),
                    label: "Dashboard".into(),
                    path: "/".into(),
                    icon: Some("layout-dashboard".into()),
                    permission: Some("dashboard:read".into()),
                    order: Some(10),
                    feature_flag: None,
                    children: vec![],
                },
                crate::manifest::NavigationItem {
                    id: "assets".into(),
                    label: "Assets".into(),
                    path: "/assets".into(),
                    icon: Some("boxes".into()),
                    permission: Some("asset:read".into()),
                    order: Some(20),
                    feature_flag: None,
                    children: vec![],
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_register_and_query() {
        let mut registry = PluginRegistry::empty();
        let manifest = make_core_ui_manifest();
        registry.register(manifest);

        assert!(registry.is_enabled("sip-core-ui"));
        assert_eq!(registry.plugin_ids(), vec!["sip-core-ui"]);
        assert_eq!(registry.enabled_ui_plugins().len(), 1);
    }

    #[test]
    fn test_aggregate_navigation() {
        let mut registry = PluginRegistry::empty();
        registry.register(make_core_ui_manifest());

        let nav = registry.aggregate_navigation();
        assert_eq!(nav.len(), 2);
        assert_eq!(nav[0].id, "dashboard");
        assert_eq!(nav[1].id, "assets");
    }

    #[test]
    fn test_disable_plugin() {
        let mut registry = PluginRegistry::empty();
        registry.register(make_core_ui_manifest());
        registry.disable("sip-core-ui");

        assert!(!registry.is_enabled("sip-core-ui"));
        assert_eq!(registry.enabled_ui_plugins().len(), 0);
        assert!(registry.aggregate_navigation().is_empty());
    }

    #[test]
    fn test_public_plugins() {
        let mut registry = PluginRegistry::empty();
        registry.register(make_core_ui_manifest());

        let public = registry.public_plugins();
        assert_eq!(public.len(), 1);
        assert_eq!(public[0].id, "sip-core-ui");
        assert!(public[0].ui.is_some());
    }

    #[test]
    fn test_custom_enabled_list() {
        let mut registry = PluginRegistry::empty();
        registry.register(make_core_ui_manifest());

        registry.set_enabled_list(&vec![]);
        assert!(!registry.is_enabled("sip-core-ui"));

        registry.set_enabled_list(&vec!["sip-core-ui".into()]);
        assert!(registry.is_enabled("sip-core-ui"));
    }

    #[test]
    fn test_load_from_str() {
        let toml_str = r#"
id = "test-plugin"
name = "Test Plugin"
version = "1.0.0"
plugin_type = "ui"

[ui]
kind = "web"
framework = "nextjs"
entrypoint = "/"
"#;
        let manifest = load_manifest_from_str(toml_str).unwrap();
        assert_eq!(manifest.id, "test-plugin");
        assert_eq!(manifest.plugin_type, crate::manifest::PluginType::Ui);
    }

    #[test]
    fn test_load_manifest_with_compatibility_theme_routes_actions() {
        let toml_str = r#"
id = "rich-ui-plugin"
name = "Rich UI Plugin"
version = "2.0.0"
plugin_type = "ui"

[ui]
kind = "web"
framework = "react"
entrypoint = "/rich"
enabled = true
description = "A rich UI plugin"
icon = "star"
category = "analytics"

[ui.compatibility]
level = "native"
sip_ui_version = "0.1.0"
requires_shell = true
uses_sip_components = true
uses_theme_tokens = true
allows_global_css = false

[ui.theme]
supports_dark_mode = true
supports_density = true
supports_accent_color = true
uses_design_tokens = true

[[ui.routes]]
id = "analytics"
path = "/analytics"
component = "AnalyticsPage"
layout = "sip-dashboard"
title = "Analytics"
required_permissions = ["analytics:read"]

[[ui.actions]]
id = "refresh-dashboard"
label = "Refresh"
icon = "refresh-cw"
route = "/refresh"
placement = ["toolbar"]
required_permissions = ["dashboard:refresh"]
"#;
        let manifest = load_manifest_from_str(toml_str).unwrap();
        assert_eq!(manifest.id, "rich-ui-plugin");
        assert_eq!(manifest.plugin_type, crate::manifest::PluginType::Ui);

        let ui = manifest.ui.as_ref().unwrap();
        assert!(ui.enabled);
        assert_eq!(ui.description.as_deref(), Some("A rich UI plugin"));
        assert_eq!(ui.icon.as_deref(), Some("star"));
        assert_eq!(ui.category.as_deref(), Some("analytics"));

        let compat = ui.compatibility.as_ref().unwrap();
        assert_eq!(
            compat.level,
            Some(crate::manifest::UiCompatibilityLevel::Native)
        );
        assert_eq!(compat.sip_ui_version.as_deref(), Some("0.1.0"));
        assert!(compat.requires_shell);
        assert!(compat.uses_sip_components);
        assert!(compat.uses_theme_tokens);
        assert!(!compat.allows_global_css);

        let theme = ui.theme.as_ref().unwrap();
        assert!(theme.supports_dark_mode);
        assert!(theme.supports_density);
        assert!(theme.supports_accent_color);
        assert!(theme.uses_design_tokens);

        assert_eq!(ui.routes.len(), 1);
        assert_eq!(ui.routes[0].id, "analytics");
        assert_eq!(ui.routes[0].path, "/analytics");
        assert_eq!(ui.routes[0].layout.as_deref(), Some("sip-dashboard"));

        assert_eq!(ui.actions.len(), 1);
        assert_eq!(ui.actions[0].id, "refresh-dashboard");
        assert_eq!(ui.actions[0].label, "Refresh");
    }

    #[test]
    fn test_validate_all() {
        let mut registry = PluginRegistry::empty();
        registry.register(make_core_ui_manifest());
        let result = registry.validate_all();
        assert!(result.valid, "Validation errors: {:?}", result.errors);
    }
}
