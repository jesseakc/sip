use std::collections::HashMap;

/// Represents a declared extension point that plugins can contribute to.
#[derive(Debug, Clone)]
pub struct ExtensionPointDef {
    pub id: String,
    pub description: String,
    pub plugin_type: String, // "ui" or "functional"
}

/// Known SIP extension points that plugins can contribute to.
/// These IDs are stable and documented.
pub fn known_extension_points() -> Vec<ExtensionPointDef> {
    vec![
        // ── Shell ──
        ExtensionPointDef {
            id: "app.shell.sidebar.nav".into(),
            description: "Sidebar navigation items".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "app.shell.topbar.actions".into(),
            description: "Top bar action buttons".into(),
            plugin_type: "ui".into(),
        },
        // ── Dashboard ──
        ExtensionPointDef {
            id: "dashboard.cards".into(),
            description: "Dashboard summary card widgets".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "dashboard.widgets".into(),
            description: "Dashboard detailed widgets".into(),
            plugin_type: "ui".into(),
        },
        // ── Assets ──
        ExtensionPointDef {
            id: "asset.list.columns".into(),
            description: "Additional columns in asset list view".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "asset.detail.tabs".into(),
            description: "Additional tabs on asset detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "asset.detail.sidebar".into(),
            description: "Sidebar panels on asset detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "asset.detail.actions".into(),
            description: "Action buttons on asset detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "asset.create.form.sections".into(),
            description: "Additional form sections in asset creation".into(),
            plugin_type: "ui".into(),
        },
        // ── Work Orders ──
        ExtensionPointDef {
            id: "work_order.list.columns".into(),
            description: "Additional columns in work order list view".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "work_order.detail.tabs".into(),
            description: "Additional tabs on work order detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "work_order.detail.sidebar".into(),
            description: "Sidebar panels on work order detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "work_order.detail.actions".into(),
            description: "Action buttons on work order detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "work_order.create.form.sections".into(),
            description: "Additional form sections in work order creation".into(),
            plugin_type: "ui".into(),
        },
        // ── Schedules ──
        ExtensionPointDef {
            id: "schedule.list.columns".into(),
            description: "Additional columns in schedule list view".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "schedule.detail.actions".into(),
            description: "Action buttons on schedule detail page".into(),
            plugin_type: "ui".into(),
        },
        // ── Inspections ──
        ExtensionPointDef {
            id: "inspection.list.columns".into(),
            description: "Additional columns in inspection list view".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "inspection.detail.panels".into(),
            description: "Additional panels on inspection detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "inspection.detail.actions".into(),
            description: "Action buttons on inspection detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "inspection_checklist.items".into(),
            description: "Additional checklist item types in inspections".into(),
            plugin_type: "ui".into(),
        },
        // ── Parts ──
        ExtensionPointDef {
            id: "parts.list.columns".into(),
            description: "Additional columns in parts list view".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "parts.detail.actions".into(),
            description: "Action buttons on parts detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "parts.create.form.sections".into(),
            description: "Additional form sections in part creation".into(),
            plugin_type: "ui".into(),
        },
        // ── Settings ──
        ExtensionPointDef {
            id: "settings.sections".into(),
            description: "Additional sections on the settings page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "settings.general.form".into(),
            description: "Additional fields in general settings form".into(),
            plugin_type: "ui".into(),
        },
        // ── Command Palette ──
        ExtensionPointDef {
            id: "command_palette.actions".into(),
            description: "Actions available in command palette".into(),
            plugin_type: "ui".into(),
        },
        // ── AI Chat ──
        ExtensionPointDef {
            id: "ai_chat.tools".into(),
            description: "AI agent tools available in chat".into(),
            plugin_type: "ui".into(),
        },
        // ── Documents ──
        ExtensionPointDef {
            id: "document.list.columns".into(),
            description: "Additional columns in document list view".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "document.detail.actions".into(),
            description: "Action buttons on document detail page".into(),
            plugin_type: "ui".into(),
        },
        // ── Locations ──
        ExtensionPointDef {
            id: "location.list.columns".into(),
            description: "Additional columns in location list view".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "location.detail.actions".into(),
            description: "Action buttons on location detail page".into(),
            plugin_type: "ui".into(),
        },
        // ── Teams / Users ──
        ExtensionPointDef {
            id: "team.list.columns".into(),
            description: "Additional columns in team list view".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "team.detail.actions".into(),
            description: "Action buttons on team detail page".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "user.list.columns".into(),
            description: "Additional columns in user list view".into(),
            plugin_type: "ui".into(),
        },
        ExtensionPointDef {
            id: "user.detail.actions".into(),
            description: "Action buttons on user detail page".into(),
            plugin_type: "ui".into(),
        },
        // ── Migration ──
        ExtensionPointDef {
            id: "migration.source_connector".into(),
            description: "Connects to external source systems for data extraction".into(),
            plugin_type: "functional".into(),
        },
        ExtensionPointDef {
            id: "migration.file_parser".into(),
            description: "Parses file formats (CSV, JSON, XML) into source records".into(),
            plugin_type: "functional".into(),
        },
        ExtensionPointDef {
            id: "migration.field_mapper".into(),
            description: "Maps source fields to canonical SIP fields".into(),
            plugin_type: "functional".into(),
        },
        ExtensionPointDef {
            id: "migration.transformer".into(),
            description: "Transforms field values during mapping (e.g. date format, lookup)".into(),
            plugin_type: "functional".into(),
        },
        ExtensionPointDef {
            id: "migration.validator".into(),
            description: "Validates staged records against business rules".into(),
            plugin_type: "functional".into(),
        },
        ExtensionPointDef {
            id: "migration.duplicate_resolver".into(),
            description: "Resolves duplicate candidates detected during import".into(),
            plugin_type: "functional".into(),
        },
        ExtensionPointDef {
            id: "migration.post_import_hook".into(),
            description: "Hook called after each import run completes".into(),
            plugin_type: "functional".into(),
        },
        ExtensionPointDef {
            id: "migration.report_generator".into(),
            description: "Generates formatted migration reports".into(),
            plugin_type: "functional".into(),
        },
    ]
}

/// For backward compat with the old HashMap-based extension registry.
pub type ExtensionRegistry = HashMap<String, Vec<String>>;

/// Create an empty extension registry.
pub fn default_extension_points() -> ExtensionRegistry {
    HashMap::new()
}
