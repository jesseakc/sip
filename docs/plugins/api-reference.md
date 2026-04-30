# Plugin API Reference

All plugin discovery endpoints are **public** — no authentication required. This allows frontend clients, mobile apps, and AI agents to discover SIP capabilities before authenticating.

## Base URL

```
http://localhost:8000/api/v1
```

## Endpoints

### `GET /api/v1/plugins`

List all enabled plugins with safe public metadata.

**Response:**
```json
{
  "data": [
    {
      "id": "sip-core-ui",
      "name": "SIP Core UI",
      "version": "0.1.0",
      "description": "Official first-party web interface for the SIP Service Intelligence Platform.",
      "author": "SIP Project",
      "license": "AGPL-3.0-or-later",
      "sip_version": "0.1.0",
      "plugin_type": "ui",
      "capabilities": [],
      "ui": {
        "kind": "web",
        "framework": "nextjs",
        "entrypoint": "/",
        "dev_url": "http://localhost:3000",
        "production_mount": "/"
      },
      "navigation": [
        {
          "id": "dashboard",
          "label": "Dashboard",
          "path": "/",
          "icon": "layout-dashboard",
          "permission": "dashboard:read",
          "order": 10,
          "feature_flag": null,
          "children": []
        }
      ],
      "resources": [...],
      "extension_points": [...],
      "enabled": true
    }
  ]
}
```

**Notes:**
- Only enabled plugins are returned
- API keys, secrets, and internal config are never included
- `backend` sections are stripped from UI plugins in the public view

---

### `GET /api/v1/plugins/enabled`

List only the IDs of enabled plugins. Lightweight — useful for capability checks.

**Response:**
```json
{
  "data": ["sip-core-ui"]
}
```

---

### `GET /api/v1/plugins/:plugin_id`

Get public metadata for a single plugin.

**Response:** Same shape as a single item from `/api/v1/plugins`. Returns `{ "data": null, "error": "Plugin not found" }` for unknown IDs.

---

### `GET /api/v1/ui/navigation`

**The key endpoint for frontend integration.** Returns the aggregated sidebar navigation from all enabled UI plugins, sorted by `order`.

**Response:**
```json
{
  "data": [
    {
      "id": "dashboard",
      "label": "Dashboard",
      "path": "/",
      "icon": "layout-dashboard",
      "permission": "dashboard:read",
      "order": 10,
      "feature_flag": null,
      "children": []
    },
    {
      "id": "assets",
      "label": "Assets",
      "path": "/assets",
      "icon": "packages",
      "permission": "assets:read",
      "order": 20,
      "feature_flag": null,
      "children": []
    }
  ]
}
```

**Frontend integration (React/Next.js example):**
```typescript
useEffect(() => {
  fetch(`${API_BASE}/ui/navigation`)
    .then(res => res.json())
    .then(json => {
      if (Array.isArray(json?.data)) {
        setNavItems(json.data);
      }
    })
    .catch(() => {
      // Fall back to local defaults
    });
}, []);
```

**Notes:**
- Items from all enabled UI plugins are merged and sorted by `order` (ascending), then by plugin ID (stability tiebreaker)
- When a plugin is disabled, its navigation items are excluded
- The response is the same shape whether loaded from TOML manifests or from the embedded defaults

---

### `GET /api/v1/ui/plugins`

List only UI plugins (type `"ui"` or `"hybrid"`) with their navigation items.

**Response:**
```json
{
  "data": [
    {
      "id": "sip-core-ui",
      "name": "SIP Core UI",
      "version": "0.1.0",
      "description": "Official first-party web interface...",
      "ui": {
        "kind": "web",
        "framework": "nextjs",
        "entrypoint": "/",
        "dev_url": "http://localhost:3000",
        "production_mount": "/"
      },
      "navigation": [...],
      "enabled": true
    }
  ]
}
```

**Use case:** A mobile app or CLItool only cares about UI plugins. This endpoint filters to only UI-relevant results.

---

### `GET /api/v1/ui/capabilities`

List resources and extension points across all enabled plugins.

**Response:**
```json
{
  "data": {
    "resources": [
      {
        "id": "asset",
        "label": "Asset",
        "plural_label": "Assets",
        "route_base": "/assets",
        "icon": "packages",
        "columns": [
          { "field": "name", "label": "Name", "sortable": true },
          { "field": "status", "label": "Status", "sortable": true }
        ],
        "search_fields": ["name", "serial_number"],
        "default_sort": "name"
      }
    ],
    "extension_points": [
      { "id": "app.shell.sidebar.nav", "plugin_id": null, "type": "ui" }
    ]
  }
}
```

**Use case:** Auto-generating resource list pages, form builders, or admin dashboards from resource metadata.

---

### `GET /api/v1/ui/extension-points`

List all known extension points (35 built-in) plus any custom ones declared by plugins.

**Response:**
```json
{
  "data": {
    "known": [
      {
        "id": "app.shell.sidebar.nav",
        "description": "Sidebar navigation items",
        "type": "ui"
      },
      {
        "id": "asset.detail.tabs",
        "description": "Additional tabs on asset detail page",
        "type": "ui"
      }
    ],
    "declared": [
      { "id": "app.shell.sidebar.nav", "plugin_id": null, "type": "ui" }
    ]
  }
}
```

---

## Response Envelope

All plugin endpoints follow the standard SIP response format:

```json
{
  "data": <payload>
}
```

Errors use:
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message"
  }
}
```

## Permission Model

Plugin discovery endpoints are **public by design**. They expose metadata only — no operational data, no secrets, no internal configuration. The philosophy is:

- **Discovery is safe**: Knowing that a "Billing" plugin exists doesn't grant access to billing data
- **Navigation is declarative**: Menu items inform the UI what routes exist, but the API enforces access per-request via JWT
- **Resources are schemas**: Column definitions describe data shapes, not actual data

For endpoints that return operational data (assets, work orders, etc.), standard JWT-based auth with RBAC and tenant isolation applies.
