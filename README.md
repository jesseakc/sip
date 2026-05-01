<h1 align="center">SIP — Service Intelligence Platform</h1>

<p align="center">
  <strong>The open-source, AI-native CMMS and asset CRM for the physical world</strong><br/>
  Structured maintenance intelligence for LLMs. Built in Rust. Licensed AGPLv3.
</p>

<p align="center">
  <a href="#quick-start"><img src="https://img.shields.io/badge/docker-compose_up_→_running-blue" /></a>
  <a href="#demo-login"><img src="https://img.shields.io/badge/demo-login_here-lightgrey" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-AGPLv3-green" /></a>
  <a href="./docs/openapi.yaml"><img src="https://img.shields.io/badge/API-OpenAPI_3.0-orange" /></a>
  <img src="https://img.shields.io/badge/language-Rust_+_TypeScript-purple" />
</p>

---

## The Problem

The software that maintains the physical world is broken.

### Legacy CMMS: data silos that can't reason

Every factory, hospital, fleet, building, and power plant relies on a **CMMS** (Computerized Maintenance Management System) to track assets, schedule preventive maintenance, and log work orders. The market for this software is $1.2 billion and growing at 9% CAGR — yet the dominant platforms (IBM Maximo, SAP PM, Oracle EAM) were architected decades ago. They share the same fundamental flaws:

| Problem | What It Means In Practice |
|----------|---------------------------|
| **Single-industry schemas** | A manufacturing CMMS can't model a hospital's HVAC system. A fleet platform can't handle kitchen equipment. Every industry reinvents the data model. |
| **Human-only interfaces** | Data is stored for PDF compliance reports, not for machines. There is no API an AI agent can query. No MCP endpoints. No function-calling tools. |
| **AI bolted on as marketing** | Legacy vendors add a "chat with your data" widget that RAGs over unstructured PDFs. No structured retrieval. No citations. No audit trail. No RBAC enforcement in the retrieval path. |
| **Closed-source lock-in** | Maintenance data outlives the software vendor. Organizations risk data hostage situations. Switching costs are existential. |
| **Reactive, not intelligent** | Work orders are logged after the fact. The question "what fixed this last time?" requires calling a senior technician, not querying a database. There is no compounding operational intelligence. |
| **No agentic surface** | AI agents cannot create work orders, assign technicians, check inventory, or surface compliance gaps — because the platform has no structured, permissioned API for them to use. |

### Asset CRM: the missing category

CRM (Customer Relationship Management) transformed sales and support by giving every customer a structured, queryable record with full interaction history. Salesforce built a $300 billion business on that insight. **But assets have no equivalent.**

When a technician writes "replaced bearing, found inner race spalling due to contamination" in a work order, that note should become retrievable intelligence for every future query about that asset type. Instead, it disappears into a closed database. There is no asset CRM — no system that treats assets as first-class entities with a complete, queryable, AI-consumable operational history.

### Why existing tools fall short

| Category | Examples | Why They Fail |
|----------|----------|---------------|
| **Enterprise CMMS** | IBM Maximo, SAP PM, Oracle EAM | $100K+ deployments, on-premise, zero AI integration, locked schemas |
| **Mid-Market CMMS** | Fiix, MaintainX, UpKeep | Single-industry focus, closed-source, AI is a marketing afterthought |
| **Open-Source CMMS** | openMAINT, Fracttal | No AI layer, no plugin ecosystem, limited adoption |
| **Horizontal Tools** | Jira, ServiceNow, Monday.com | Not maintenance-native; work orders need the asset context that these tools lack |
| **AI Wrappers** | CustomGPT, ChatPDF | Surface-level RAG over unstructured data — no permission model, no citation tracking, no audit trail, no state machine enforcement |

---

## Why SIP Is the Future

**SIP** is an open-source, AI-native **CMMS and asset CRM** designed from first principles for the age of LLMs and AI agents.

### Machine-readable before human-readable

SIP starts with a **service-domain canonical model** — a cross-industry, structured representation of every asset, work order, repair decision, inspection finding, replacement part, and resolution note. This is not a passive ledger. It is a **structured maintenance substrate** that both humans and AI agents consume through the same public REST API.

The principle is simple: if a human can see it, an AI must be able to query it — through the same endpoint, with the same auth, generating the same audit trail.

### AI is not a feature. It's the data model.

SIP does not bolt AI onto a legacy schema. The data model itself is AI-native:

- **Every entity is API-queryable.** Assets, work orders, schedules, inspections, parts, documents, and activities all have structured REST endpoints with consistent JSON schemas.
- **Every state change is audited immutably.** Activity records track who did what, when, and why. AI agents are first-class users with traceable identities.
- **Every AI answer is grounded.** SIPmem — SIP's hybrid memory system — enforces that answers cite retrievable source records. No hallucinated facts. No unsourced claims.
- **AI retrieval enforces authorization.** The same RLS and RBAC that protect the API also govern what the AI can retrieve. An AI query from a technician never surfaces data from another organization, or data the technician lacks permission to see.

### SIPmem: hybrid memory for service intelligence

SIPmem combines six cooperating memory layers in a single PostgreSQL database:

1. **SQL Memory** — Authoritative operational truth. Exact facts are verified against current database records.
2. **Vector Memory** — Semantic search over work order notes and document chunks via pgvector HNSW.
3. **RAG Memory** — Permission-safe context assembly with source citations and evidence ranking.
4. **Graph Memory** — Relationship traversal via parent/child hierarchies, ltree paths, and document links.
5. **Temporal Memory** — Change-over-time preservation through Activity records and status history tables.
6. **Verification Memory** — Post-response checks that cited records exist, are accessible, and support the claims made.

A retrieval router classifies each question into one of seven types (exact fact, similarity, relationship, temporal, document QA, root cause, hybrid) and dispatches to the appropriate memory layers per a decision matrix.

### Open source is a feature, not a tactic

SIP is **AGPLv3**. You can self-host it forever. Your maintenance data belongs to you. The business model is optional hosting and support — not data lock-in. Every feature available in a hosted version is available in the open-source version.

### Built for the real world

- **Cross-industry by design.** The schema lives in the database, not in the code. Asset types are JSON Schema documents. Any physical object — from a robotic arm to a dishwasher to a lawn — can be modeled without code changes.
- **Modular monolith.** 19 Rust crates with well-defined interface boundaries. Services extract only when there is a measurable bottleneck. Cargo feature flags let you compile only what you need.
- **Single command deploy.** `docker compose up` brings up the full stack: API, frontend, PostgreSQL+pgvector, Redis, MinIO, and Ollama — with migrations applied and seed data loaded.

---

## Table of Contents

- [Architecture](#architecture)
- [Plugin Architecture](#plugin-architecture)
- [Quick Start](#quick-start)
- [Demo Login](#demo-login)
- [MVP Features](#mvp-features)
- [API Reference](#api-reference)
- [Project Structure](#project-structure)
- [Development](#development)
- [SIPmem — Hybrid Memory System](#sipmem--hybrid-memory-system)
- [Service Domain Model](#service-domain-model)
- [License](#license)

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                      Any UI / Client                          │
│   web UI  ·  mobile PWA  ·  AI agents  ·  CLI  ·  MCP server │
└───────────────────────────┬──────────────────────────────────┘
                            │  REST API (JSON + SSE)
┌───────────────────────────▼──────────────────────────────────┐
│                    sip-api (Axum)                              │
│  JWT auth  ·  RBAC middleware  ·  64 endpoints  ·  SSE streaming│
└───────────────────────────┬──────────────────────────────────┘
                            │
┌───────────────────────────▼──────────────────────────────────┐
│              sip-application (services)                        │
│  AuthService  ·  AssetService  ·  WorkOrderService            │
│  ScheduleService  ·  PartService  ·  InspectionService        │
│  DocumentService  ·  ActivityService  ·  AIService            │
│  Permission enforcement  ·  State machine validation          │
└───────────────────────────┬──────────────────────────────────┘
                            │
┌───────────────────────────▼──────────────────────────────────┐
│              sip-domain (canonical model)                      │
│  26 entities  ·  state machines  ·  repository traits          │
│  ID newtypes  ·  SipError  ·  TenantContext                    │
└───────────────────────────┬──────────────────────────────────┘
                            │
┌───────────────────────────▼──────────────────────────────────┐
│           sip-infrastructure (data layer)                      │
│  PostgreSQL+pgvector+ltree  ·  Redis  ·  MinIO                 │
│  RLS tenant isolation  ·  optimistic locking                   │
└──────────────────────────────────────────────────────────────┘
```

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Backend** | Rust (Axum, Tokio, SQLx) | API server, domain logic, auth, scheduling |
| **Frontend** | TypeScript (Next.js 15) | Web UI, SSE streaming, PWA-capable |
| **Database** | PostgreSQL 16 + pgvector | Operational truth, vector search, RLS |
| **Vector Search** | pgvector (HNSW) | Semantic similarity over WO notes, document chunks |
| **Graph** | Recursive CTEs + ltree | Asset hierarchy, location paths |
| **AI** | Ollama / OpenAI / Anthropic / Bedrock | Provider abstraction via LLM provider trait |
| **Cache** | Redis/Valkey | Session store, rate limiting |
| **Object Storage** | MinIO (self-hosted) | Document uploads, extracted text |
| **Plugins** | TOML manifests + Rust registry | UI plugins, functional plugins, hybrid plugins |
| **Deployment** | Docker Compose | 6 containers, single `docker compose up` |

### Plugin Architecture

**The official SIP frontend is itself a UI plugin.** It is not a hardcoded special case — it ships as `sip-core-ui`, registered and discovered through the same plugin framework that future plugins will use.

SIP supports three plugin types:
- **UI plugins** — contribute navigation, routes, dashboards, forms, and extension points
- **Functional plugins** — contribute backend logic, API routes, jobs, automations, integrations, AI tools, and MCP tools
- **Hybrid plugins** — contribute both UI and backend capabilities

Each plugin is defined by a `plugin.toml` manifest in its own directory under `plugins/`:
```
plugins/
└── sip-core-ui/
    └── plugin.toml      # First-party web frontend plugin
```

**Plugin discovery** (public API, no auth required):
```bash
curl http://localhost:8000/api/v1/plugins           # List all enabled plugins
curl http://localhost:8000/api/v1/ui/navigation     # Aggregated sidebar navigation
curl http://localhost:8000/api/v1/ui/plugins        # UI-specific plugin info
curl http://localhost:8000/api/v1/ui/extension-points  # Available extension points
```

**The frontend sidebar is plugin-driven.** On page load, the Next.js shell fetches `/api/v1/ui/navigation` and renders the sidebar from the response. If the API is unavailable, it falls back to an embedded default navigation — so local development works without the backend running.

📖 **Plugin Development Docs:**
- [Plugin Architecture Overview](./docs/plugins/overview.md)
- [Manifest Reference](./docs/plugins/manifest-reference.md) — every field in `plugin.toml`
- [API Reference](./docs/plugins/api-reference.md) — plugin discovery endpoints
- [Extension Points](./docs/plugins/extension-points.md) — 35 UI injection slots
- [Examples](./docs/plugins/examples.md) — walkthroughs for common plugin types

---

## Quick Start

### Prerequisites

- **Docker** and **Docker Compose v2+**
- 16 GB RAM recommended (12 minimum — Ollama needs ~4 GB)

### Start the Full Stack

```bash
# 1. Clone the repo
git clone https://github.com/jesseakc/sip.git
cd sip

# 2. Copy environment template
cp .env.example .env

# 3. Start everything (API, frontend, PostgreSQL, Redis, MinIO, Ollama)
docker compose up --build

# 4. Wait for migrations + Ollama model pull (~2 minutes first run)
#    Then access:
#    - Frontend:  http://localhost:3000
#    - API:       http://localhost:8000
#    - MinIO:     http://localhost:9001
#    - Ollama:    http://localhost:11434
```

### Services (docker-compose.yml)

| Service | Port | Description |
|---------|------|-------------|
| `sip-frontend` | 3000 | Next.js 15 production build |
| `sip-api` | 8000 | Rust Axum API server |
| `postgres` | 5432 | PostgreSQL 16 + pgvector + postgis + ltree + pg_trgm |
| `redis` | 6379 | Redis 7 (Valkey) |
| `minio` | 9000/9001 | S3-compatible object storage |
| `ollama` | 11434 | Local LLM (default: `llama3.1:8b`) — optional if using hosted provider |

### Running Without Ollama (Hosted LLM or AI Disabled)

**1. AI Disabled Mode** — SIP works without any LLM:
```bash
SIP_AI_ENABLED=false SIP_AI_PROVIDER=disabled docker compose up
```
Or in `.env`:
```env
SIP_AI_ENABLED=false
SIP_AI_PROVIDER=disabled
```
Then: `docker compose up` (the ollama service will not start without `--profile ollama`).

**2. Hosted OpenAI Mode** — Use OpenAI instead of local Ollama:
```env
SIP_AI_ENABLED=true
SIP_AI_PROVIDER=openai
SIP_AI_OPENAI_API_KEY=sk-your-key-here
SIP_AI_OPENAI_MODEL=gpt-4o
```
Then: `docker compose up` (ollama is unused; omit `--profile ollama`).

**3. Local Ollama (default demo mode)**:
```bash
docker compose --profile ollama up
```

### Configuration Reference

SIP uses a layered configuration system with this precedence:
1. **Hardcoded safe defaults** (in `sip-config/src/lib.rs`)
2. **`Sip.toml`** (optional, placed in the working directory)
3. **`SIP_` prefixed environment variables** (highest precedence)

All environment variables use the `SIP_` prefix. Nested config uses underscores:
`SIP_AI_OLLAMA_URL` maps to `ai.ollama.url`.

Key configuration sections:

| Section | Purpose | Key Variables |
|---------|---------|---------------|
| `SIP_ENV` | Deployment profile | `local`, `development`, `test`, `staging`, `production` |
| `SIP_DATABASE_URL` | PostgreSQL connection | Required |
| `SIP_REDIS_URL` | Redis connection | Optional (caching, rate limiting) |
| `SIP_OBJECT_STORAGE_*` | MinIO/S3 | Required for document ingestion |
| `SIP_AUTH_JWT_SECRET` | JWT signing key | Required (min 32 chars in prod) |
| `SIP_AI_*` | LLM provider config | See `.env.example` for full reference |
| `SIP_FEATURES_*` | Feature flags | Enable/disable capabilities at runtime |

**Backward compatibility**: The old flat env vars `SIP_OLLAMA_URL` and `SIP_OLLAMA_MODEL` still work and are automatically mapped to `SIP_AI_OLLAMA_URL` / `SIP_AI_OLLAMA_MODEL`.

**Config inspection** (development/local only):
```bash
curl http://localhost:8000/admin/config/status -H "Authorization: Bearer <token>"
```
Shows active environment, AI provider, enabled features, validation status, and redacted config — never exposes secrets.

---

## Demo Login

The seed migration creates a complete demo organization. Use these credentials:

| Role | Email | Password |
|------|-------|----------|
| **Admin** | `admin@acme.local` | `password` |
| Manager | `manager@acme.local` | `password` |
| Technician | `tech1@acme.local` | `password` |
| Viewer | `viewer@acme.local` | `password` |
| Vendor | `vendor@acme.local` | `password` |
| Auditor | `auditor@acme.local` | `password` |

---

## MVP Features

### ✅ Implemented

| # | Capability | Status |
|---|-----------|--------|
| 1 | **Organization & tenant model** | Full CRUD. Slug-based tenancy. RLS isolation. OrganizationSettings (timezone, currency, feature flags, AI config). |
| 2 | **User model & RBAC** | JWT auth with Argon2id. 6 roles (Admin, Manager, Technician, Viewer, Vendor, Auditor). Permission enforcement in every service method. |
| 3 | **Location hierarchy** | Site → Building → Floor → Room. CRUD with parent/child. Geo coordinates. Children/assets sub-resources. |
| 4 | **AssetType with JSON Schema** | 6 reference types shipped (Pump, Motor, Conveyor, HVAC Unit, Vehicle, Generic Equipment). Custom attribute schemas. |
| 5 | **Asset registry** | Full CRUD. Status lifecycle (Operational↔Degraded↔Down↔Maintenance→Retired). State machine validation. Parent/child hierarchy. Tagging. Location assignment. Firmware/software/hardware version tracking. |
| 6 | **Manufacturer & AssetModel** | 5 manufacturers, 25+ models. CRUD for both. Asset pre-populates type+attributes when model selected. |
| 7 | **Work order lifecycle** | Full 10-state machine: Draft → Open → Assigned → Accepted → InProgress → OnHold → Completed → Reviewed → Closed. Cancel + Reopen. 9 lifecycle sub-endpoints. Time tracking. Resolution notes. |
| 8 | **Preventive maintenance scheduling** | Cron-based schedules. CronExpression parser. Auto-generate WOs on trigger. Deduplication. 60s background tick loop. |
| 9 | **Inspections** | Work orders with checklist items. PASS_FAIL, NUMERIC, TEXT, PHOTO response types. Failed items recorded. |
| 10 | **Activity audit log** | Immutable append-only. Tracks actor (human, AI agent, system, plugin), entity, action. Created on every state transition. |
| 11 | **Document management** | Upload metadata. Processing status lifecycle (Pending→Extracting→...→Indexed→Failed). Archive. Link to assets/work orders. |
| 12 | **Parts inventory** | CRUD with quantity tracking. Atomic quantity_on_hand decrement on usage. Low stock detection. |
| 13 | **AI Q&A (Knowledge Agent)** | SSE streaming chat. RAG pipeline with relational context retrieval. Vector similarity search (pgvector). Structured JSON responses with confidence, citations, sources. |
| 14 | **AI citations** | Every response includes source citations (work order ID, document name, timestamp, quote). "I don't know" fallback when no context. |
| 15 | **Work order assignments** | Multi-assignee support (User, Team, Vendor, AI Agent). Role-based assignment (Primary, Secondary, Observer, Approver, etc.). |
| 16 | **Docker Compose deployment** | 6 services. Auto-migration. Ollama model pull. Health checks on all services. |
| 17 | **Seed data** | Demo org, 35 assets, 50 work orders across all states, 8 users across all roles, 2 teams, 5 schedules, 2 inspections, 3 documents, AI conversation. |
| 18 | **Plugin framework** | TOML manifest registry. UI/functional/hybrid plugin types. First-party frontend shipped as `sip-core-ui` plugin. Plugin discovery API endpoints. Sidebar navigation driven by plugin registry. Declarative extension points (35 defined). Manifest validation (unique IDs, semver, dependency checks, navigation validation). |
| 19 | **Migration Core** | Full migration pipeline: 11 entities, canonical import DTOs, 16 API endpoints, field mappings, source→SIP external ID mapping, dry-run/import/rollback, 8 plugin extension points. RLS on all tables. Gated behind plugins feature flag. |

### 🚧 Deferred to Later Phases

| Feature | Phase |
|---------|-------|
| Per-tenant LLM provider config | 1.5 |
| Advanced RAG pipeline (re-ranking, hybrid search) | 1.5 |
| Notification service (email, push, webhook) | 2 |
| AI tool execution (create WO, update status) | 2 |
| Predictive maintenance ML | 3 |
| Compliance agent (ISO/OSHA/FDA) | 3 |
| Native mobile apps | 4 |
| Enterprise SSO (SAML/OIDC) | 4 |

---

## API Reference

Full OpenAPI 3.0 specification: [`docs/openapi.yaml`](./docs/openapi.yaml)

### Authentication

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
| `POST` | `/api/v1/ai/chat` | AI Q&A (SSE streaming with SIPmem Answer Contract) |
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

### Response Envelope

All API responses use a consistent envelope:

```json
{
  "status": "ok",
  "data": { ... }
}
```

```json
{
  "status": "error",
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Insufficient permissions"
  }
}
```

### AI Response Format (SIPmem Answer Contract)

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

## Project Structure

```
sip/
├── Cargo.toml                    # Rust workspace (19 crates)
├── Cargo.lock                    # Pinned dependencies
├── .dockerignore
├── .gitignore
├── docker-compose.yml            # Full local stack
├── .env.example                  # Environment template
├── LICENSE                       # AGPLv3
│
├── crates/                       # Rust modular monolith
│   ├── sip-api/                  # Axum HTTP server (binary) — 64 routes
│   │   ├── src/main.rs           # Router, middleware, app state
│   │   ├── src/routes/           # 15 route modules (auth, assets, work_orders, etc.)
│   │   ├── src/middleware/       # JWT auth middleware
│   │   └── src/response.rs       # Consistent response envelope
│   │
│   ├── sip-application/          # Application services (Service Layer)
│   │   └── src/services.rs       # All service logic (AuthService, AssetService,
│   │                             #   WorkOrderService, AIService, etc.)
│   │
│   ├── sip-domain/               # Domain entities, state machines, traits
│   │   ├── src/entity/           # 26 entity types
│   │   ├── src/id.rs             # UUID newtype wrappers
│   │   ├── src/repository.rs     # Repository traits
│   │   ├── src/error.rs          # SipError enum
│   │   └── src/tenant.rs         # TenantContext + permission checks
│   │
│   ├── sip-infrastructure/       # Data layer
│   │   ├── src/repositories.rs   # Pg*Repository implementations (SQLx)
│   │   ├── src/db.rs             # Connection pool
│   │   ├── src/storage.rs        # MinIO client
│   │   └── src/redis.rs          # Redis client
│   │
│   ├── sip-auth/                 # JWT, Argon2id, RBAC permission maps
│   ├── sip-tenancy/              # RLS session helpers (set_rls_org_pool)
│   ├── sip-scheduler/            # Cron engine, due date calculator
│   ├── sip-config/               # figment-based SipConfig (SIP_* prefix)
│   ├── sip-observability/        # Tracing, metrics
│   ├── sip-ai/                   # AI orchestration stubs (ready for Phase 1.5)
│   ├── sip-search/               # Search indexers stubs
│   ├── sip-parts/                # Parts service stubs
│   ├── sip-documents/            # Document processing stubs
│   ├── sip-activity/             # Activity service stubs
│   ├── sip-work-orders/          # WO state machine stubs
│   ├── sip-assets/               # Asset service stubs
│   ├── sip-outbox/               # Outbox event stubs
│   ├── sip-plugins/              # Plugin registry stubs
│   └── sip-cli/                  # Admin CLI (binary)
│
├── frontend/                     # Next.js 15 TypeScript frontend
│   ├── app/                      # App Router pages
│   │   ├── layout.tsx            # Root layout + AuthProvider
│   │   ├── shell.tsx             # Responsive sidebar navigation
│   │   ├── page.tsx              # Dashboard
│   │   ├── login/                # Login page
│   │   ├── assets/               # Asset list + detail
│   │   ├── work-orders/          # Work order list + detail
│   │   ├── schedules/            # Schedule management
│   │   ├── inspections/          # Inspection list
│   │   ├── parts/                # Parts inventory
│   │   ├── locations/            # Location hierarchy
│   │   ├── teams/                # Team management
│   │   ├── users/                # User management
│   │   ├── documents/            # Document list
│   │   ├── ai-chat/              # AI chat (SSE streaming)
│   │   └── settings/             # Organization settings
│   ├── lib/                      # API client + Auth context
│   ├── next.config.ts            # standalone output
│   └── tailwind.config.ts        # Custom design tokens
│
├── migrations/                   # PostgreSQL migrations
│   ├── 001_initial_schema.sql    # 30 tables, enums, extensions
│   ├── 002_indexes.sql           # HNSW, B-tree indexes
│   ├── 003_rls_policies.sql      # Row-level security policies
│   ├── 004_seed_data.sql         # Demo data
│   ├── 005_fix_schema_issues.sql # Document enum fix, asset version
│   └── 006_sipmem_tables.sql     # Verification traces, feedback, enum extensions
│
├── docker/                       # Dockerfiles
│   ├── Dockerfile.api             # Multi-stage Rust build
│   └── Dockerfile.frontend        # Next.js standalone build
│
└── docs/                         # Documentation
    ├── PRD-v2.md                 # Full Product Requirements Document (v5.7)
    └── openapi.yaml              # OpenAPI 3.0 specification (1700+ lines)
```

---

## SIPmem — Hybrid Memory System

SIPmem is SIP's accurate hybrid memory system that produces grounded, auditable, permission-safe answers about physical assets and service operations.

### Six Memory Layers

| Layer | Purpose | Implementation |
|-------|---------|---------------|
| **SQL Memory** | Authoritative operational truth | PostgreSQL with RLS, services layer queries |
| **Vector Memory** | Semantic similarity search | pgvector HNSW, cosine similarity, work_order_notes + document_chunks |
| **RAG Memory** | Permission-safe context assembly | Multi-path retrieval, citation tracking, evidence ranking |
| **Graph Memory** | Relationship traversal | parent_id hierarchy, ltree materialized paths, DocumentLink |
| **Temporal Memory** | Change-over-time preservation | Activity audit log, WorkOrderStatusHistory, timestamps |
| **Verification Memory** | Answer accuracy checking | VerificationTraces, verified_claims, contradiction detection |

### Retrieval Router

Questions are classified into 7 types and routed to appropriate memory layers:

| Question Type | Primary | Secondary | Example |
|--------------|---------|-----------|---------|
| Exact Factual | SQL | Activity | "What is the serial number?" |
| Similarity | VECTOR | SQL verification | "Have we seen this symptom before?" |
| Relationship | GRAPH | SQL | "What assets depend on this?" |
| Temporal | TEMPORAL | SQL | "What changed since last month?" |
| Document QA | VECTOR | RAG | "What does the manual say?" |
| Root Cause | HYBRID | Agent-ranked | "Why does this keep failing?" |
| General | HYBRID | — | "Tell me about Pump A." |

### Product Principles

| # | Principle |
|---|-----------|
| **P1** | Machine-readable before human-readable — the API is the canonical representation |
| **P2** | The public API is the AI surface — AI agents use the same REST API as the UI |
| **P3** | Every AI answer must be grounded — citations from retrievable source records |
| **P4** | AI retrieval enforces authorization — same RLS + RBAC as direct API access |
| **P5** | Operational actions create reusable intelligence — every repair note is AI-indexed |
| **P10** | Audit everything, immutably — Activity records on every state change |

---

## Service Domain Model

### Core Entities

| Entity | Description | State Machine |
|--------|-------------|---------------|
| **Organization** | Tenancy boundary | Active → Archived |
| **OrganizationSettings** | Timezone, currency, AI config, feature flags | — |
| **Location** | Physical/logical site (Site→Building→Floor→Room) | — |
| **Asset** | Any physical thing requiring maintenance | Operational↔Degraded↔Down↔Maintenance→Retired |
| **AssetType** | Classification with JSON Schema custom attributes | — |
| **Manufacturer** | Company that makes assets | — |
| **AssetModel** | Specific product model | Active→Deprecated→EndOfSupport→Retired |
| **WorkOrder** | Unit of maintenance work | Draft→Open→Assigned→Accepted→InProgress↔OnHold→Completed→Reviewed→Closed |
| **WorkOrderAssignment** | Multiple assignees per WO with roles | Assigned→Accepted→Declined→Removed→Completed |
| **WorkOrderStatusHistory** | Query-optimized lifecycle timeline | — |
| **Schedule** | Recurring PM definition | Cron expression → auto-generate WOs |
| **Inspection** | Structured checklist on INSPECTION work orders | — |
| **InspectionChecklistItem** | PASS_FAIL, NUMERIC, TEXT, PHOTO items | — |
| **Part** | Consumable inventory | quantity_on_hand decremented atomically |
| **PartUsage** | Parts consumed by a work order | — |
| **Document** | Uploaded file with processing lifecycle | Pending→Extracting→...→Indexed→Failed |
| **DocumentLink** | Links documents to entities | — |
| **DocumentChunk** | Text segments for embedding | — |
| **EmbeddingRecord** | Vector embedding with source reference | — |
| **Activity** | Immutable audit log (append-only) | — |
| **AIConversation** | Multi-turn AI chat session | — |
| **AIMessage** | Chat message with sources, traces | — |
| **AIRetrievalTrace** | What was retrieved and why | — |
| **VerificationTrace** | Answer accuracy checkpoint | — |
| **AgentIdentity** | AI agent identity record | — |

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

## Development

### Rust Backend

Requires: **Rust 1.80+**, **PostgreSQL 16+** with pgvector, postgis, ltree extensions.

```bash
# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features native-tls,postgres

# Create database and run migrations
createdb sip
sqlx migrate run --database-url postgres://localhost:5432/sip
# Or apply manually:
for f in migrations/0*.sql; do
  psql postgres://localhost:5432/sip -f "$f"
done

# Check workspace
cargo check --workspace                  # ~1-2s
cargo check --workspace --all-features

# Run API server
cargo run -p sip-api

# Run CLI
cargo run -p sip-cli -- --help
```

### Next.js Frontend

Requires: **Node.js 22+**, **npm 10+**.

```bash
cd frontend
npm install
npm run dev         # Development (hot reload)
npm run build       # Production build
npm start           # Production server (standalone)
```

### Environment Variables

Copy `.env.example` to `.env` and adjust:

| Variable | Default | Required |
|----------|---------|----------|
| `SIP_DATABASE_URL` | `postgres://sip:sip@localhost:5432/sip` | Yes |
| `SIP_SERVER_HOST` | `0.0.0.0` | Yes |
| `SIP_SERVER_PORT` | `8000` | Yes |
| `SIP_JWT_SECRET` | `change-me-in-production-...` | **Change for production** |
| `SIP_REDIS_URL` | `redis://localhost:6379` | Yes |
| `SIP_MINIO_ENDPOINT` | `http://localhost:9000` | Yes |
| `SIP_MINIO_BUCKET` | `sip-documents` | Yes |
| `SIP_OLLAMA_URL` | `http://localhost:11434` | Yes |
| `SIP_OLLAMA_MODEL` | `llama3.1:8b` | Yes |
| `NEXT_PUBLIC_API_URL` | `http://localhost:8000/api/v1` | Yes |

### Feature Flags

```bash
# Full build (default)
cargo build -p sip-api

# Without AI
cargo build -p sip-api --no-default-features

# Specific features
cargo build -p sip-api --no-default-features --features "ai,export"
```

### Feature Flag Matrix

| Flag | Enables |
|------|---------|
| `ai` | AI chat endpoint, conversations, RAG pipeline, SSE streaming |
| `documents` | Document upload/list/archive endpoints |
| `export` | Data export endpoints |
| `plugins` | Plugin registry (Phase 1.5) |

---

## Database

### Tables

30 tables across 6 migrations covering organizations, locations, assets, asset_types, manufacturers, asset_models, work_orders, work_order_assignments, work_order_status_history, schedules, inspections, inspection_checklist_items, parts, part_usage, asset_parts, users, teams, team_members, documents, document_links, document_chunks, embedding_records, activities, idempotency_keys, agent_identities, ai_conversations, ai_messages, ai_retrieval_traces, outbox_events, verification_traces, ai_answer_feedback, organization_settings.

### Extensions

- `pgvector` — vector embeddings with HNSW indexing
- `ltree` — hierarchical path queries on locations
- `postgis` — geographic coordinates
- `pg_trgm` — trigram text search
- `uuid-ossp` — UUID generation

### Row-Level Security

Every tenant-owned table has RLS enabled with `organization_id = current_org_id()` policies. The `set_rls_org_pool` helper sets the session variable before each query.

---

## License

**GNU Affero General Public License v3.0** — see [LICENSE](./LICENSE).

SIP is open source. Users can self-host forever. Data portability is guaranteed. Every feature in the hosted version is available in the open-source version.
