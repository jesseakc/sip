# SIP Navigation System

The SIP sidebar navigation is built from plugin manifests — no frontend code changes are needed to add, reorder, or remove sidebar items.

---

## How Navigation Is Declared

Every UI or hybrid plugin declares navigation items at the top level of its `plugin.toml`:

```toml
[[navigation]]
id = "my-plugin"
label = "My Plugin"
path = "/my-plugin"
icon = "puzzle"
permission = "my-plugin:read"
order = 90

[[navigation.children]]
id = "my-plugin-home"
label = "Home"
path = "/my-plugin"
permission = "my-plugin:read"

[[navigation.children]]
id = "my-plugin-settings"
label = "Settings"
path = "/my-plugin/settings"
permission = "my-plugin:manage"
```

### Field Reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | **Yes** | Unique ID within the plugin. Must be stable across versions. |
| `label` | string | **Yes** | Text displayed in the sidebar. Keep short. |
| `path` | string | **Yes** | URL path. Must start with `/`. Must be unique across all sidebar links. |
| `icon` | string | No | Icon name from Lucide. See [Icon Names](#icon-names). |
| `permission` | string | No | Permission gate. Items are hidden if the user lacks this permission. Format: `resource:action`. |
| `order` | int | No | Display order within the section. Lower numbers appear first. Default: `999`. |
| `feature_flag` | string | No | Future gate: feature flag name required to show this item. |
| `children` | nav[] | No | Sub-items displayed as a collapsible group. Each child has the same fields (minus `children`). |

---

## Navigation Sections

The frontend groups sidebar items into conceptual sections. Each nav item is assigned to a section by the UI shell based on the plugin's declared metadata.

| Section ID | Label | Typical Contents |
|------------|-------|------------------|
| `core` | Core | Dashboard, assets, work orders |
| `service` | Service | Schedules, inspections, parts |
| `intelligence` | Intelligence | AI Chat, reports, analytics |
| `admin` | Admin | Locations, teams, users, documents |
| `developer` | Developer | API keys, webhooks, logs |
| `settings` | Settings | General, integrations, plugin config |

Sections are rendered as collapsible groups in the sidebar with a label and icon. Items within a section are sorted by their `order` field (ascending).

The section-to-item mapping is maintained by the frontend. Plugins do not need to declare a section explicitly — the UI shell assigns items based on the plugin's `category` and the nav item's `path`.

---

## Item Ordering

Within each section, items are ordered by:

1. **`order` field** (ascending — lower numbers first)
2. **Plugin registration order** (stable tiebreaker for equal `order` values)

```toml
[[navigation]]
id = "assets"
label = "Assets"
path = "/assets"
order = 20   # Renders before "Work Orders" (order=30)

[[navigation]]
id = "work-orders"
label = "Work Orders"
path = "/work-orders"
order = 30
```

Items without an explicit `order` default to `999`, which pushes them to the bottom of their section.

---

## Nested Children

Parent items with a `children` array render as collapsible sidebar groups. The parent itself is clickable (navigating to its `path`). Children are only visible when the group is expanded.

```toml
[[navigation]]
id = "assets"
label = "Assets"
path = "/assets"
icon = "packages"
order = 20

[[navigation.children]]
id = "assets-list"
label = "All Assets"
path = "/assets"

[[navigation.children]]
id = "assets-import"
label = "Import"
path = "/assets/import"
permission = "assets:import"
```

The frontend highlights the active child and auto-expands the parent group when the user is on a child route.

**Constraints:**
- Children can only be one level deep (no grandchildren)
- Children use the parent's `icon` by default
- Children can override `permission` to gate specific sub-pages

---

## Permission-Gated Items

Navigation items and their children respect permission gates. If a user lacks the required permission, the item is silently hidden from the sidebar.

**Permission format:** `resource:action`

Examples:
- `assets:read` — user can view assets
- `work_orders:write` — user can create/edit work orders
- `admin:access` — user can access admin pages
- `my-plugin:manage` — user can manage plugin settings

If a user has none of the permissions required by a plugin's navigation items, the plugin's entire section is hidden.

Permissions are **informational in the manifest** — actual enforcement happens at the API layer via JWT claims and RBAC.

---

## Icon Names

Icons come from [Lucide](https://lucide.dev). Common choices:

| Icon Name | Visual | Use Case |
|-----------|--------|----------|
| `layout-dashboard` | Grid | Dashboard |
| `packages` | Box | Assets, inventory |
| `clipboard-list` | Clipboard | Work orders, tasks |
| `calendar` | Calendar | Schedules |
| `clipboard-check` | Checked clipboard | Inspections |
| `wrench` | Wrench | Parts, maintenance |
| `message-square` | Chat bubble | AI Chat |
| `map-pin` | Map pin | Locations |
| `users` | People | Teams |
| `user` | Person | User, profile |
| `file-text` | Document | Documents |
| `settings` | Gear | Settings |
| `bell` | Bell | Notifications |
| `bar-chart` | Bar chart | Reports |
| `zap` | Lightning | Quick actions |
| `puzzle` | Puzzle piece | Plugins, integrations |
| `git-branch` | Branch | Migration Studio, imports |
| `shield` | Shield | Security, compliance |
| `search` | Magnifier | Search |

See [lucide.dev/icons](https://lucide.dev/icons) for the full catalog (1,400+ icons).

---

## How the Frontend Fetches Navigation

The frontend calls `GET /api/v1/ui/navigation` on app load:

```
GET /api/v1/ui/navigation
```

**Response:**
```json
{
  "data": [
    {
      "id": "dashboard",
      "label": "Dashboard",
      "path": "/",
      "icon": "layout-dashboard",
      "order": 10,
      "children": []
    },
    {
      "id": "assets",
      "label": "Assets",
      "path": "/assets",
      "icon": "packages",
      "permission": "assets:read",
      "order": 20,
      "children": []
    }
  ]
}
```

The response is a flat array of top-level navigation items, each with their `children` array populated. The array is **already sorted** — the API handles ordering.

---

## Fallback Behavior

When the API is unavailable (server not running, plugin directory missing), the frontend falls back to **embedded defaults**. These are compiled into the frontend binary and include the core navigation items (Dashboard, Assets, Work Orders, etc.).

This ensures the app is always usable even if the plugin registry is not reachable.

---

## Sidebar Rendering

The frontend sidebar component:

1. Fetches `/api/v1/ui/navigation`
2. Sorts items by `order`
3. Groups items into sections (Core, Service, Intelligence, Admin, Developer, Settings)
4. Renders each section as a collapsible group with a label
5. Renders each nav item as a link with an icon and label
6. Gated items (where the user lacks the permission) are filtered out

---

## Active Route Highlighting

The frontend determines which nav item is "active" based on the current URL path. Active items receive a highlighted background and primary text color.

**Matching logic:**
- Exact path match: `/assets` highlights the "Assets" item
- Prefix match: `/assets/123` highlights "Assets" (since `/assets` is a prefix)
- Child match: The parent group auto-expands and the specific child is highlighted

The active state uses the `--sip-color-primary-light` background and `--sip-color-primary` text color.

---

## Cross-Plugin Navigation

Plugins can link to each other's routes. A plugin's navigation item can have a `path` that points to another plugin's route. The sidebar treats all nav items equally — there is no plugin ownership of paths.

```toml
# Plugin: my-reporting-plugin
[[navigation]]
id = "reporting-assets"
label = "Asset Reports"
path = "/assets/reports"   # Links into the assets area
order = 25
```

---

## See Also

- [UI Plugins](./ui-plugins.md) — full guide to building UI plugins
- [Manifest Reference](./manifest-reference.md) — complete TOML schema
- [Extension Points](./extension-points.md) — `app.shell.sidebar.nav` extension point
