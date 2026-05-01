# SIP-Native UI Plugins

A **SIP-native UI plugin** is a plugin that renders its interface inside the SIP shell using SIP design tokens, SIP UI components, and the SIP extension system. It looks and behaves like a first-party page — not a third-party embed.

When a user navigates to a plugin route, the plugin's content renders within the SIP shell layout: sidebar, topbar, breadcrumbs, and theme all remain intact. The plugin contributes pages, cards, actions, and settings panels through the same extension mechanism as `sip-core-ui`.

---

## Compatibility Levels

Every UI plugin declares a `compatibility.level` in its `[ui.compatibility]` block. The level tells SIP how well the plugin integrates with the shell.

### `native`

The plugin uses SIP design tokens (`var(--sip-color-*)`), SIP UI components (`SipPage`, `SipCard`, etc.), and renders entirely within the SIP shell. It does not inject global CSS. The result is indistinguishable from a first-party page.

| Flag | Value |
|------|-------|
| `requires_shell` | `true` |
| `uses_sip_components` | `true` |
| `uses_theme_tokens` | `true` |
| `allows_global_css` | `false` |

### `compatible`

The plugin runs inside the SIP shell and may use some SIP tokens, but also brings its own component library or CSS framework. It does not inject global CSS that could break the shell.

| Flag | Value |
|------|-------|
| `requires_shell` | `true` |
| `uses_sip_components` | `false` |
| `uses_theme_tokens` | `true` |
| `allows_global_css` | `false` |

### `standalone`

The plugin is a completely independent application. It may be mounted inside an iframe or opened in a new tab. SIP provides navigation integration but delegates all rendering to the plugin.

| Flag | Value |
|------|-------|
| `requires_shell` | `false` |
| `uses_sip_components` | `false` |
| `uses_theme_tokens` | `false` |
| `allows_global_css` | `true` |

### Declaration

```toml
[ui.compatibility]
level = "native"
sip_ui_version = "^1.0.0"
requires_shell = true
uses_sip_components = true
uses_theme_tokens = true
allows_global_css = false
```

---

## Declaring Navigation

Plugins contribute navigation items at the top level of the manifest. Each item represents a sidebar link. The registry aggregates items from all enabled UI plugins, sorts them by `order`, and serves them at `GET /api/v1/ui/navigation`.

```toml
[[navigation]]
id = "my-plugin"
label = "My Plugin"
path = "/my-plugin"
icon = "puzzle"
order = 90
permission = "my-plugin:read"

[[navigation.children]]
id = "my-plugin-dashboard"
label = "Dashboard"
path = "/my-plugin"
permission = "my-plugin:read"

[[navigation.children]]
id = "my-plugin-settings"
label = "Settings"
path = "/my-plugin/settings"
permission = "my-plugin:manage"
```

**Field reference:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique ID within the plugin |
| `label` | string | Yes | Display text in the sidebar |
| `path` | string | Yes | URL path, must start with `/` |
| `icon` | string | No | Lucide icon name |
| `permission` | string | No | Permission gate (format: `resource:action`) |
| `order` | int | No | Sort order (lower = first). Default: 999 |
| `feature_flag` | string | No | Future feature-flag gate |
| `children` | nav[] | No | Nested sub-navigation items |

**Section grouping** is handled by the frontend. The sidebar groups items by conceptual sections: **Core**, **Service**, **Intelligence**, **Admin**, **Developer**, **Settings**. The frontend assigns each nav item to a section based on configuration or plugin category. See [Navigation](./navigation.md) for details.

---

## Declaring Routes

Routes are declared inside `[ui]`. Each route maps a URL path to a plugin component with a layout and permission guard.

```toml
[[ui.routes]]
id = "my-plugin-home"
path = "/my-plugin"
component = "MyPluginDashboard"
layout = "sip-page"
title = "My Plugin"
breadcrumb = "Dashboard"
required_permissions = ["my-plugin:read"]
```

**Field reference:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique route ID |
| `path` | string | Yes | URL path the route handles |
| `component` | string | No | Component name to render |
| `layout` | string | No | Layout type (see below) |
| `title` | string | No | Page title (browser tab) |
| `breadcrumb` | string | No | Breadcrumb label |
| `required_permissions` | string[] | No | Permissions required to access the route |

**Valid layout types:**

| Layout | Behavior |
|--------|----------|
| `sip-page` | Content area with standard padding/max-width. Use for normal detail and list pages. |
| `sip-dashboard` | Dashboard grid layout. Use for overview pages with cards and widgets. |
| `sip-settings` | Settings form layout. Use for configuration and admin pages. |
| `embedded` | No chrome — renders inline inside a parent slot. Use for extension point contributions. |
| `standalone` | Full-page, no shell. The plugin owns the entire viewport. |

---

## Declaring Actions

Actions are invocable commands exposed to the SIP shell. They can appear in the command palette, dashboard toolbar, or context menus.

```toml
[[ui.actions]]
id = "my-plugin.quick-report"
label = "Generate Report"
icon = "bar-chart"
route = "/my-plugin/report"
placement = ["command_palette", "toolbar"]
required_permissions = ["my-plugin:export"]
```

**Field reference:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique action ID |
| `label` | string | Yes | Display label |
| `icon` | string | No | Lucide icon name |
| `route` | string | No | Path to navigate to on invocation |
| `placement` | string[] | Yes | Where the action appears |
| `required_permissions` | string[] | No | Permissions required |

**Valid placement values:**

| Placement | Appears in |
|-----------|------------|
| `command_palette` | Cmd+K / Ctrl+K command palette |
| `toolbar` | Page-level toolbar (dashboard, lists) |
| `context-menu` | Right-click context menu |
| `dashboard` | Dashboard action bar |

---

## Using SIP Design Tokens

SIP exposes its entire visual identity as CSS custom properties. Plugins must reference these tokens instead of hardcoding colors, spacing, or typography values.

**In CSS:**
```css
.my-card {
  background: var(--sip-color-surface);
  border: 1px solid var(--sip-color-border);
  border-radius: var(--sip-radius-lg);
  padding: var(--sip-spacing-lg);
  color: var(--sip-color-text);
}
```

**In inline styles (React):**
```tsx
<div style={{
  color: 'var(--sip-color-text)',
  background: 'var(--sip-color-surface)',
  padding: 'var(--sip-spacing-md)',
}}>
  Content
</div>
```

See [Theme Contract](./theme-contract.md) for every available token.

---

## Using SIP UI Components

SIP ships a library of React components designed for plugin use. Import them from `@/components/sip`:

| Component | Use |
|-----------|-----|
| `SipPage` | Page-level wrapper with max-width and padding |
| `SipPageHeader` | Title, description, breadcrumbs, and action buttons |
| `SipCard` | Contained card surface with border and shadow |
| `SipButton` | Themed button with variant/size support |
| `SipInput` | Themed text input |
| `SipSelect` | Themed dropdown select |
| `SipBadge` | Label badge for status/metadata |
| `SipStatusBadge` | Status-indicator badge with color mapping |
| `SipAlert` | Alert/notification banner |
| `SipEmptyState` | Placeholder for empty lists/results |
| `SipPermissionGate` | Conditionally render children based on user permissions |
| `SipExtensionSlot` | Render extension point contributions |
| `SipPluginBoundary` | Error boundary that isolates plugin failures |

**Example — a plugin dashboard page:**
```tsx
import { SipPage, SipPageHeader, SipCard, SipButton } from '@/components/sip';

export function MyPluginDashboard() {
  return (
    <SipPage>
      <SipPageHeader
        title="My Plugin"
        description="Manage your resources"
        actions={<SipButton variant="primary">New Item</SipButton>}
      />
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
        gap: 'var(--sip-spacing-lg)',
      }}>
        <SipCard>
          <h3 style={{ fontSize: 'var(--sip-font-size-lg)' }}>Total Items</h3>
          <p style={{ fontSize: 'var(--sip-font-size-2xl)', fontWeight: 700 }}>42</p>
        </SipCard>
      </div>
    </SipPage>
  );
}
```

---

## Plugin Permissions and Visibility

Permission checks happen at two levels:

1. **Navigation**: Nav items with a `permission` field are hidden from users who lack the permission.
2. **Routes**: Routes with `required_permissions` are rejected (HTTP 403) for unauthorized users.

The SIP frontend wraps plugin content in `<SipPluginBoundary>` for error isolation and can optionally gate entire plugin sections with `<SipPermissionGate>`.

```tsx
<SipPluginBoundary pluginId="my-plugin">
  <SipPermissionGate permission="my-plugin:read">
    <MyPluginDashboard />
  </SipPermissionGate>
</SipPluginBoundary>
```

---

## Avoiding Global CSS Conflicts

Plugins at the `native` and `compatible` levels must not inject global CSS. All plugin styles must be scoped under the `[data-sip-plugin]` attribute:

```css
[data-sip-plugin="my-plugin"] .my-custom-class {
  /* styles are isolated to this plugin */
}
```

The `<SipPluginBoundary>` component automatically wraps plugin content in `<div data-sip-plugin={pluginId}>`, providing a scoping root. The global `:root` sets `[data-sip-plugin] { isolation: isolate; }` to create a stacking context boundary.

**Do not:**
- Override SIP shell classes (`.sidebar`, `.topbar`, etc.)
- Set global `body`, `*`, or `html` styles
- Use `!important` on layout properties

**Do:**
- Scope all custom CSS under `[data-sip-plugin="your-id"]`
- Use CSS modules or CSS-in-JS with the `data-sip-plugin` attribute
- Reference SIP tokens for colors, spacing, typography

---

## Testing Dark Mode and Responsive Layouts

### Dark Mode

SIP supports dark mode via `prefers-color-scheme: dark`. All design tokens automatically swap. To test:

1. In Chrome DevTools: **Rendering** tab > **Emulate CSS media feature prefers-color-scheme** > `dark`
2. Or toggle your OS dark mode setting

Verify that all plugin surfaces use token-based colors that respond to the media query. No color should remain hardcoded.

### Responsive Layout

The SIP shell is responsive. The sidebar collapses on narrow viewports. Plugins should:

- Use `SipPage` which handles max-width and responsive padding
- Avoid fixed pixel widths — use tokens (`var(--sip-layout-sidebar-width)`) and relative units
- Test at 320px, 768px, 1024px, and 1440px viewport widths

---

## Full Example Manifest

See [`docs/plugins/examples/sip-native-ui-plugin.toml`](./examples/sip-native-ui-plugin.toml) for a complete reference manifest demonstrating every UI plugin field.

---

## Next Steps

- [Theme Contract](./theme-contract.md) — every design token available
- [Navigation](./navigation.md) — sidebar rendering and section grouping
- [Extension Points](./extension-points.md) — 35 slots for UI injection
- [Manifest Reference](./manifest-reference.md) — complete TOML schema
- [SIP-Native UI Checklist](./sip-native-ui-checklist.md) — pre-ship review list
