# Plugin Examples

Step-by-step walkthroughs for building SIP plugins.

---

## Example 1: Adding a Custom Navigation Item

The simplest plugin — adds a link to the sidebar.

### 1. Create the plugin directory

```bash
mkdir -p plugins/my-custom-link
```

### 2. Create `plugin.toml`

```toml
# plugins/my-custom-link/plugin.toml
id = "my-custom-link"
name = "My Custom Link"
version = "1.0.0"
description = "Adds a custom navigation item to the SIP sidebar."
author = "Your Name"
license = "MIT"
sip_version = "0.1.0"
plugin_type = "ui"
enabled_by_default = true

[ui]
kind = "web"
framework = "nextjs"

[[navigation]]
id = "custom-link"
label = "External Docs"
path = "https://docs.example.com"
icon = "external-link"
order = 200
```

### 3. Restart SIP

```bash
docker compose restart sip-api
```

### 4. Verify

```bash
curl http://localhost:8000/api/v1/ui/navigation | jq '.data[] | select(.id == "custom-link")'
```

The sidebar now shows "External Docs" at the bottom.

---

## Example 2: A Functional Plugin with an API Route

A backend plugin that exposes a custom endpoint.

### 1. Create the plugin directory

```bash
mkdir -p plugins/hello-world
```

### 2. Create `plugin.toml`

```toml
# plugins/hello-world/plugin.toml
id = "hello-world"
name = "Hello World Plugin"
version = "1.0.0"
description = "A functional plugin that adds a /hello endpoint."
author = "Your Name"
license = "MIT"
sip_version = "0.1.0"
plugin_type = "functional"
enabled_by_default = true

permissions = ["hello:read"]

[backend]
api_routes = ["/api/v1/hello"]
```

### 3. Implement the backend handler

> **Note:** The current plugin framework handles manifest loading and discovery. Backend route registration from plugin manifests is a roadmap feature. For now, functional plugin manifests are **declarative** — they document what the plugin provides. The actual route implementation goes in the `sip-api` crate (or a future plugin SDK).

```rust
// In sip-api/src/routes/ (future plugin SDK):
pub async fn hello_world() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "data": { "message": "Hello from the plugin!" }
    }))
}
```

### 4. Verify the manifest is loaded

```bash
curl http://localhost:8000/api/v1/plugins/hello-world | jq .
```

```json
{
  "data": {
    "id": "hello-world",
    "name": "Hello World Plugin",
    "plugin_type": "functional",
    "enabled": true
  }
}
```

---

## Example 3: A Hybrid Plugin (UI + Backend)

A plugin that adds a "Warranty Tracker" page to the sidebar **and** a backend job to check expiring warranties.

### `plugin.toml`

```toml
# plugins/warranty-tracker/plugin.toml
id = "warranty-tracker"
name = "Warranty Tracker"
version = "1.0.0"
description = "Track asset warranties with expiry alerts."
author = "Your Name"
license = "AGPL-3.0-or-later"
sip_version = "0.1.0"
plugin_type = "hybrid"
enabled_by_default = true

permissions = [
    "warranties:read",
    "warranties:write",
    "assets:read",
]

dependencies = ["sip-core-ui"]

[ui]
kind = "web"
framework = "nextjs"

[[navigation]]
id = "warranties"
label = "Warranties"
path = "/warranties"
icon = "shield"
permission = "warranties:read"
order = 65

[[extension_points]]
id = "asset.detail.tabs"
type = "ui"
metadata = { tab_id = "warranty", label = "Warranty", order = 30 }

[[extension_points]]
id = "asset.detail.sidebar"
type = "ui"
metadata = { panel_id = "warranty_status", label = "Warranty Status" }

[backend]
api_routes = ["/api/v1/warranties"]

[[backend.jobs]]
id = "check-expiring-warranties"
name = "Check Expiring Warranties"
description = "Scans assets for warranties expiring within 30 days and sends alerts."
schedule = "0 8 * * *"

[marketplace]
category = "integration"
tags = ["warranty", "compliance", "alerts"]
```

### Frontend Page (Next.js)

```tsx
// plugins/warranty-tracker/src/pages/warranties.tsx (future SDK)
// For now, build pages as part of the sip-core-ui frontend or as a separate
// micro-frontend served at the plugin's dev_url.

export default function WarrantiesPage() {
  return (
    <div>
      <h1>Warranty Tracker</h1>
      {/* Fetch /api/v1/warranties and render */}
    </div>
  );
}
```

---

## Example 4: An Integration Plugin (Webhooks + MCP Tools)

A plugin that connects SIP to an external service (e.g., Slack) and provides MCP tools for AI agents.

### `plugin.toml`

```toml
# plugins/slack-integration/plugin.toml
id = "slack-integration"
name = "Slack Integration"
version = "1.0.0"
description = "Send work order notifications to Slack and provide AI agent tools."
author = "Your Name"
license = "MIT"
sip_version = "0.1.0"
plugin_type = "functional"
enabled_by_default = true

permissions = [
    "work_orders:read",
    "notifications:send",
]

[backend]
api_routes = ["/api/v1/integrations/slack"]

[[backend.webhooks]]
id = "wo-created-slack"
name = "Work Order Created → Slack"
description = "Posts a message to Slack when a new work order is created."
event = "work_order.created"
url = "https://hooks.slack.com/services/..."
secret_env = "SLACK_WEBHOOK_SECRET"

[[backend.mcp_tools]]
id = "search_work_orders_by_asset"
name = "Search Work Orders by Asset"
description = "Find all work orders for a given asset, optionally filtered by status."
input_schema = {
  type = "object",
  properties = {
    asset_id = { type = "string", description = "UUID of the asset" },
    status = { type = "string", description = "Filter by status (open, completed, etc.)" }
  },
  required = ["asset_id"]
}

[[backend.ai_tools]]
id = "suggest_parts"
name = "Suggest Replacement Parts"
description = "Given an asset model, suggest common replacement parts based on maintenance history."
function = "suggest_replacement_parts"
parameters = {
  type = "object",
  properties = {
    asset_model_id = { type = "string", description = "UUID of the asset model" }
  },
  required = ["asset_model_id"]
}

[config_schema]
schema = {
  type = "object",
  properties = {
    slack_webhook_url = { type = "string", description = "Slack incoming webhook URL" },
    slack_channel = { type = "string", description = "Default Slack channel" }
  }
}

[marketplace]
category = "integration"
tags = ["slack", "notifications", "messaging", "ai-tools"]
```

---

## Example 5: A Mobile App Plugin

A UI plugin that represents a companion mobile app.

### `plugin.toml`

```toml
# plugins/sip-mobile/plugin.toml
id = "sip-mobile"
name = "SIP Mobile Companion"
version = "1.0.0"
description = "Mobile companion app for technicians in the field."
author = "SIP Community"
license = "AGPL-3.0-or-later"
sip_version = "0.1.0"
plugin_type = "ui"
enabled_by_default = false

[ui]
kind = "mobile"
framework = "flutter"
entrypoint = "sipmobile://"
dev_url = "http://localhost:8080"

[[navigation]]
id = "mobile-app"
label = "Mobile App"
path = "/mobile"
icon = "smartphone"
order = 500

[marketplace]
category = "ui"
tags = ["mobile", "flutter", "technician", "field-service"]
homepage = "https://github.com/sip-community/sip-mobile"
```

---

## Plugin Development Checklist

Use this checklist when building a new SIP plugin:

- [ ] **Directory**: Created `plugins/{plugin-id}/plugin.toml`
- [ ] **ID**: Unique, lowercase, `[a-z0-9_-]` only, matches directory name
- [ ] **Type**: Correctly set to `"ui"`, `"functional"`, or `"hybrid"`
- [ ] **UI section**: Present if type is `"ui"` or `"hybrid"`
- [ ] **Navigation**: Paths start with `/`, IDs are unique, icons from the Lucide set
- [ ] **Permissions**: Follow `resource:action` format
- [ ] **Dependencies**: Listed if depending on other plugins
- [ ] **No secrets**: No API keys, passwords, or tokens in the manifest
- [ ] **Validation**: `cargo test -p sip-plugins` passes
- [ ] **Discovery**: `curl http://localhost:8000/api/v1/plugins/{plugin-id}` returns the plugin
- [ ] **Navigation**: `curl http://localhost:8000/api/v1/ui/navigation` includes your items (if UI plugin)
- [ ] **Marketplace**: Optional metadata filled in for discoverability

## Troubleshooting

### Plugin not showing up in `/api/v1/plugins`

1. Check that the manifest is in `plugins/{plugin-id}/plugin.toml`
2. Check that the `id` field matches the directory name
3. Restart the SIP API: `docker compose restart sip-api`
4. Check API logs for validation errors

### Navigation items not appearing in the sidebar

1. Verify the plugin type is `"ui"` or `"hybrid"`
2. Verify the plugin is enabled (`enabled_by_default = true`)
3. Check `curl http://localhost:8000/api/v1/ui/navigation` — does it include your items?
4. Check the frontend console for fetch errors

### Manifest validation errors

Run `cargo test -p sip-plugins` to run the validation test suite. Common issues:
- Plugin ID uses invalid characters (spaces, uppercase, special chars)
- Navigation path doesn't start with `/`
- Duplicate navigation item IDs
- UI plugin missing `[ui]` section
