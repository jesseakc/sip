# UI Extension Points

Extension points are named slots where plugins can inject UI elements. SIP ships with **35 canonical extension points** covering the shell, dashboard, entity detail pages, settings, and AI tools.

Plugins declare which extension points they contribute to in their manifest:

```toml
[[extension_points]]
id = "asset.detail.tabs"
type = "ui"
metadata = { tab_id = "warranty", label = "Warranty Info" }
```

The full catalog is queryable at runtime:
```bash
curl http://localhost:8000/api/v1/ui/extension-points
```

---

## Shell

| Extension Point ID | Description |
|-------------------|-------------|
| `app.shell.sidebar.nav` | Sidebar navigation items. The primary way UI plugins add menu entries. |
| `app.shell.topbar.actions` | Top bar action buttons (notifications, quick actions, user menu items). |

---

## Dashboard

| Extension Point ID | Description |
|-------------------|-------------|
| `dashboard.cards` | Summary card widgets (KPIs, counts, status summaries). |
| `dashboard.widgets` | Detailed dashboard widgets (charts, tables, activity feeds). |

---

## Assets

| Extension Point ID | Description |
|-------------------|-------------|
| `asset.list.columns` | Additional columns in the asset list/table view. |
| `asset.detail.tabs` | Additional tabs on the asset detail page. |
| `asset.detail.sidebar` | Sidebar panels on the asset detail page (metadata, linked items). |
| `asset.detail.actions` | Action buttons on the asset detail page (custom workflows). |
| `asset.create.form.sections` | Additional form sections in the asset creation flow. |

---

## Work Orders

| Extension Point ID | Description |
|-------------------|-------------|
| `work_order.list.columns` | Additional columns in the work order list/table view. |
| `work_order.detail.tabs` | Additional tabs on the work order detail page. |
| `work_order.detail.sidebar` | Sidebar panels (related assets, history, documents). |
| `work_order.detail.actions` | Action buttons (custom transitions, external actions). |
| `work_order.create.form.sections` | Additional form sections in work order creation. |

---

## Schedules

| Extension Point ID | Description |
|-------------------|-------------|
| `schedule.list.columns` | Additional columns in the schedule list view. |
| `schedule.detail.actions` | Action buttons on the schedule detail page. |

---

## Inspections

| Extension Point ID | Description |
|-------------------|-------------|
| `inspection.list.columns` | Additional columns in the inspection list view. |
| `inspection.detail.panels` | Additional panels on the inspection detail page. |
| `inspection.detail.actions` | Action buttons on the inspection detail page. |
| `inspection_checklist.items` | Additional checklist item types (beyond PASS_FAIL, NUMERIC, TEXT, PHOTO). |

---

## Parts

| Extension Point ID | Description |
|-------------------|-------------|
| `parts.list.columns` | Additional columns in the parts inventory list. |
| `parts.detail.actions` | Action buttons on the part detail page. |
| `parts.create.form.sections` | Additional form sections in part creation. |

---

## Settings

| Extension Point ID | Description |
|-------------------|-------------|
| `settings.sections` | Additional sections on the settings page (plugin config, integrations). |
| `settings.general.form` | Additional fields in the general settings form. |

---

## Command Palette & AI

| Extension Point ID | Description |
|-------------------|-------------|
| `command_palette.actions` | Quick actions available in the command palette (Cmd+K / Ctrl+K). |
| `ai_chat.tools` | AI agent tools available in the chat interface. |

---

## Documents

| Extension Point ID | Description |
|-------------------|-------------|
| `document.list.columns` | Additional columns in the document list view. |
| `document.detail.actions` | Action buttons on the document detail page. |

---

## Locations

| Extension Point ID | Description |
|-------------------|-------------|
| `location.list.columns` | Additional columns in the location list view. |
| `location.detail.actions` | Action buttons on the location detail page. |

---

## Teams & Users

| Extension Point ID | Description |
|-------------------|-------------|
| `team.list.columns` | Additional columns in the team list view. |
| `team.detail.actions` | Action buttons on the team detail page. |
| `user.list.columns` | Additional columns in the user list view. |
| `user.detail.actions` | Action buttons on the user detail page. |

---

## Using Extension Points

### In a UI Plugin Manifest

```toml
[[extension_points]]
id = "asset.detail.tabs"
type = "ui"
metadata = { tab_id = "compliance", label = "Compliance", order = 50 }

[[extension_points]]
id = "app.shell.sidebar.nav"
type = "ui"
```

### In Code (Future API)

The extension point system is currently **declarative only** — plugins declare what they contribute, and the SIP frontend renders contributions at the declared slots. Future versions will add:

- A `GET /api/v1/ui/extension-points/:id/contributions` endpoint returning what each slot contains
- Frontend components that dynamically render plugin contributions into extension slots
- An extension SDK for plugins to register React components at specific slots

### Adding New Extension Points

Plugins can declare custom extension point IDs:

```toml
[[extension_points]]
id = "my_plugin.custom_panel"
type = "ui"
```

Custom extension points appear in the `declared` array of the `/api/v1/ui/extension-points` response. Other plugins can then target them by setting `plugin_id` to the declaring plugin's ID.
