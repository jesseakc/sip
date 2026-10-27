# Platform reference

[Documentation home](README.md)

Detailed material moved from the project README. This is a capability and data
model reference, not a release certification. The [architecture guide](architecture.md)
records which runtime paths are connected and which remain unfinished.

- [Architecture and plugins](#architecture-and-plugins)
- [Capabilities](#capabilities)
- [API examples](#api-examples)
- [Project structure](#project-structure)
- [Service domain model](#service-domain-model)
- [Database](#database)

## Architecture and plugins

SIP uses a Next.js frontend, a Rust API, and PostgreSQL. Compose also provisions
Redis and MinIO. For diagrams, layer boundaries, and what is connected today,
see the [architecture guide](architecture.md).

### Plugin architecture

The first-party frontend has a `sip-core-ui` manifest. SIP also supplies default navigation when the plugin system is disabled. Plugins are discovered through the registry when enabled.

Plugins are intended to use SIP's APIs, permissions, and service workflows. A manifest declaration alone does not implement a capability.

SIP supports three plugin types:

- **UI plugins**: contribute navigation, routes, dashboards, forms, and extension points
- **Functional plugins**: contribute backend logic, API routes, jobs, automations, integrations, AI tools, and MCP tools
- **Hybrid plugins**: contribute both UI and backend capabilities

UI plugins declare a **compatibility level**: `native` (uses SIP shell, tokens, and components), `compatible` (uses SIP navigation but may have custom styling), or `standalone` (externally hosted, visually isolated). See [UI Plugins docs](plugins/ui-plugins.md).

Each plugin is defined by a `plugin.toml` manifest in its own directory under `plugins/`:
```
plugins/
├── sip-core-ui/
│   └── plugin.toml           # First-party web frontend plugin
└── sip-migration-studio/
    ├── plugin.toml            # Hybrid plugin for CRM/CMMS data import
    ├── docs/                  # Plugin documentation
    └── examples/              # Sample CSV/JSON import files
```

### Reference Plugins

**`sip-migration-studio`**: A hybrid plugin that imports customer data from external CRMs and CMMS systems. CSV and JSON file upload with client-side parsing → field mapping wizard → validation → dry run → execute import → rollback. Built on the Migration Core Framework. Consumes the same `/api/v1/migrations/*` API that any plugin can use.

**`sip-test-bench`**: A standalone UI plugin that hosts its own application. Can load a separately prepared SMRT fixture through the migration pipeline, streams AI answers and shows their retrieval and verification traces, exercises any endpoint as any of the six demo roles, and runs a battery of integrity checks. It is the first plugin to ship its own UI: the API serves `plugins/sip-test-bench/dist/` and the shell mounts it in an iframe. See [Hosting a Plugin's UI](plugin-hosting.md).

**Plugin Development Docs:**

- [Plugin Architecture Overview](plugins/overview.md)
- [Hosting a Plugin's UI](plugin-hosting.md). The iframe host, the `mount` field, and the token handshake.
- [Manifest Reference](plugins/manifest-reference.md). Every field in `plugin.toml`.
- [API Reference](plugins/api-reference.md). Plugin discovery endpoints.
- [Extension Points](plugins/extension-points.md). 35 UI injection slots.
- [Examples](plugins/examples.md). Walkthroughs for common plugin types.

**Migration Docs:**

- [Migration API Reference](migrations/api-reference.md)
- [Importable object types and load order](migrations/overview.md#supported-object-types)
- [External ID Mapping](migrations/external-id-mapping.md)
- [Migration Studio Plugin](../plugins/sip-migration-studio/README.md)

**Optional test data:**

- [SMRT Maintenance Logs fixture](../data/smrt-maintenance-logs/README.md). Tools for preparing maintenance narratives as SIP import records. Dataset files are obtained separately and are not installed automatically.

**Implementation Tracker:** [PROGRESS.md](PROGRESS.md). Full PRD implementation status.

---

## Capabilities

### Capability inventory

| # | Capability | Details |
|---|-----------|--------|
| 1 | **Organization & tenant model** | Full CRUD. Slug-based tenancy. RLS isolation. OrganizationSettings (timezone, currency, feature flags, AI config). |
| 2 | **User model & RBAC** | JWT auth with Argon2id. 6 roles (Admin, Manager, Technician, Viewer, Vendor, Auditor). Permission enforcement in every service method. |
| 3 | **Location hierarchy** | Site → Building → Floor → Room. CRUD with parent/child. Geo coordinates. Children/assets sub-resources. |
| 4 | **AssetType with JSON Schema** | Shared reference types for Pump, Motor, Conveyor, HVAC Unit, Vehicle, and Generic Equipment. Custom attribute schemas. |
| 5 | **Asset registry** | Full CRUD. Status lifecycle (Operational↔Degraded↔Down↔Maintenance→Retired). State machine validation. Parent/child hierarchy. Tagging. Location assignment. Firmware/software/hardware version tracking. |
| 6 | **Manufacturer & AssetModel** | Manufacturer and model records with CRUD for both. Asset pre-populates type+attributes when model selected. |
| 7 | **Work order lifecycle** | Full 10-state machine: Draft → Open → Assigned → Accepted → InProgress → OnHold → Completed → Reviewed → Closed. Cancel + Reopen. 9 lifecycle sub-endpoints. Time tracking. Resolution notes. |
| 8 | **Preventive maintenance scheduling** | Cron-based schedules. CronExpression parser. Auto-generate WOs on trigger. Deduplication. 60s background tick loop. |
| 9 | **Inspections** | Work orders with checklist items. PASS_FAIL, NUMERIC, TEXT, PHOTO response types. Failed items recorded. |
| 10 | **Activity audit log** | Append-only activity records. Tracks actor (human, AI agent, system, plugin), entity, action. Records service activity and state changes. |
| 11 | **Document records** | Metadata, processing states, linking, and archive support. The running API has no file-upload ingestion path yet. |
| 12 | **Parts inventory** | CRUD with quantity tracking. Atomic quantity_on_hand decrement on usage. Low stock detection. |
| 13 | **AI Q&A** | Optional provider-backed streaming chat with operational context and stored source references. The full SIPmem verification pipeline is not connected. |
| 14 | **AI source references** | Source and retrieval-trace records support inspection. Source references do not guarantee that generated claims are correct. |
| 15 | **Work order assignments** | Multi-assignee support (User, Team, Vendor, AI Agent). Role-based assignment (Primary, Secondary, Observer, Approver, etc.). |
| 16 | **Docker Compose deployment** | API, frontend, PostgreSQL, Redis, and MinIO, plus a migration job and optional Ollama profile. Health checks on core services. |
| 17 | **First-administrator setup** | Fresh installs remove the old demo tenant. A token-protected bootstrap endpoint creates the first organization and administrator. See the Quick Start. |
| 18 | **Plugin framework** | TOML manifest registry. UI/functional/hybrid plugin types. First-party frontend shipped as `sip-core-ui` plugin. Plugin discovery API endpoints. Sidebar navigation driven by plugin registry. Declarative extension points (35 defined). Manifest validation (unique IDs, semver, dependency checks, navigation validation). |
| 19 | **Migration Core** | Canonical import DTOs, field mappings, external ID mapping, validation, dry runs, imports, and partial rollback. Seven object types currently have import handlers; see the migration overview for supported types and limits. |
| 20 | **SIPmem evidence engine design** | Typed memory, fact ledger, temporal resolver, retrieval recipes, and pluggable verification components exist in separate crates. They are not wired into the live API pipeline. |
| 21 | **UI Plugin Platform** | Manifest contract: UiCompatibility (native/compatible/standalone), UiThemeConfig, UiRouteDef, UiActionDef. Design tokens v1.0 (30+ CSS custom properties, dark mode, density modes). 11 shared SIP UI components (SipPage, SipCard, SipButton, SipBadge, SipAlert, etc.). /api/v1/ui/theme endpoint. Example native plugin manifest. |

---

## API examples

OpenAPI specification: [`docs/openapi.yaml`](openapi.yaml)

### Authentication

First installation also uses `POST /api/v1/auth/bootstrap`. See the [first-administrator setup](../QUICKSTART.md#create-your-first-administrator).

```
POST   /api/v1/auth/login          → { token, refresh_token }
POST   /api/v1/auth/refresh         → { token }
POST   /api/v1/auth/logout
GET    /api/v1/auth/me              → { id, name, email, role, permissions }
```

### Assets

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/assets` | List all assets |
| `POST` | `/api/v1/assets` | Create an asset |
| `GET` | `/api/v1/assets/{id}` | Get asset by ID |
| `PATCH` | `/api/v1/assets/{id}` | Update asset status |
| `POST` | `/api/v1/assets/{id}/archive` | Soft-delete asset |
| `GET` | `/api/v1/assets/{id}/children` | List child assets |
| `GET` | `/api/v1/assets/{id}/work-orders` | List work orders for asset |

### Work Orders

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/work-orders` | List all work orders |
| `POST` | `/api/v1/work-orders` | Create work order (Draft) |
| `GET` | `/api/v1/work-orders/{id}` | Get work order by ID |
| `PATCH` | `/api/v1/work-orders/{id}` | Update work order |
| `POST` | `/api/v1/work-orders/{id}/publish` | Draft → Open |
| `POST` | `/api/v1/work-orders/{id}/start` | Accepted → InProgress |
| `POST` | `/api/v1/work-orders/{id}/hold` | InProgress → OnHold |
| `POST` | `/api/v1/work-orders/{id}/resume` | OnHold → InProgress |
| `POST` | `/api/v1/work-orders/{id}/complete` | InProgress → Completed |
| `POST` | `/api/v1/work-orders/{id}/review` | Completed → Reviewed |
| `POST` | `/api/v1/work-orders/{id}/close` | Reviewed → Closed |
| `POST` | `/api/v1/work-orders/{id}/cancel` | Any → Cancelled |
| `POST` | `/api/v1/work-orders/{id}/reopen` | Closed → Open/InProgress |
| `POST` | `/api/v1/work-orders/{id}/archive` | Soft-delete |
| `GET` | `/api/v1/work-orders/{id}/assignments` | List assignments |
| `POST` | `/api/v1/work-orders/{id}/assignments` | Create assignment |
| `PATCH` | `/api/v1/work-orders/{id}/assignments/{aid}` | Update assignment |
| `DELETE` | `/api/v1/work-orders/{id}/assignments/{aid}` | Remove assignment |
| `GET` | `/api/v1/work-orders/{id}/parts` | List parts used |
| `POST` | `/api/v1/work-orders/{id}/parts` | Log part usage |

### Schedules, Inspections, Parts

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/schedules` | List schedules |
| `POST` | `/api/v1/schedules` | Create schedule |
| `GET` | `/api/v1/schedules/{id}` | Get schedule |
| `PATCH` | `/api/v1/schedules/{id}` | Update schedule |
| `POST` | `/api/v1/schedules/{id}/archive` | Archive schedule |
| `GET` | `/api/v1/inspections` | List inspections |
| `GET` | `/api/v1/inspections/{id}` | Get inspection |
| `PATCH` | `/api/v1/inspections/{id}/items` | Update checklist items |
| `GET` | `/api/v1/parts` | List parts |
| `POST` | `/api/v1/parts` | Create part |
| `GET` | `/api/v1/parts/{id}` | Get part |
| `PATCH` | `/api/v1/parts/{id}` | Update part |

### Supporting APIs

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/locations` | List locations |
| `POST` | `/api/v1/locations` | Create location |
| `GET` | `/api/v1/locations/{id}` | Get location |
| `PATCH` | `/api/v1/locations/{id}` | Update location |
| `GET` | `/api/v1/locations/{id}/children` | List child locations |
| `GET` | `/api/v1/locations/{id}/assets` | List assets at location |
| `GET` | `/api/v1/organizations/me` | Get current org |
| `PATCH` | `/api/v1/organizations/me` | Update org settings |
| `POST` | `/api/v1/organizations` | Create organization |
| `GET` | `/api/v1/asset-types` | List asset types |
| `POST` | `/api/v1/asset-types` | Create asset type |
| `GET` | `/api/v1/asset-types/{id}` | Get asset type |
| `PATCH` | `/api/v1/asset-types/{id}` | Update asset type |
| `GET` | `/api/v1/manufacturers` | List manufacturers |
| `POST` | `/api/v1/manufacturers` | Create manufacturer |
| `GET` | `/api/v1/manufacturers/{id}` | Get manufacturer |
| `GET` | `/api/v1/manufacturers/{id}/models` | List models for manufacturer |
| `GET` | `/api/v1/models` | List models |
| `POST` | `/api/v1/models` | Create model |
| `GET` | `/api/v1/models/{id}` | Get model |
| `GET` | `/api/v1/users` | List users |
| `POST` | `/api/v1/users` | Create user |
| `GET` | `/api/v1/users/{id}` | Get user |
| `PATCH` | `/api/v1/users/{id}` | Update user |
| `GET` | `/api/v1/teams` | List teams |
| `POST` | `/api/v1/teams` | Create team |
| `GET` | `/api/v1/teams/{id}` | Get team |
| `PATCH` | `/api/v1/teams/{id}` | Update team |
| `GET` | `/api/v1/documents` | List documents |
| `POST` | `/api/v1/documents` | Create document metadata |
| `GET` | `/api/v1/documents/{id}` | Get document |
| `POST` | `/api/v1/documents/{id}/archive` | Archive document |
| `GET` | `/api/v1/activities` | List activities |

### AI Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/ai/chat` | AI Q&A (SSE streaming; see the architecture guide for the current path) |
| `GET` | `/api/v1/ai/conversations` | List AI conversations |
| `GET` | `/api/v1/ai/conversations/{id}` | Get conversation + messages |
| `GET` | `/api/v1/ai/messages/{id}/retrieval-trace` | Get retrieval trace |
| `GET` | `/api/v1/ai/messages/{id}/verification-trace` | Get verification trace |
| `POST` | `/api/v1/ai/messages/{id}/feedback` | Submit feedback on AI answer |

### Utility

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/export` | Export all tenant data (JSON) |
| `POST` | `/api/v1/export/work-orders` | Export work orders |
| `GET` | `/api/v1/capabilities` | List enabled features |
| `GET` | `/api/v1/health` | Health check |
| `GET` | `/api/v1/health/ready` | Readiness check |
| `GET` | `/.well-known/sip-api` | API discovery |

### Response envelopes

Most resource endpoints use a `data` wrapper. Authentication and some utility endpoints differ; for example, login returns `token` and `refresh_token` at the top level. This is an illustrative resource response:

```json
{
  "data": { "id": "example-record" }
}
```

```json
{
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Insufficient permissions"
  }
}
```

### Target AI response format (SIPmem design)

This is the intended answer contract, not the current streaming chat response. The full SIPmem pipeline is not wired into the API. See the [architecture guide](architecture.md).

```json
{
  "answer": "Pump A has had three bearing failures in the last 90 days.",
  "confidence": 0.86,
  "verification_status": "VERIFIED",
  "retrieval_paths": ["SQL", "VECTOR", "TEMPORAL"],
  "sources": [
    {
      "type": "work_order",
      "id": "wo_123",
      "source_scope": "PRIVATE_TENANT",
      "retrieval_type": "VECTOR",
      "verified_by_sql": true,
      "quote": "Bearing assembly replaced due to inner race spalling...",
      "timestamp": "2026-04-21T15:00:00Z"
    }
  ],
  "verified_claims": [
    {
      "claim": "Three bearing failures in 90 days",
      "status": "VERIFIED",
      "supporting_sources": ["wo_123", "wo_124", "wo_125"]
    }
  ],
  "unsupported_claims": [],
  "contradicted_claims": [],
  "recommended_actions": [
    {
      "action": "create_work_order",
      "title": "Inspect bearing assembly on Pump A",
      "justification": "Three failures in 90 days suggests systemic issue",
      "requires_confirmation": true
    }
  ]
}
```

---

## Project structure

| Path | Purpose |
|------|---------|
| `Cargo.toml`, `Cargo.lock` | Rust workspace and pinned dependencies. |
| `crates/sip-api/` | HTTP server, routes, middleware, and response types. |
| `crates/sip-application/` | Services and import orchestration. |
| `crates/sip-domain/` | Entities, IDs, repository traits, errors, and state rules. |
| `crates/sip-infrastructure/` | SQLx repositories and data-service adapters. |
| `crates/sip-auth/`, `crates/sip-tenancy/` | Passwords, tokens, permissions, and tenant context. |
| `crates/sip-scheduler/` | Cron parsing and recurring-work scheduling. |
| `crates/sip-config/`, `crates/sip-observability/` | Configuration and tracing. |
| `crates/sip-ai/`, `crates/sip-search/` | Provider integration and search-related code. |
| `crates/sipmem-core/`, `crates/sipmem-adapters/` | Memory types, recipes, retrieval, and verification design. |
| `crates/sip-plugins/` | Plugin manifests, registry, and discovery. |
| `crates/sip-assets/`, `crates/sip-work-orders/`, `crates/sip-parts/`, `crates/sip-documents/`, `crates/sip-activity/`, `crates/sip-outbox/` | Domain-specific modules; see the architecture guide for runtime connections. |
| `crates/sip-cli/` | CLI command placeholders. |
| `frontend/app/` | Dashboard, equipment, maintenance, inventory, team, document, chat, and settings pages. |
| `frontend/lib/` | API client and authentication state. |
| `frontend/components/sip/` | Shared UI components and plugin boundaries. |
| `migrations/` | Schema, indexes, tenant policies, and forward migrations, including removal of the old demo tenant. |
| `plugins/` | Core UI manifest, Migration Studio, and Test Bench. |
| `data/` | Optional test-data preparation tools. |
| `docker/`, `docker-compose.yml` | Image builds and local services. |
| `.env.example`, `Sip.toml` | Local configuration examples. |
| `docs/` | Guides, API and domain references, design, and planning. |

The [architecture guide](architecture.md) describes which modules are connected
at runtime. The [development guide](development.md) covers builds and configuration.

## Service domain model

These tables describe the domain vocabulary. Entity definitions and state rules live in [`sip-domain`](../crates/sip-domain/src).

### Core Entities

| Entity | Description | State Machine |
|--------|-------------|---------------|
| **Organization** | Tenancy boundary | Active → Archived |
| **OrganizationSettings** | Timezone, currency, AI config, feature flags | None |
| **Location** | Physical/logical site (Site→Building→Floor→Room) | None |
| **Asset** | Any physical thing requiring maintenance | Operational↔Degraded↔Down↔Maintenance→Retired |
| **AssetType** | Classification with JSON Schema custom attributes | None |
| **Manufacturer** | Company that makes assets | None |
| **AssetModel** | Specific product model | Active→Deprecated→EndOfSupport→Retired |
| **WorkOrder** | Unit of maintenance work | Draft→Open→Assigned→Accepted→InProgress↔OnHold→Completed→Reviewed→Closed |
| **WorkOrderAssignment** | Multiple assignees per WO with roles | Assigned→Accepted→Declined→Removed→Completed |
| **WorkOrderStatusHistory** | Query-optimized lifecycle timeline | None |
| **Schedule** | Recurring PM definition | Cron expression → auto-generate WOs |
| **Inspection** | Structured checklist on INSPECTION work orders | None |
| **InspectionChecklistItem** | PASS_FAIL, NUMERIC, TEXT, PHOTO items | None |
| **Part** | Consumable inventory | quantity_on_hand decremented atomically |
| **PartUsage** | Parts consumed by a work order | None |
| **Document** | File metadata and intended processing lifecycle | Pending→Extracting→...→Indexed→Failed |
| **DocumentLink** | Links documents to entities | None |
| **DocumentChunk** | Text segments for embedding | None |
| **EmbeddingRecord** | Vector embedding with source reference | None |
| **Activity** | Append-only activity record | None |
| **AIConversation** | Multi-turn AI chat session | None |
| **AIMessage** | Chat message with sources, traces | None |
| **AIRetrievalTrace** | What was retrieved and why | None |
| **VerificationTrace** | Answer accuracy checkpoint | None |
| **AgentIdentity** | AI agent identity record | None |

### Permission Model

| Role | Scope |
|------|-------|
| **Admin** | `*` (all permissions) |
| **Manager** | org:read, asset:create/update/read/archive, work_order:create/update/read/cancel/assign/review, schedule:manage/read, user:read, team:manage, part:manage/read, document:upload/read/archive, activity:read, ai:query |
| **Technician** | org:read, asset:read, work_order:create/update/read, schedule:read, user:read, part:consume/read, document:upload/read, ai:query |
| **Viewer** | org:read, asset:read, work_order:read, schedule:read, user:read, part:read, document:read, ai:query |
| **Vendor** | asset:read, work_order:read, part:read, document:read, ai:query |
| **Auditor** | org:read, asset:read, work_order:read, schedule:read, user:read, part:read, document:read, activity:read, ai:query |

---

## Database

### Tables

The schema covers organizations, locations, assets, asset_types, manufacturers, asset_models, work_orders, work_order_assignments, work_order_status_history, schedules, inspections, inspection_checklist_items, parts, part_usage, asset_parts, users, teams, team_members, documents, document_links, document_chunks, embedding_records, activities, idempotency_keys, agent_identities, ai_conversations, ai_messages, ai_retrieval_traces, outbox_events, verification_traces, ai_answer_feedback, organization_settings.

### Extensions

- `pgvector`: vector embeddings with HNSW indexing
- `ltree`: hierarchical path queries on locations
- Geographic coordinates use ordinary columns; PostGIS is not required by the current migrations.
- `pg_trgm`: trigram text search
- `uuid-ossp`: UUID generation

### Row-Level Security

The database includes tenant row-level-security policies and tenant-context helpers. Their enforcement depends on the database role and query path. See the [authentication and tenancy analysis](architecture.md) for current behavior and limitations.
