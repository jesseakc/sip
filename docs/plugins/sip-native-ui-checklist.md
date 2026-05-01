# SIP-Native UI Plugin Checklist

Use this checklist before shipping a SIP-native (`compatibility.level = "native"`) UI plugin.

## Manifest

- [ ] Plugin manifest has `plugin_type = "hybrid"` or `"ui"`
- [ ] `[ui]` section is present and `enabled = true`
- [ ] `[ui.compatibility]` block is present
- [ ] `[ui.compatibility].level` is set to `"native"`
- [ ] `uses_sip_components = true`
- [ ] `uses_theme_tokens = true`
- [ ] `allows_global_css = false`
- [ ] `requires_shell = true`
- [ ] `[ui.theme]` block declares `uses_design_tokens = true`
- [ ] `sip_ui_version` is set to a compatible version (e.g., `"^1.0.0"`)

## Design Tokens

- [ ] No hardcoded color values in CSS or inline styles
- [ ] All colors use `var(--sip-color-*)` tokens
- [ ] All spacing uses `var(--sip-spacing-*)` tokens
- [ ] All border radii use `var(--sip-radius-*)` tokens
- [ ] All fonts use `var(--sip-font-*)` tokens
- [ ] Box shadows use `var(--sip-shadow-*)` tokens
- [ ] Transitions use `var(--sip-transition-*)` tokens
- [ ] Layout dimensions reference `var(--sip-layout-*)` tokens where appropriate

## SIP Components

- [ ] Page-level wrapper uses `<SipPage>`
- [ ] Page headers use `<SipPageHeader>`
- [ ] Cards use `<SipCard>`
- [ ] Buttons use `<SipButton>`
- [ ] Form inputs use `<SipInput>` / `<SipSelect>`
- [ ] Badges use `<SipBadge>` / `<SipStatusBadge>`
- [ ] Alerts use `<SipAlert>`
- [ ] Empty states use `<SipEmptyState>`
- [ ] Extension point slots use `<SipExtensionSlot>`
- [ ] Content is wrapped in `<SipPluginBoundary pluginId="your-plugin">`
- [ ] Permission gates use `<SipPermissionGate>`

## CSS Scoping

- [ ] All custom CSS is scoped under `[data-sip-plugin="your-plugin"]`
- [ ] No global CSS is injected (no bare element selectors, no `*` rules, no `body`/`html` overrides)
- [ ] No `!important` declarations on layout properties
- [ ] No z-index values that could conflict with the shell
- [ ] CSS isolation is confirmed — plugin styles do not affect the sidebar, topbar, or other plugins

## Navigation

- [ ] Top-level `[[navigation]]` items are declared with `id`, `label`, `path`
- [ ] Each nav item has a valid `icon` from the Lucide set
- [ ] `order` values avoid collisions with core nav items (core items use 10–50 range)
- [ ] `permission` is set on items that require authorization
- [ ] Nested children are declared where sub-pages exist
- [ ] Nav paths start with `/` and are unique

## Routes

- [ ] `[[ui.routes]]` entry exists for each page
- [ ] Each route has a unique `id`
- [ ] `layout` is set to an appropriate value (`sip-page`, `sip-dashboard`, `sip-settings`, `embedded`, `standalone`)
- [ ] `title` is set for browser tab text
- [ ] `breadcrumb` is set for breadcrumb trail
- [ ] `required_permissions` is set on restricted routes

## Extension Points

- [ ] `[[ui.extension_points]]` entries declared where the plugin injects into shared slots
- [ ] Extension point IDs match known extension point IDs (from `/api/v1/ui/extension-points`)
- [ ] Extension point `type` is set to `"ui"` for UI contributions

## Theme Compatibility

- [ ] Theme is inherited from `sip-core` (`inherits = "sip-core"`)
- [ ] Dark mode renders correctly with all token references
- [ ] Density modes (compact/comfortable/spacious) do not break layout
- [ ] Accent color changes (if supported) are respected

## Dark Mode

- [ ] All surfaces use token colors (not hardcoded white/gray)
- [ ] Text remains readable against surface backgrounds in dark mode
- [ ] Semantic colors (danger, warning, success) remain distinguishable
- [ ] Box shadows appear correctly on dark surfaces
- [ ] Tested with Chrome DevTools `prefers-color-scheme: dark` emulation

## Responsive Layout

- [ ] Plugin renders inside `<SipPage>` which handles max-width and responsive padding
- [ ] Plugin works with sidebar expanded (default, 16rem width)
- [ ] Plugin works with sidebar collapsed (mobile or toggled)
- [ ] Plugin works at 320px viewport width
- [ ] Plugin works at 768px viewport width
- [ ] Plugin works at 1024px viewport width
- [ ] Plugin works at 1440px viewport width
- [ ] No horizontal overflow at any breakpoint
- [ ] Touch targets are at least 44x44px on mobile

## Error Handling

- [ ] `<SipPluginBoundary>` wraps plugin content with an appropriate `pluginId`
- [ ] Error boundary fallback is provided (or the default red banner is acceptable)
- [ ] Plugin gracefully handles missing data (empty arrays, null fields)
- [ ] Plugin shows `<SipEmptyState>` when a list has no items
- [ ] API errors are caught and shown via `<SipAlert>`

## Performance

- [ ] Plugin does not block the shell's initial render
- [ ] Heavy computation is deferred or chunked
- [ ] Plugin does not cause layout shift on mount

## Final Review

- [ ] Plugin appears in the sidebar under the correct section
- [ ] Clicking the nav item navigates to the correct route
- [ ] Breadcrumbs display correctly on all plugin pages
- [ ] Plugin does not break existing SIP pages
- [ ] Plugin can be disabled without breaking the shell
- [ ] First load renders without errors in the console
