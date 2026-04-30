# SIP Plugin Architecture

## Overview

SIP is built on a plugin engine from day one. **The official web frontend is itself a UI plugin** (`sip-core-ui`) — it is not a special case. Every UI surface, backend module, integration, and AI tool that ships with SIP or that the community builds runs through the same plugin framework.

This design means:
- Future mobile apps, CLIs, dashboards, and AI agents can discover SIP capabilities through the same API
- Community plugins work exactly like first-party ones
- The sidebar, dashboards, resource views, and settings panels are all extension points
- Navigation is declarative — no code changes needed to add a menu item

## Plugin Types

| Type | Purpose | Examples |
|------|---------|----------|
| **UI** (`plugin_type = "ui"`) | Contributes visual surfaces — navigation, routes, dashboards, forms, custom views, mobile app metadata | `sip-core-ui` (the default web frontend), a future mobile app, a customer portal |
| **Functional** (`plugin_type = "functional"`) | Contributes backend logic — API routes, jobs, automations, integrations, webhooks, MCP tools, AI tools, data model extensions | A billing module, an ERP connector, a custom report generator, an MLOps pipeline |
| **Hybrid** (`plugin_type = "hybrid"`) | Contributes both UI surfaces and backend logic | A full parts inventory module with its own pages and backend routes, a compliance dashboard with background audit jobs |

## How It Works

```
┌──────────────────────────────────────────────────────────────┐
│                    SIP API Server                             │
│                                                              │
│  ┌─────────────┐  ┌──────────────────────────────────────┐   │
│  │ Plugin      │  │ Plugin Registry (in-memory, RwLock)  │   │
│  │ Discovery    │  │                                      │   │
│  │ Endpoints   │  │  ┌──────────┐  ┌──────────┐         │   │
│  │ /api/v1/    │──▶│  │sip-core  │  │billing   │  ...    │   │
│  │   plugins   │  │  │  -ui     │  │  -plugin │         │   │
│  │   ui/nav    │  │  └──────────┘  └──────────┘         │   │
│  └─────────────┘  └──────────────────────────────────────┘   │
│                         ▲                                     │
│                         │ loads on startup                    │
│                   ┌─────┴─────┐                               │
│                   │ plugins/  │                               │
│                   │  ├─ sip-core-ui/plugin.toml               │
│                   │  └─ billing-plugin/plugin.toml            │
│                   └───────────┘                               │
└──────────────────────────────────────────────────────────────┘
         │
         │ GET /api/v1/ui/navigation
         ▼
┌──────────────────────┐      ┌──────────────────────────┐
│   SIP Frontend        │      │   Third-Party App         │
│   (Next.js shell.tsx) │      │   (mobile, CLI, TUI)      │
│                       │      │                           │
│   Fetches nav from    │      │   Fetches same API as     │
│   /ui/navigation      │      │   the first-party UI      │
│   Falls back to       │      │                           │
│   embedded defaults   │      │                           │
└──────────────────────┘      └──────────────────────────┘
```

## Lifecycle

1. **Startup**: SIP loads all `plugin.toml` files from the `plugins/` directory (or uses embedded defaults if the directory is missing)
2. **Validation**: Each manifest is validated — duplicate IDs, invalid paths, missing required fields
3. **Registration**: Valid manifests are inserted into the in-memory `PluginRegistry`
4. **Enable/Disable**: Plugins are enabled based on `enabled_by_default`, overridable via `SIP_ENABLED_PLUGINS` config
5. **Discovery**: Clients (frontends, mobile apps, AI agents) query the public plugin API endpoints
6. **Navigation**: The frontend fetches `/api/v1/ui/navigation` and renders the sidebar from the aggregated response

## Directory Structure

```
plugins/
├── sip-core-ui/            # First-party web UI (ships with SIP)
│   └── plugin.toml
├── your-custom-plugin/     # Your plugin
│   ├── plugin.toml         #   Manifest (required)
│   ├── src/                #   Frontend source (for UI plugins)
│   └── README.md           #   Plugin documentation
└── another-plugin/
    └── plugin.toml
```

## Key Principles

1. **API-first**: The frontend consumes SIP through public API contracts. Same endpoints for the official UI, mobile apps, CLIs, and AI agents.
2. **Plugin-first**: The official frontend is not special. It is `sip-core-ui` — one enabled plugin among potentially many.
3. **Declarative before arbitrary code**: Navigation, resources, extension points, and permissions are declared in TOML manifests. No arbitrary remote code execution.
4. **Security by default**: Plugin manifests are validated. Secrets live in environment variables, never in manifest files. Public API responses never expose API keys, passwords, or internal config.
5. **Progressive enhancement**: Start with a TOML manifest. Add routes. Add frontend components. Add backend logic. The framework scales with complexity.

## Next Steps

- [Manifest Reference](./manifest-reference.md) — every field in `plugin.toml`
- [API Reference](./api-reference.md) — plugin discovery REST endpoints
- [Extension Points](./extension-points.md) — 35 slots where plugins can inject UI
- [Examples](./examples.md) — walkthroughs for common plugin types
