use serde::{Deserialize, Serialize};

// ─── Plugin Type ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    Ui,
    Functional,
    Hybrid,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Ui => write!(f, "ui"),
            PluginType::Functional => write!(f, "functional"),
            PluginType::Hybrid => write!(f, "hybrid"),
        }
    }
}

// ─── UI Kind ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UiKind {
    Web,
    Mobile,
    Desktop,
    Cli,
    Tui,
    External,
}

impl std::fmt::Display for UiKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiKind::Web => write!(f, "web"),
            UiKind::Mobile => write!(f, "mobile"),
            UiKind::Desktop => write!(f, "desktop"),
            UiKind::Cli => write!(f, "cli"),
            UiKind::Tui => write!(f, "tui"),
            UiKind::External => write!(f, "external"),
        }
    }
}

// ─── UI Framework ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UiFramework {
    Nextjs,
    React,
    Vue,
    Svelte,
    Html,
    Flutter,
    SwiftUi,
    Jvm,
    Custom(String),
}

// ─── Navigation ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationItem {
    pub id: String,
    pub label: String,
    pub path: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub permission: Option<String>,
    #[serde(default)]
    pub order: Option<i32>,
    #[serde(default)]
    pub feature_flag: Option<String>,
    #[serde(default)]
    pub children: Vec<NavigationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationSection {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub items: Vec<NavigationItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    #[serde(default)]
    pub enabled: bool,
    pub kind: UiKind,
    #[serde(default)]
    pub framework: Option<UiFramework>,
    #[serde(default)]
    pub entrypoint: Option<String>,
    #[serde(default)]
    pub dev_url: Option<String>,
    #[serde(default)]
    pub production_mount: Option<String>,
    #[serde(default)]
    pub api_base_env: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub compatibility: Option<UiCompatibility>,
    #[serde(default)]
    pub theme: Option<UiThemeConfig>,
    #[serde(default)]
    pub routes: Vec<UiRouteDef>,
    #[serde(default)]
    pub actions: Vec<UiActionDef>,
}

// ─── Backend Extension ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    #[serde(default)]
    pub api_routes: Vec<String>,
    #[serde(default)]
    pub jobs: Vec<JobDef>,
    #[serde(default)]
    pub automations: Vec<AutomationDef>,
    #[serde(default)]
    pub event_subscribers: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub data_model_extensions: Vec<String>,
    #[serde(default)]
    pub migrations: Vec<String>,
    #[serde(default)]
    pub webhooks: Vec<WebhookDef>,
    #[serde(default)]
    pub mcp_tools: Vec<McpToolDef>,
    #[serde(default)]
    pub ai_tools: Vec<AiToolDef>,
    #[serde(default)]
    pub external_integrations: Vec<IntegrationDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub schedule: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub trigger: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub event: String,
    pub url: String,
    #[serde(default)]
    pub secret_env: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiToolDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub function: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub provider: String,
    #[serde(default)]
    pub config_schema: Option<serde_json::Value>,
}

// ─── UI Compatibility Level ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UiCompatibilityLevel {
    Native,
    Compatible,
    Standalone,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiCompatibility {
    #[serde(default)]
    pub level: Option<UiCompatibilityLevel>,
    #[serde(default)]
    pub sip_ui_version: Option<String>,
    #[serde(default = "default_true")]
    pub requires_shell: bool,
    #[serde(default)]
    pub uses_sip_components: bool,
    #[serde(default)]
    pub uses_theme_tokens: bool,
    #[serde(default = "default_true")]
    pub allows_global_css: bool,
}

// ─── UI Theme ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiThemeConfig {
    #[serde(default)]
    pub inherits: Option<String>,
    #[serde(default)]
    pub supports_dark_mode: bool,
    #[serde(default)]
    pub supports_density: bool,
    #[serde(default)]
    pub supports_accent_color: bool,
    #[serde(default)]
    pub uses_design_tokens: bool,
}

// ─── UI Route ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiRouteDef {
    pub id: String,
    pub path: String,
    #[serde(default)]
    pub component: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub breadcrumb: Option<String>,
    #[serde(default)]
    pub required_permissions: Vec<String>,
}

// ─── UI Action ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiActionDef {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub placement: Vec<String>,
    #[serde(default)]
    pub required_permissions: Vec<String>,
}

// ─── Resources ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDef {
    pub id: String,
    pub label: String,
    pub plural_label: String,
    #[serde(default)]
    pub route_base: Option<String>,
    #[serde(default)]
    pub permissions: Vec<ResourcePermission>,
    #[serde(default)]
    pub list_view: Option<ViewDef>,
    #[serde(default)]
    pub detail_view: Option<ViewDef>,
    #[serde(default)]
    pub create_form: Option<FormDef>,
    #[serde(default)]
    pub edit_form: Option<FormDef>,
    #[serde(default)]
    pub filters: Vec<FilterDef>,
    #[serde(default)]
    pub actions: Vec<ActionDef>,
    #[serde(default)]
    pub search_fields: Vec<String>,
    #[serde(default)]
    pub default_sort: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub feature_flag: Option<String>,
    #[serde(default)]
    pub columns: Vec<ColumnDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePermission {
    pub action: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDef {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub extension_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormDef {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub sections: Vec<FormSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSection {
    pub id: String,
    pub label: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterDef {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub filter_type: String,
    #[serde(default)]
    pub options: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub permission: Option<String>,
    #[serde(default)]
    pub confirmation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub field: String,
    pub label: String,
    #[serde(default)]
    pub sortable: bool,
    #[serde(default)]
    pub filterable: bool,
    #[serde(default)]
    pub width: Option<String>,
}

// ─── Extension Points ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPointDecl {
    pub id: String,
    pub plugin_id: Option<String>,
    #[serde(rename = "type")]
    pub ext_type: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

// ─── Plugin Config ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigDef {
    pub schema: serde_json::Value,
}

// ─── Marketplace ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceMeta {
    #[serde(default)]
    pub price: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub screenshots: Vec<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub support_email: Option<String>,
}

// ─── Plugin Manifest (root) ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    // ── Identity ──
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default = "default_sip_version")]
    pub sip_version: String,

    // ── Type ──
    #[serde(default = "default_plugin_type")]
    pub plugin_type: PluginType,
    #[serde(default = "default_true")]
    pub enabled_by_default: bool,

    // ── Dependencies ──
    #[serde(default)]
    pub dependencies: Vec<String>,

    // ── Permissions ──
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub api_scopes: Vec<String>,

    // ── Capabilities ──
    #[serde(default)]
    pub capabilities: Vec<String>,

    // ── Config ──
    #[serde(default)]
    pub config_schema: Option<PluginConfigDef>,

    // ── UI ──
    #[serde(default)]
    pub ui: Option<UiConfig>,

    // ── Navigation ──
    /// Top-level navigation items contributed by this plugin.
    /// These are aggregated across all enabled UI plugins by the registry.
    #[serde(default)]
    pub navigation: Vec<NavigationItem>,

    // ── Backend ──
    #[serde(default)]
    pub backend: Option<BackendConfig>,

    // ── Resources ──
    #[serde(default)]
    pub resources: Vec<ResourceDef>,

    // ── Extension Points ──
    #[serde(default)]
    pub extension_points: Vec<ExtensionPointDecl>,

    // ── Marketplace ──
    #[serde(default)]
    pub marketplace: Option<MarketplaceMeta>,
}

// ─── Default functions ──────────────────────────────────────────────────────

fn default_sip_version() -> String {
    "0.1.0".to_string()
}

fn default_plugin_type() -> PluginType {
    PluginType::Functional
}

fn default_true() -> bool {
    true
}

// ─── Public-safe redaction ──────────────────────────────────────────────────

impl PluginManifest {
    /// Return a version of the manifest safe for public API exposure.
    /// Removes internal config, secrets, backend-only details.
    pub fn public_view(&self) -> PublicPluginInfo {
        PublicPluginInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            license: self.license.clone(),
            sip_version: self.sip_version.clone(),
            plugin_type: self.plugin_type.clone(),
            capabilities: self.capabilities.clone(),
            ui: self.ui.as_ref().map(|u| PublicUiInfo {
                kind: u.kind.clone(),
                framework: u.framework.clone(),
                entrypoint: u.entrypoint.clone(),
                dev_url: u.dev_url.clone(),
                production_mount: u.production_mount.clone(),
                enabled: u.enabled,
                compatibility: u.compatibility.clone(),
                theme: u.theme.clone(),
            }),
            navigation: self.navigation.clone(),
            resources: self.resources.iter().map(|r| ResourceDef {
                id: r.id.clone(),
                label: r.label.clone(),
                plural_label: r.plural_label.clone(),
                route_base: r.route_base.clone(),
                permissions: r.permissions.clone(),
                list_view: r.list_view.clone(),
                detail_view: r.detail_view.clone(),
                create_form: r.create_form.clone(),
                edit_form: r.edit_form.clone(),
                filters: r.filters.clone(),
                actions: r.actions.clone(),
                search_fields: r.search_fields.clone(),
                default_sort: r.default_sort.clone(),
                icon: r.icon.clone(),
                feature_flag: r.feature_flag.clone(),
                columns: r.columns.clone(),
            }).collect(),
            extension_points: self.extension_points.clone(),
            enabled: true, // populated by registry
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicPluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub sip_version: String,
    pub plugin_type: PluginType,
    pub capabilities: Vec<String>,
    pub ui: Option<PublicUiInfo>,
    pub navigation: Vec<NavigationItem>,
    pub resources: Vec<ResourceDef>,
    pub extension_points: Vec<ExtensionPointDecl>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUiInfo {
    pub kind: UiKind,
    pub framework: Option<UiFramework>,
    pub entrypoint: Option<String>,
    pub dev_url: Option<String>,
    pub production_mount: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    pub compatibility: Option<UiCompatibility>,
    pub theme: Option<UiThemeConfig>,
}

// ─── Default ────────────────────────────────────────────────────────────────

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: String::new(),
            description: None,
            author: None,
            license: None,
            sip_version: default_sip_version(),
            plugin_type: PluginType::Functional,
            enabled_by_default: true,
            dependencies: vec![],
            permissions: vec![],
            api_scopes: vec![],
            capabilities: vec![],
            config_schema: None,
            ui: None,
            navigation: vec![],
            backend: None,
            resources: vec![],
            extension_points: vec![],
            marketplace: None,
        }
    }
}

impl Default for UiKind {
    fn default() -> Self {
        UiKind::Web
    }
}

impl Default for UiFramework {
    fn default() -> Self {
        UiFramework::Nextjs
    }
}
