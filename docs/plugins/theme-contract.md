# SIP Theme Token Contract — v1.0.0

The SIP theme is a stable system of **CSS custom properties** exposed on `:root`. Every UI plugin — whether native, compatible, or standalone — can reference these tokens. Native plugins **must** use them instead of hardcoded values.

Tokens automatically respond to `prefers-color-scheme: dark` and `.density-compact` / `.density-spacious` class names.

---

## How to Reference Tokens

### In CSS

```css
.my-surface {
  background: var(--sip-color-surface);
  border: 1px solid var(--sip-color-border);
  border-radius: var(--sip-radius-lg);
  padding: var(--sip-spacing-md);
}
```

### In Inline Styles (React)

```tsx
<div style={{
  color: 'var(--sip-color-text)',
  fontSize: 'var(--sip-font-size-base)',
  padding: 'var(--sip-spacing-lg)',
}}>
  Hello
</div>
```

### Scoping Plugin CSS

All plugin styles must be scoped under `[data-sip-plugin]` to avoid leaking into the SIP shell:

```css
[data-sip-plugin="my-plugin"] .card {
  background: var(--sip-color-surface);
  border-radius: var(--sip-radius-lg);
  box-shadow: var(--sip-shadow-card);
}
```

---

## Color Tokens

### Surface Colors

| Token | Light Value | Purpose |
|-------|-------------|---------|
| `--sip-color-background` | `#f9fafb` | Page background (behind cards) |
| `--sip-color-surface` | `#ffffff` | Card, modal, dropdown surface |
| `--sip-color-surface-muted` | `#f3f4f6` | Secondary surface (tabs, footers, inline code) |
| `--sip-color-surface-hover` | `#f9fafb` | Row/button hover state |

### Border Colors

| Token | Light Value | Purpose |
|-------|-------------|---------|
| `--sip-color-border` | `#e5e7eb` | Default border |
| `--sip-color-border-light` | `#f3f4f6` | Subtle separator border |

### Text Colors

| Token | Light Value | Purpose |
|-------|-------------|---------|
| `--sip-color-text` | `#111827` | Primary body text |
| `--sip-color-text-muted` | `#6b7280` | Secondary text (descriptions, placeholders, labels) |
| `--sip-color-text-inverse` | `#ffffff` | Text on dark/inverted backgrounds |

### Semantic Colors

| Token | Light Value | Purpose |
|-------|-------------|---------|
| `--sip-color-primary` | `#2563eb` | Primary action color (buttons, links, focus rings) |
| `--sip-color-primary-hover` | `#1d4ed8` | Primary hover state |
| `--sip-color-primary-light` | `#eff6ff` | Primary tint background (selected row, info alerts) |
| `--sip-color-primary-text` | `#1e40af` | Primary text (links, emphasis on light backgrounds) |
| `--sip-color-danger` | `#dc2626` | Error, destructive action |
| `--sip-color-danger-hover` | `#b91c1c` | Danger hover state |
| `--sip-color-danger-light` | `#fef2f2` | Danger tint background (error alerts) |
| `--sip-color-warning` | `#d97706` | Warning, caution |
| `--sip-color-warning-light` | `#fffbeb` | Warning tint background |
| `--sip-color-success` | `#16a34a` | Success, confirmation |
| `--sip-color-success-light` | `#f0fdf4` | Success tint background |
| `--sip-color-info` | `#0891b2` | Information, neutral alert |
| `--sip-color-info-light` | `#ecfeff` | Info tint background |

### Dark Mode

All color tokens swap to dark-appropriate values when `prefers-color-scheme: dark` is active:

| Token | Dark Value |
|-------|-------------|
| `--sip-color-background` | `#111827` |
| `--sip-color-surface` | `#1f2937` |
| `--sip-color-surface-muted` | `#374151` |
| `--sip-color-surface-hover` | `#2d3748` |
| `--sip-color-border` | `#4b5563` |
| `--sip-color-border-light` | `#374151` |
| `--sip-color-text` | `#f9fafb` |
| `--sip-color-text-muted` | `#9ca3af` |
| `--sip-color-text-inverse` | `#111827` |
| `--sip-color-primary` | `#3b82f6` |
| `--sip-color-primary-hover` | `#60a5fa` |
| `--sip-color-primary-light` | `#1e3a5f` |
| `--sip-color-primary-text` | `#93c5fd` |
| `--sip-shadow-card` | `0 1px 3px rgba(0,0,0,0.3)` |
| `--sip-shadow-popover` | `0 4px 6px rgba(0,0,0,0.4)` |

Danger, warning, success, and info colors remain unchanged in dark mode (they have sufficient contrast on dark surfaces).

---

## Radius Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `--sip-radius-sm` | `0.25rem` | Small elements: badges, tags, small buttons |
| `--sip-radius-md` | `0.375rem` | Inputs, select dropdowns, medium buttons |
| `--sip-radius-lg` | `0.5rem` | Cards, modals, panels |
| `--sip-radius-xl` | `0.75rem` | Large containers, page sections |
| `--sip-radius-full` | `9999px` | Pill badges, circular avatars |

```css
.sip-card {
  border-radius: var(--sip-radius-lg);
}
```

---

## Spacing Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `--sip-spacing-xs` | `0.25rem` | Tight gaps: icon+label, inline actions |
| `--sip-spacing-sm` | `0.5rem` | Compact gaps: form field spacing, button groups |
| `--sip-spacing-md` | `1rem` | Default gap: card padding, section spacing |
| `--sip-spacing-lg` | `1.5rem` | Generous gap: page sections, card grids |
| `--sip-spacing-xl` | `2rem` | Wide gap: major page sections |
| `--sip-spacing-2xl` | `3rem` | Separator: hero sections, page headers |

```css
.page-section {
  padding: var(--sip-spacing-lg);
  margin-bottom: var(--sip-spacing-xl);
}
```

---

## Typography Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `--sip-font-sans` | `ui-sans-serif, system-ui, -apple-system, sans-serif` | Body text, headings |
| `--sip-font-mono` | `ui-monospace, SFMono-Regular, Menlo, monospace` | Code, data values |
| `--sip-font-size-xs` | `0.75rem` | Captions, fine print |
| `--sip-font-size-sm` | `0.875rem` | Secondary text, form labels |
| `--sip-font-size-base` | `1rem` | Body text |
| `--sip-font-size-lg` | `1.125rem` | Card titles, emphasized text |
| `--sip-font-size-xl` | `1.25rem` | Section headings |
| `--sip-font-size-2xl` | `1.5rem` | Page titles |

```css
.page-title {
  font-family: var(--sip-font-sans);
  font-size: var(--sip-font-size-2xl);
  font-weight: 700;
}
```

---

## Shadow Tokens

| Token | Value (Light) | Usage |
|-------|---------------|-------|
| `--sip-shadow-card` | `0 1px 3px rgba(0,0,0,0.1), 0 1px 2px rgba(0,0,0,0.06)` | Cards, panels |
| `--sip-shadow-popover` | `0 4px 6px rgba(0,0,0,0.07), 0 10px 15px rgba(0,0,0,0.1)` | Dropdowns, modals, popovers |

Both tokens fade in dark mode (card: `rgba(0,0,0,0.3)`, popover: `rgba(0,0,0,0.4)`).

---

## Layout Tokens

| Token | Value | Purpose |
|-------|-------|---------|
| `--sip-layout-sidebar-width` | `16rem` | Width of the collapsed/expanded sidebar |
| `--sip-layout-topbar-height` | `4rem` | Height of the top navigation bar |
| `--sip-layout-content-width` | `72rem` | Max-width of page content area |

```css
.sidebar-spacer {
  width: var(--sip-layout-sidebar-width);
}

.content-area {
  max-width: var(--sip-layout-content-width);
  margin: 0 auto;
}
```

---

## Transition Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `--sip-transition-fast` | `150ms ease` | Hover states, focus rings |
| `--sip-transition-normal` | `200ms ease` | Modal open/close, sidebar toggle |

```css
.button {
  transition: background-color var(--sip-transition-fast);
}

.button:hover {
  background: var(--sip-color-primary-hover);
}
```

---

## Density Modes

SIP supports three density modes, controlled by a class on the `<body>` or `<html>` element. These re-define spacing tokens:

### Compact (`.density-compact`)

| Token | Default | Compact |
|-------|---------|---------|
| `--sip-spacing-md` | `1rem` | `0.75rem` |
| `--sip-spacing-lg` | `1.5rem` | `1rem` |
| `--sip-spacing-xl` | `2rem` | `1.5rem` |

### Comfortable (default, no class)

Standard spacing. No override needed.

### Spacious (`.density-spacious`)

| Token | Default | Spacious |
|-------|---------|----------|
| `--sip-spacing-md` | `1rem` | `1.25rem` |
| `--sip-spacing-lg` | `1.5rem` | `2rem` |
| `--sip-spacing-xl` | `2rem` | `2.5rem` |

**Note:** Density only affects spacing tokens. Typography, radius, and color tokens are unaffected.

---

## Dark Mode Support

SIP dark mode is driven by `prefers-color-scheme: dark`. All color and shadow tokens swap automatically. Plugins do not need to add their own dark mode media queries — they simply reference the tokens, which already handle the swap.

```css
/* Correct — token handles both modes */
.my-block {
  background: var(--sip-color-surface);
  color: var(--sip-color-text);
}

/* Incorrect — hardcoded, won't respond to dark mode */
.my-block {
  background: #ffffff;
  color: #111827;
}
```

If a plugin needs to conditionally render content based on the current theme, it can check `prefers-color-scheme` via JavaScript:

```tsx
const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
```

---

## The `/api/v1/ui/theme` Endpoint

Plugins can fetch theme metadata at runtime:

```
GET /api/v1/ui/theme
```

**Response shape:**

```json
{
  "data": {
    "theme": "sip-core",
    "mode": "system",
    "density": "comfortable",
    "supportsDarkMode": true,
    "supportsAccentColor": true,
    "tokensVersion": "1.0.0",
    "tokens": {
      "colors": {
        "background": "#f9fafb",
        "surface": "#ffffff",
        "surfaceMuted": "#f3f4f6",
        "border": "#e5e7eb",
        "text": "#111827",
        "textMuted": "#6b7280",
        "primary": "#2563eb",
        "primaryHover": "#1d4ed8",
        "danger": "#dc2626",
        "warning": "#d97706",
        "success": "#16a34a"
      },
      "radius": {
        "sm": "0.25rem",
        "md": "0.375rem",
        "lg": "0.5rem",
        "xl": "0.75rem"
      },
      "spacing": {
        "xs": "0.25rem",
        "sm": "0.5rem",
        "md": "1rem",
        "lg": "1.5rem",
        "xl": "2rem"
      }
    }
  }
}
```

---

## CSS Scoping Under `[data-sip-plugin]`

The `<SipPluginBoundary>` component wraps each plugin in:

```html
<div data-sip-plugin="my-plugin">
  <!-- plugin content -->
</div>
```

The global stylesheet sets `[data-sip-plugin] { isolation: isolate; }` to create a CSS stacking context boundary. This prevents `z-index` conflicts between plugins and the shell.

All plugin custom CSS must be scoped:

```css
/* Correct — scoped */
[data-sip-plugin="my-plugin"] .data-table {
  border-collapse: collapse;
}

/* Incorrect — global */
.data-table {
  border-collapse: collapse;
}
```

---

## Utility Token Classes

SIP provides a few convenience classes in addition to the custom properties:

| Class | Effect |
|-------|--------|
| `.sip-surface` | `background: var(--sip-color-surface)`, border, rounded corners |
| `.sip-surface-muted` | `background: var(--sip-color-surface-muted)` |
| `.sip-border` | `border: 1px solid var(--sip-color-border)` |

---

## Design Token Versioning

Token names are versioned (`tokensVersion: "1.0.0"`). The SIP team commits to backward compatibility: new tokens may be added, but existing tokens will not be removed or renamed within the same major version. Plugins can declare a minimum token version via `sip_ui_version` in their compatibility block.

---

## See Also

- [UI Plugins](./ui-plugins.md) — how to build a SIP-native UI plugin
- [Navigation](./navigation.md) — sidebar and routing
- [SIP-Native UI Checklist](./sip-native-ui-checklist.md) — pre-ship review
