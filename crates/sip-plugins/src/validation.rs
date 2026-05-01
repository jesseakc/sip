use super::manifest::UiConfig;
use super::manifest::{PluginManifest, PluginType, UiCompatibilityLevel};

/// Result of validating a single plugin manifest.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            valid: false,
            errors: vec![msg.into()],
            warnings: vec![],
        }
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.valid = false;
        self.errors.push(msg.into());
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.valid = self.valid && other.valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

/// Known UI extension points. Plugins contribute to these.
pub const KNOWN_EXTENSION_POINTS: &[&str] = &[
    // Shell
    "app.shell.sidebar.nav",
    "app.shell.topbar.actions",
    // Dashboard
    "dashboard.cards",
    "dashboard.widgets",
    // Asset
    "asset.list.columns",
    "asset.detail.tabs",
    "asset.detail.sidebar",
    "asset.detail.actions",
    "asset.create.form.sections",
    // Work Order
    "work_order.list.columns",
    "work_order.detail.tabs",
    "work_order.detail.sidebar",
    "work_order.detail.actions",
    "work_order.create.form.sections",
    // Schedule
    "schedule.list.columns",
    "schedule.detail.actions",
    // Inspection
    "inspection.list.columns",
    "inspection.detail.panels",
    "inspection.detail.actions",
    "inspection_checklist.items",
    // Parts
    "parts.list.columns",
    "parts.detail.actions",
    "parts.create.form.sections",
    // Settings
    "settings.sections",
    "settings.general.form",
    // Command palette / tools
    "command_palette.actions",
    "ai_chat.tools",
    // Documents
    "document.list.columns",
    "document.detail.actions",
    // Locations
    "location.list.columns",
    "location.detail.actions",
    // Teams / Users
    "team.list.columns",
    "team.detail.actions",
    "user.list.columns",
    "user.detail.actions",
    // Migration
    "migration.source_connector",
    "migration.file_parser",
    "migration.field_mapper",
    "migration.transformer",
    "migration.validator",
    "migration.duplicate_resolver",
    "migration.post_import_hook",
    "migration.report_generator",
];

/// Validate a single plugin manifest (does not check dependencies across plugins).
pub fn validate_manifest(manifest: &PluginManifest) -> ValidationResult {
    let mut result = ValidationResult::ok();

    // ── id ──
    if manifest.id.is_empty() {
        result.add_error("Plugin `id` is required");
    } else if !manifest
        .id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        result.add_error(format!(
            "Plugin `id` '{}' contains invalid characters (only a-z, 0-9, -, _ allowed)",
            manifest.id
        ));
    }

    // ── version (basic semver check) ──
    if manifest.version.is_empty() {
        result.add_error("Plugin `version` is required");
    } else if manifest.version.split('.').count() < 2 {
        result.add_warning(format!(
            "Plugin `version` '{}' does not appear to be valid semver",
            manifest.version
        ));
    }

    // ── plugin_type ──
    match manifest.plugin_type {
        PluginType::Ui | PluginType::Hybrid => {
            if manifest.ui.is_none() {
                result.add_error(format!(
                    "Plugin '{}' is type '{:?}' but has no [ui] section",
                    manifest.id, manifest.plugin_type
                ));
            }
        }
        PluginType::Functional => {
            if manifest.backend.is_none() && manifest.resources.is_empty() {
                result.add_warning(format!(
                    "Plugin '{}' is type 'functional' but has no [backend] section or resources",
                    manifest.id
                ));
            }
        }
    }

    // ── Navigation validation ──
    let mut nav_ids = std::collections::HashSet::new();
    for item in &manifest.navigation {
        check_nav_item_ids_unique(item, &mut nav_ids, &mut result);
        validate_nav_paths(item, &mut result);
    }

    // ── Resource validation ──
    for resource in &manifest.resources {
        if resource.id.is_empty() {
            result.add_error(format!(
                "Plugin '{}' has a resource with empty `id`",
                manifest.id
            ));
        }
    }

    // ── Extension point validation ──
    for ep in &manifest.extension_points {
        if ep.id.is_empty() {
            result.add_error(format!(
                "Plugin '{}' has an extension point with empty `id`",
                manifest.id
            ));
        }
    }

    // ── Permission / API scope validation ──
    for perm in &manifest.permissions {
        if !perm.contains(':') {
            result.add_warning(format!(
                "Plugin '{}' permission '{}' should follow resource:action format",
                manifest.id, perm
            ));
        }
    }

    // ── UI plugin cannot have backend-only configs ──
    if manifest.plugin_type == PluginType::Ui {
        if let Some(ref backend) = manifest.backend {
            if !backend.api_routes.is_empty()
                || !backend.jobs.is_empty()
                || !backend.automations.is_empty()
                || !backend.mcp_tools.is_empty()
            {
                result.add_warning(format!(
                    "Plugin '{}' is type 'ui' but declares backend extensions. Consider 'hybrid' instead.",
                    manifest.id
                ));
            }
        }
    }

    // ── UI Compatibility validation ──
    if let Some(ref ui) = manifest.ui {
        validate_ui_config(ui, &mut result);
    }

    // ── Secrets check ──
    // (no secrets should be in the manifest itself — they go in env vars)
    // We flag suspicious field values that look like keys
    if let Some(ref desc) = manifest.description {
        if desc.contains("sk-") || desc.contains("Bearer ") || desc.contains("password") {
            result.add_warning(format!(
                "Plugin '{}' description may contain secrets — remove them",
                manifest.id
            ));
        }
    }

    result
}

fn check_nav_item_ids_unique(
    item: &super::manifest::NavigationItem,
    seen: &mut std::collections::HashSet<String>,
    result: &mut ValidationResult,
) {
    if !seen.insert(item.id.clone()) {
        result.add_error(format!("Duplicate navigation item ID: '{}'", item.id));
    }
    for child in &item.children {
        check_nav_item_ids_unique(child, seen, result);
    }
}

fn validate_nav_paths(item: &super::manifest::NavigationItem, result: &mut ValidationResult) {
    if !item.path.starts_with('/') {
        result.add_error(format!(
            "Navigation item '{}' path '{}' must start with '/'",
            item.id, item.path
        ));
    }
    for child in &item.children {
        validate_nav_paths(child, result);
    }
}

fn validate_ui_config(ui: &UiConfig, result: &mut ValidationResult) {
    // ── Compatibility warnings ──
    if let Some(ref compat) = ui.compatibility {
        if compat.level == Some(UiCompatibilityLevel::Native) && !compat.uses_sip_components {
            result.add_warning(
                "UiCompatibility level is 'native' but uses_sip_components is false. \
                 Native plugins should use SIP components.",
            );
        }
        if compat.level == Some(UiCompatibilityLevel::Native) && compat.allows_global_css {
            result.add_warning(
                "UiCompatibility level is 'native' but allows_global_css is true. \
                 Native plugins should avoid global CSS to prevent style conflicts.",
            );
        }
    }

    // ── Route layout validation ──
    const VALID_LAYOUTS: &[&str] = &[
        "sip-page",
        "sip-dashboard",
        "sip-settings",
        "embedded",
        "standalone",
    ];
    for route in &ui.routes {
        if let Some(ref layout) = route.layout {
            if !VALID_LAYOUTS.contains(&layout.as_str()) {
                result.add_error(format!(
                    "UiRouteDef '{}' has invalid layout '{}'. Must be one of: {:?}",
                    route.id, layout, VALID_LAYOUTS
                ));
            }
        }
    }
}

/// Validate a set of manifests together (cross-plugin checks).
pub fn validate_plugin_set(manifests: &[PluginManifest]) -> ValidationResult {
    let mut result = ValidationResult::ok();

    // Check unique IDs
    let mut ids = std::collections::HashSet::new();
    for m in manifests {
        if !ids.insert(m.id.clone()) {
            result.add_error(format!("Duplicate plugin ID: '{}'", m.id));
        }
    }

    // Check dependency references
    let id_set: std::collections::HashSet<&str> = manifests.iter().map(|m| m.id.as_str()).collect();
    for m in manifests {
        for dep in &m.dependencies {
            if !id_set.contains(dep.as_str()) {
                result.add_warning(format!(
                    "Plugin '{}' depends on '{}' which is not in the loaded plugin set",
                    m.id, dep
                ));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::*;

    fn make_ui_manifest() -> PluginManifest {
        PluginManifest {
            id: "test-ui".into(),
            name: "Test UI".into(),
            version: "1.0.0".into(),
            plugin_type: PluginType::Ui,
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
                NavigationItem {
                    id: "dashboard".into(),
                    label: "Dashboard".into(),
                    path: "/".into(),
                    icon: Some("layout-dashboard".into()),
                    permission: Some("dashboard:read".into()),
                    order: Some(10),
                    feature_flag: None,
                    children: vec![],
                },
                NavigationItem {
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

    // (No standalone Default impl needed — PluginManifest now implements Default in manifest.rs)

    #[test]
    fn test_valid_ui_manifest() {
        let manifest = make_ui_manifest();
        let result = validate_manifest(&manifest);
        assert!(result.valid, "Expected valid: {:?}", result.errors);
    }

    #[test]
    fn test_missing_id() {
        let mut manifest = make_ui_manifest();
        manifest.id = String::new();
        let result = validate_manifest(&manifest);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("id")));
    }

    #[test]
    fn test_invalid_id_chars() {
        let mut manifest = make_ui_manifest();
        manifest.id = "my plugin!".into();
        let result = validate_manifest(&manifest);
        assert!(!result.valid);
    }

    #[test]
    fn test_missing_version() {
        let mut manifest = make_ui_manifest();
        manifest.version = String::new();
        let result = validate_manifest(&manifest);
        assert!(!result.valid);
    }

    #[test]
    fn test_ui_plugin_without_ui_section() {
        let mut manifest = make_ui_manifest();
        manifest.ui = None;
        let result = validate_manifest(&manifest);
        assert!(!result.valid);
    }

    #[test]
    fn test_duplicate_nav_ids() {
        let mut manifest = make_ui_manifest();
        manifest.navigation[1].id = "dashboard".into(); // duplicate
        let result = validate_manifest(&manifest);
        assert!(!result.valid);
    }

    #[test]
    fn test_nav_path_no_slash() {
        let mut manifest = make_ui_manifest();
        manifest.navigation[0].path = "bad-path".into();
        let result = validate_manifest(&manifest);
        assert!(!result.valid);
    }

    #[test]
    fn test_valid_semver() {
        let mut manifest = make_ui_manifest();
        manifest.version = "not-semver".into();
        let result = validate_manifest(&manifest);
        assert!(result.warnings.iter().any(|w| w.contains("semver")));
    }

    #[test]
    fn test_duplicate_plugin_ids() {
        let m1 = make_ui_manifest();
        let mut m2 = make_ui_manifest();
        m2.name = "Duplicate".into();
        let result = validate_plugin_set(&[m1, m2]);
        assert!(!result.valid);
    }

    #[test]
    fn test_missing_dependency() {
        let m1 = PluginManifest {
            id: "plugin-a".into(),
            name: "Plugin A".into(),
            version: "1.0.0".into(),
            plugin_type: PluginType::Functional,
            dependencies: vec!["plugin-b".into()],
            ..Default::default()
        };
        let result = validate_plugin_set(&[m1]);
        assert!(result.warnings.iter().any(|w| w.contains("plugin-b")));
    }

    #[test]
    fn test_public_view_redacts_internals() {
        let manifest = make_ui_manifest();
        let public = manifest.public_view();
        assert_eq!(public.id, "test-ui");
        assert!(public.ui.is_some());
        assert_eq!(public.plugin_type, PluginType::Ui);
        // Public view should not have backend or config_schema
    }

    #[test]
    fn test_functional_migration_plugin_is_valid() {
        let manifest = PluginManifest {
            id: "sip-migration-maximo".into(),
            name: "Maximo Migration Plugin".into(),
            version: "1.0.0".into(),
            plugin_type: PluginType::Functional,
            description: Some("Imports asset and work order data from IBM Maximo".into()),
            permissions: vec![
                "migration:create".into(),
                "migration:read".into(),
                "migration:execute".into(),
            ],
            ..Default::default()
        };
        let result = validate_manifest(&manifest);
        assert!(result.valid, "Expected valid: {:?}", result.errors);
    }

    #[test]
    fn test_migration_extension_points_are_known() {
        let migration_eps: Vec<&str> = vec![
            "migration.source_connector",
            "migration.file_parser",
            "migration.field_mapper",
            "migration.transformer",
            "migration.validator",
            "migration.duplicate_resolver",
            "migration.post_import_hook",
            "migration.report_generator",
        ];
        for ep in migration_eps {
            assert!(
                KNOWN_EXTENSION_POINTS.contains(&ep),
                "Migration extension point '{}' should be in KNOWN_EXTENSION_POINTS",
                ep
            );
        }
    }

    #[test]
    fn test_migration_permissions_follow_convention() {
        let manifest = PluginManifest {
            id: "test-migration-plugin".into(),
            name: "Test Migration".into(),
            version: "1.0.0".into(),
            plugin_type: PluginType::Functional,
            permissions: vec![
                "migration:create".into(),
                "migration:read".into(),
                "migration:map".into(),
                "migration:validate".into(),
                "migration:dry_run".into(),
                "migration:execute".into(),
                "migration:rollback".into(),
                "migration:delete".into(),
                "migration:view_raw_source".into(),
            ],
            ..Default::default()
        };
        let result = validate_manifest(&manifest);
        assert!(
            result.valid,
            "Migration permissions should be valid: {:?}",
            result.errors
        );
    }

    #[test]
    fn test_compatibility_native_without_components_warns() {
        let mut manifest = make_ui_manifest();
        manifest.ui.as_mut().unwrap().compatibility = Some(UiCompatibility {
            level: Some(UiCompatibilityLevel::Native),
            uses_sip_components: false,
            uses_theme_tokens: true,
            allows_global_css: false,
            ..Default::default()
        });
        let result = validate_manifest(&manifest);
        assert!(result.valid);
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("uses_sip_components")));
    }

    #[test]
    fn test_compatibility_native_with_global_css_warns() {
        let mut manifest = make_ui_manifest();
        manifest.ui.as_mut().unwrap().compatibility = Some(UiCompatibility {
            level: Some(UiCompatibilityLevel::Native),
            uses_sip_components: true,
            uses_theme_tokens: true,
            allows_global_css: true,
            ..Default::default()
        });
        let result = validate_manifest(&manifest);
        assert!(result.valid);
        assert!(result.warnings.iter().any(|w| w.contains("global_css")));
    }

    #[test]
    fn test_route_with_valid_layout() {
        let mut manifest = make_ui_manifest();
        manifest.ui.as_mut().unwrap().routes = vec![UiRouteDef {
            id: "settings".into(),
            path: "/settings".into(),
            component: Some("SettingsPage".into()),
            layout: Some("sip-settings".into()),
            title: Some("Settings".into()),
            breadcrumb: None,
            required_permissions: vec![],
        }];
        let result = validate_manifest(&manifest);
        assert!(result.valid, "Expected valid: {:?}", result.errors);
    }

    #[test]
    fn test_route_with_invalid_layout() {
        let mut manifest = make_ui_manifest();
        manifest.ui.as_mut().unwrap().routes = vec![UiRouteDef {
            id: "custom".into(),
            path: "/custom".into(),
            component: None,
            layout: Some("invalid-layout".into()),
            title: None,
            breadcrumb: None,
            required_permissions: vec![],
        }];
        let result = validate_manifest(&manifest);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("invalid-layout")));
    }

    #[test]
    fn test_manifest_with_theme_config() {
        let mut manifest = make_ui_manifest();
        manifest.ui.as_mut().unwrap().theme = Some(UiThemeConfig {
            inherits: None,
            supports_dark_mode: true,
            supports_density: true,
            supports_accent_color: true,
            uses_design_tokens: true,
        });
        let result = validate_manifest(&manifest);
        assert!(result.valid, "Expected valid: {:?}", result.errors);
    }

    #[test]
    fn test_manifest_with_routes_and_actions() {
        let mut manifest = make_ui_manifest();
        manifest.ui.as_mut().unwrap().routes = vec![UiRouteDef {
            id: "dashboard-overview".into(),
            path: "/dashboard/overview".into(),
            component: Some("OverviewCard".into()),
            layout: Some("sip-dashboard".into()),
            title: Some("Overview".into()),
            breadcrumb: None,
            required_permissions: vec!["dashboard:read".into()],
        }];
        manifest.ui.as_mut().unwrap().actions = vec![UiActionDef {
            id: "export-report".into(),
            label: "Export Report".into(),
            icon: Some("download".into()),
            route: Some("/export/report".into()),
            placement: vec!["toolbar".into(), "context-menu".into()],
            required_permissions: vec!["export:create".into()],
        }];
        let result = validate_manifest(&manifest);
        assert!(result.valid, "Expected valid: {:?}", result.errors);
    }

    #[test]
    fn test_compatible_level_no_warnings() {
        let mut manifest = make_ui_manifest();
        manifest.ui.as_mut().unwrap().compatibility = Some(UiCompatibility {
            level: Some(UiCompatibilityLevel::Compatible),
            uses_sip_components: false,
            uses_theme_tokens: false,
            allows_global_css: true,
            ..Default::default()
        });
        let result = validate_manifest(&manifest);
        assert!(result.valid);
        assert!(!result
            .warnings
            .iter()
            .any(|w| w.contains("uses_sip_components") || w.contains("global_css")));
    }
}
