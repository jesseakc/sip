# Plugin Manifest Reference

Every SIP plugin is defined by a `plugin.toml` file. This document is the canonical reference for every field.

## File Location

```
plugins/{plugin-id}/plugin.toml
```

The `plugin-id` must match the `id` field in the manifest. Example: `plugins/sip-core-ui/plugin.toml` with `id = "sip-core-ui"`.

## Complete Specification

### `[identity]` — Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Unique plugin identifier. Only `a-z`, `0-9`, `-`, `_` allowed. Must match the directory name. Example: `"my-billing-plugin"` |
| `name` | string | **Yes** | Human-readable name. Example: `"Billing Module"` |
| `version` | string | **Yes** | Semantic version. Example: `"1.0.0"` |
| `description` | string | No | Short description of the plugin's purpose |
| `author` | string | No | Plugin author or organization |
| `license` | string | No | SPDX license identifier. Example: `"AGPL-3.0-or-later"` |
| `sip_version` | string | No | Minimum SIP version compatibility. Default: `"0.1.0"` |
| `plugin_type` | string | **Yes** | `"ui"`, `"functional"`, or `"hybrid"` |
| `enabled_by_default` | bool | No | Whether the plugin is enabled on first load. Default: `true` |
| `dependencies` | string[] | No | Plugin IDs this plugin depends on. Warnings are generated if dependencies are missing |
| `permissions` | string[] | No | Permission scopes the plugin requires. Format: `resource:action` (e.g., `"assets:read"`, `"work_orders:write"`) |
| `api_scopes` | string[] | No | API scope identifiers (future use) |
| `capabilities` | string[] | No | Declared capabilities (e.g., `"rag"`, `"semantic_search"`, `"pdf_export"`) |

### `[ui]` — UI Configuration

Required for `plugin_type = "ui"` or `"hybrid"`.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `kind` | string | **Yes** | UI type: `"web"`, `"mobile"`, `"desktop"`, `"cli"`, `"tui"`, `"external"` |
| `framework` | string | No | Framework: `"nextjs"`, `"react"`, `"vue"`, `"svelte"`, `"html"`, `"flutter"`, `"swift_ui"`, `"jvm"` |
| `entrypoint` | string | No | URL path for the UI entrypoint. Default: `"/"` |
| `dev_url` | string | No | Development server URL. Example: `"http://localhost:3000"` |
| `production_mount` | string | No | Production mount path. Default: `"/"` |
| `api_base_env` | string | No | Environment variable name for the API base URL. Example: `"NEXT_PUBLIC_API_URL"` |

Example:
```toml
[ui]
kind = "web"
framework = "nextjs"
entrypoint = "/"
dev_url = "http://localhost:3000"
production_mount = "/"
api_base_env = "NEXT_PUBLIC_API_URL"
```

### `[[navigation]]` — Navigation Items

An array of navigation items. Each item contributes to the aggregated sidebar.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Unique nav item ID within the plugin |
| `label` | string | **Yes** | Display text |
| `path` | string | **Yes** | URL path. Must start with `/` |
| `icon` | string | No | Icon name from the SIP icon set (see below) |
| `permission` | string | No | Permission required to see this item. Format: `resource:action` |
| `order` | int | No | Display order (lower = first). Default: 999 |
| `feature_flag` | string | No | Feature flag that gates this item (future use) |
| `children` | nav[] | No | Nested sub-navigation items |

Example:
```toml
[[navigation]]
id = "dashboard"
label = "Dashboard"
path = "/"
icon = "layout-dashboard"
permission = "dashboard:read"
order = 10

[[navigation]]
id = "assets"
label = "Assets"
path = "/assets"
icon = "packages"
permission = "assets:read"
order = 20
```

Navigation items from all enabled UI plugins are merged, sorted by `order` (ascending), and served at `GET /api/v1/ui/navigation`.

### `[[resources]]` — Resource Definitions

Declares entities the plugin understands. These are used to auto-generate list views, detail views, forms, and search interfaces.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Resource identifier (e.g., `"asset"`, `"work_order"`) |
| `label` | string | **Yes** | Singular label (e.g., `"Asset"`) |
| `plural_label` | string | **Yes** | Plural label (e.g., `"Assets"`) |
| `route_base` | string | No | Base URL path for the resource |
| `icon` | string | No | Icon name |
| `search_fields` | string[] | No | Fields indexed for full-text search |
| `default_sort` | string | No | Default sort expression (e.g., `"name"`, `"created_at DESC"`) |
| `feature_flag` | string | No | Feature flag dependency |

#### `[[resources.columns]]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field` | string | **Yes** | Field name in the API response |
| `label` | string | **Yes** | Column header |
| `sortable` | bool | No | Whether the column is sortable. Default: `false` |
| `filterable` | bool | No | Whether the column can be filtered. Default: `false` |
| `width` | string | No | CSS width hint (e.g., `"120px"`, `"20%"`). Default: auto |

#### `[[resources.permissions]]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action` | string | **Yes** | Action name (e.g., `"read"`, `"write"`, `"delete"`) |
| `scope` | string | **Yes** | Scope (e.g., `"org"`, `"tenant"`, `"global"`) |

#### `[[resources.actions]]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Action identifier |
| `label` | string | **Yes** | Display label |
| `permission` | string | No | Permission required |
| `confirmation` | string | No | Confirmation message before executing |

#### `[[resources.filters]]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Filter identifier |
| `label` | string | **Yes** | Display label |
| `type` | string | **Yes** | Filter type: `"select"`, `"multi_select"`, `"date_range"`, `"text"`, `"boolean"` |
| `options` | object | No | For select types: `{ "value": "label" }` mapping |

Example:
```toml
[[resources]]
id = "asset"
label = "Asset"
plural_label = "Assets"
route_base = "/assets"
icon = "packages"
search_fields = ["name", "serial_number", "description"]
default_sort = "name"

[[resources.columns]]
field = "name"
label = "Name"
sortable = true

[[resources.columns]]
field = "status"
label = "Status"
sortable = true

[[resources.actions]]
id = "archive"
label = "Archive"
permission = "assets:write"
confirmation = "Archive this asset?"
```

### `[[extension_points]]` — Extension Point Declarations

Declares which extension points this plugin contributes to.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Extension point ID (must match a known ID or be custom) |
| `plugin_id` | string | No | Target plugin ID (for cross-plugin contributions) |
| `type` | string | **Yes** | `"ui"` or `"functional"` |
| `metadata` | object | No | Arbitrary metadata for the extension |

Example:
```toml
[[extension_points]]
id = "app.shell.sidebar.nav"
type = "ui"

[[extension_points]]
id = "asset.detail.tabs"
type = "ui"
metadata = { tab_id = "warranty", label = "Warranty" }
```

See [Extension Points](./extension-points.md) for the full catalog.

### `[backend]` — Backend Extension (Functional/Hybrid plugins)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `api_routes` | string[] | No | Additional API route prefixes the plugin handles |
| `jobs` | JobDef[] | No | Scheduled background jobs |
| `automations` | AutomationDef[] | No | Trigger-action automation rules |
| `event_subscribers` | string[] | No | Event types the plugin listens for |
| `data_model_extensions` | string[] | No | New tables or columns the plugin adds |
| `migrations` | string[] | No | Migration file paths |
| `webhooks` | WebhookDef[] | No | Outgoing webhook registrations |
| `mcp_tools` | McpToolDef[] | No | MCP (Model Context Protocol) tools for AI agents |
| `ai_tools` | AiToolDef[] | No | AI agent tools the plugin provides |
| `external_integrations` | IntegrationDef[] | No | External service connectors |

#### JobDef

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Unique job identifier |
| `name` | string | **Yes** | Display name |
| `description` | string | No | What the job does |
| `schedule` | string | **Yes** | Cron expression (e.g., `"0 */6 * * *"` for every 6 hours) |

#### AutomationDef

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Unique automation identifier |
| `name` | string | **Yes** | Display name |
| `description` | string | No | What the automation does |
| `trigger` | string | **Yes** | Event trigger (e.g., `"work_order.completed"`) |
| `action` | string | **Yes** | Action description (e.g., `"send_notification"`) |

#### McpToolDef

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Tool identifier |
| `name` | string | **Yes** | Display name |
| `description` | string | **Yes** | What the tool does (shown to LLM) |
| `input_schema` | object | **Yes** | JSON Schema for tool parameters |

```toml
[[backend.mcp_tools]]
id = "search_work_orders"
name = "Search Work Orders"
description = "Search work orders by asset ID, status, or keyword"
input_schema = { type = "object", properties = { query = { type = "string" } } }
```

#### AiToolDef

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Tool identifier |
| `name` | string | **Yes** | Display name |
| `description` | string | **Yes** | What the tool does (shown to LLM) |
| `function` | string | **Yes** | Function name to call |
| `parameters` | object | **Yes** | JSON Schema for function parameters |

#### WebhookDef

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Webhook identifier |
| `name` | string | **Yes** | Display name |
| `description` | string | No | What triggers this webhook |
| `event` | string | **Yes** | Event type (e.g., `"work_order.closed"`) |
| `url` | string | **Yes** | Target URL |
| `secret_env` | string | No | Environment variable holding the webhook secret |

#### IntegrationDef

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Integration identifier |
| `name` | string | **Yes** | Display name |
| `description` | string | No | What the integration connects to |
| `provider` | string | **Yes** | Provider name (e.g., `"salesforce"`, `"sap"`, `"slack"`) |
| `config_schema` | object | No | JSON Schema for integration configuration |

### `[config_schema]` — Plugin Configuration Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `schema` | object | **Yes** | JSON Schema describing the plugin's config options |

```toml
[config_schema]
schema = { type = "object", properties = { api_key = { type = "string", description = "API key for the service" } } }
```

### `[marketplace]` — Marketplace Metadata (Optional)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `price` | string | No | Price or `"free"` |
| `category` | string | No | Category: `"ui"`, `"integration"`, `"automation"`, `"reporting"`, `"industry"` |
| `tags` | string[] | No | Search tags |
| `screenshots` | string[] | No | Screenshot URLs |
| `homepage` | string | No | Plugin homepage URL |
| `repository` | string | No | Source code repository URL |
| `support_email` | string | No | Support contact email |

```toml
[marketplace]
category = "ui"
tags = ["official", "web", "nextjs", "administration"]
homepage = "https://github.com/jesseakc/sip"
```

## Icon Names

The SIP frontend uses [Lucide](https://lucide.dev) icons. Available icon names:

| Icon Name | Visual | Common Use |
|-----------|--------|------------|
| `layout-dashboard` | Grid layout | Dashboard |
| `packages` | Box | Assets, inventory |
| `clipboard-list` | Clipboard | Work orders, tasks |
| `calendar` | Calendar | Schedules, dates |
| `clipboard-check` | Checked clipboard | Inspections, audits |
| `wrench` | Wrench | Parts, maintenance |
| `message-square` | Chat bubble | AI Chat, messaging |
| `map-pin` | Map pin | Locations, sites |
| `users` | People | Teams, groups |
| `user` | Person | Users, profile |
| `file-text` | Document | Documents, files |
| `settings` | Gear | Settings, configuration |
| `bell` | Bell | Notifications |
| `bar-chart` | Bar chart | Reports, analytics |
| `truck` | Truck | Logistics, shipping |
| `building` | Building | Facilities |
| `shield` | Shield | Security, compliance |
| `zap` | Lightning | Quick actions |
| `search` | Magnifier | Search |
| `plus-circle` | Plus circle | Create, add |
| `trash-2` | Trash | Delete |
| `external-link` | External link | External links |
| `download` | Download | Export, download |

See [lucide.dev/icons](https://lucide.dev/icons) for the full set.

## Validation Rules

The plugin registry validates every manifest on load:

1. **`id`** must be non-empty and contain only `[a-z0-9_-]`
2. **`version`** must be non-empty and should follow semver format
3. **`plugin_type`** must be `"ui"`, `"functional"`, or `"hybrid"`
4. **UI plugins** must have a `[ui]` section
5. **Navigation item IDs** must be unique within the plugin
6. **Navigation paths** must start with `/`
7. **Duplicate plugin IDs** across plugins cause an error
8. **Missing dependency references** generate warnings
9. **UI plugins** with backend-only declarations (jobs, MCP tools) get a warning — consider `"hybrid"` instead
10. **Descriptions** containing strings that look like API keys (`sk-`) trigger a warning

## Version Compatibility

| Field | Meaning |
|-------|---------|
| `sip_version` | The plugin declares it is compatible with this SIP version or higher. SIP does not currently enforce a strict semver range check, but the field is reserved for future use. |

## Security Notes

- **Never put API keys, passwords, or secrets in `plugin.toml`.** Use environment variables referenced by the plugin's configuration.
- **The public API (`/api/v1/plugins`, `/api/v1/ui/*`) never exposes secrets.** The `public_view()` method strips internal configuration before serializing.
- **Permissions declared in the manifest are informational.** Actual enforcement happens at the API layer via JWT claims and RBAC.
- **No arbitrary code execution.** Manifest files are parsed as TOML only. No remote module loading, eval, or component injection is supported in the current architecture.
