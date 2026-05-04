<h1 align="center">SIP · Service Intelligence Platform</h1>

<p align="center">
  <strong>The open-source, AI-native CMMS and asset CRM for the physical world</strong><br/>
  Structured maintenance intelligence for LLMs. Built in Rust. Licensed AGPLv3.
</p>

<p align="center">
  <em>Part CMMS, part asset CRM, part agent-ready operational memory, and part Harness-as-a-Service for the physical world.</em>
</p>

<p align="center">
  <a href="#quick-start"><img src="https://img.shields.io/badge/docker-compose_up_→_running-blue" /></a>
  <a href="#demo-login"><img src="https://img.shields.io/badge/demo-login_here-lightgrey" /></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-AGPLv3-green" /></a>
  <a href="./docs/openapi.yaml"><img src="https://img.shields.io/badge/API-OpenAPI_3.0-orange" /></a>
  <img src="https://img.shields.io/badge/language-Rust_+_TypeScript-purple" />
  <img src="https://img.shields.io/badge/Rust-1.80%2B-orange" />
  <img src="https://img.shields.io/badge/PostgreSQL-16-blue" />
  <img src="https://img.shields.io/badge/Next.js-15-black" />
</p>

---

## Development & Testing

### Quick Test

```bash
# Run all tests (260+ tests across all crates)
cargo test --workspace --all-features

# Test a specific crate
cargo test -p sip-auth
cargo test -p sip-domain
cargo test -p sip-application

# Type-check and lint
cargo check --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --all -- --check

# Frontend
cd frontend && npx tsc --noEmit && npm run build
```

### Test Coverage by Crate

| Crate | Tests | Focus |
|-------|-------|-------|
| `sip-auth` | 24 | JWT encode/decode roundtrip, password hash/verify, RBAC roles, permission checks, default map |
| `sip-domain` | 48 | SipError Display/constructors, TenantContext permissions, state machine transitions, serde roundtrip for all 28 domain enums |
| `sip-application` | 16 | role_permissions for all UserRole variants, classify_query for all 7 question types |
| `sip-api` | 3 | JSON response envelope shapes (success, paginated, error) |
| `sip-config` | 13 | Default config loading, environment helpers, secret redaction, feature dependency validation, JWT/API key validation |
| `sip-ai` | 40 | Key rotation, provider registry, failover logic, error classification |
| `sip-plugins` | 29 | Manifest validation, registry operations, navigation aggregation |
| `sipmem-core` | 32 | Memory types, fact ledger, temporal resolver, evidence scoring, recipes, verification |
| `sipmem-adapters` | 18 | Retriever stubs, pipeline orchestration, router recipes, cross-reference verification |
| **Total** | **~260** | **0 failures across all crates** |

### Docker E2E Testing

```bash
# Pre-flight checks
./scripts/preflight.sh

# Start full stack
docker compose up --build

# Full API smoke test (health → login → migration → import → rollback)
./scripts/smoke.sh
```

Weekend test guide: [`docs/testing-weekend.md`](./docs/testing-weekend.md)

---

## Who SIP Is For

SIP is designed for:

- **Field service teams** maintaining physical equipment and infrastructure
- **Robotics companies** managing deployed machines and service fleets
- **Hospital biomedical** and clinical engineering teams
- **Facilities and plant maintenance** teams
- **Fleet, vehicle, and heavy equipment** service organizations
- **Developers** building AI agents, service plugins, and operational automation
- **Organizations migrating** away from closed CMMS or asset management systems

---

## The Problem

The software that maintains the physical world was not built for AI.

Traditional CMMS platforms are useful for work orders, preventive maintenance, compliance records, and asset tracking. But they were designed around human data entry and human reporting, not AI agents that can reason across service history, parts, failures, documents, procedures, permissions, and outcomes.

CRMs transformed sales and support by making customer history structured and queryable. Physical assets still do not have an equivalent. A robot, fleet vehicle, HVAC system, medical device, pump, or production line can accumulate years of operational knowledge, yet that knowledge is usually scattered across closed databases, free-text notes, PDFs, spreadsheets, and tribal memory.

Most modern "AI" features do not solve the core problem. They bolt chat interfaces onto legacy systems instead of rebuilding the data model, API layer, permission system, audit trail, and retrieval architecture for AI from the ground up.

SIP exists to make service intelligence structured, permission-safe, source-grounded, and agent-ready.

---

## Why SIP Is the Future

**SIP** is an open-source, AI-native **CMMS and asset CRM** designed from first principles for the age of LLMs and AI agents.

SIP is also designed as **Harness-as-a-Service**: a structured, permission-aware operational harness that lets AI agents and external systems safely interact with real-world service data, assets, documents, workflows, and actions. Instead of bolting AI onto a legacy CMMS, SIP gives AI a reliable harness built directly into the platform.

### Machine-readable before human-readable

SIP starts with a **service-domain canonical model**: a cross-industry, structured representation of every asset, work order, repair decision, inspection finding, replacement part, and resolution note. This is not a passive ledger. It is a **structured maintenance substrate** that both humans and AI agents consume through the same public REST API.

The principle is simple: if a human can see it, an AI must be able to query it. Same endpoint, same auth, same audit trail.

### AI is not a feature. It's the data model.

SIP does not bolt AI onto a legacy schema. The data model itself is AI-native:

- **Every entity is API-queryable.** Assets, work orders, schedules, inspections, parts, documents, and activities all have structured REST endpoints with consistent JSON schemas.
- **Every state change is audited immutably.** Activity records track who did what, when, and why. AI agents are first-class users with traceable identities.
- **Every AI answer is grounded.** SIPmem, SIP's hybrid memory system, enforces that answers cite retrievable source records. No hallucinated facts. No unsourced claims.
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

SIP is **AGPLv3**. You can self-host it forever. Your maintenance data belongs to you. The business model is optional hosting and support, not data lock-in. Every feature available in a hosted version is available in the open-source version.

### Built for the real world

- **Cross-industry by design.** The schema lives in the database, not in the code. Asset types are JSON Schema documents. Any physical object can be modeled without code changes, from a robotic arm to a dishwasher to a lawn mower.
- **Modular monolith.** 19 Rust crates with well-defined interface boundaries. Services extract only when there is a measurable bottleneck. Cargo feature flags let you compile only what you need.
- **Single command deploy.** `docker compose up` brings up the full stack: API, frontend, PostgreSQL+pgvector, Redis, and MinIO. Migrations run automatically and seed data loads on first start. Ollama is optional via `--profile ollama`.

---

## Why Existing CMMS and CRM Tools Fall Short

### Legacy CMMS: data silos that can't reason

Every factory, hospital, fleet, building, and power plant relies on a **CMMS** (Computerized Maintenance Management System) to track assets, schedule preventive maintenance, and log work orders. The market for this software is $1.2 billion and growing at 9% CAGR. Yet the dominant platforms (IBM Maximo, SAP PM, Oracle EAM) were architected decades ago. They share the same fundamental flaws:

| Problem | What It Means In Practice |
|----------|---------------------------|
| **Single-industry schemas** | A manufacturing CMMS can't model a hospital's HVAC system. A fleet platform can't handle kitchen equipment. Every industry reinvents the data model. |
| **Human-only interfaces** | Data is stored for PDF compliance reports, not for machines. There is no API an AI agent can query. No MCP endpoints. No function-calling tools. |
| **AI bolted on as marketing** | Legacy vendors add a "chat with your data" widget that RAGs over unstructured PDFs. No structured retrieval. No citations. No audit trail. No RBAC enforcement in the retrieval path. |
| **Closed-source lock-in** | Maintenance data outlives the software vendor. Organizations risk data hostage situations. Switching costs are existential. |
| **Reactive, not intelligent** | Work orders are logged after the fact. The question "what fixed this last time?" requires calling a senior technician, not querying a database. There is no compounding operational intelligence. |
| **No agentic surface** | AI agents cannot create work orders, assign technicians, check inventory, or surface compliance gaps. The platform has no structured, permissioned API for them to use. |

### Asset CRM: the missing category

CRM (Customer Relationship Management) transformed sales and support by giving every customer a structured, queryable record with full interaction history. Salesforce built a $300 billion business on that insight. **But assets have no equivalent.**

When a technician writes "replaced bearing, found inner race spalling due to contamination" in a work order, that note should become retrievable intelligence for every future query about that asset type. Instead, it disappears into a closed database. There is no asset CRM: no system that treats assets as first-class entities with a complete, queryable, AI-consumable operational history.

### Why existing tools fall short

| Category | Examples | Why They Fail |
|----------|----------|---------------|
| **Enterprise CMMS** | IBM Maximo, SAP PM, Oracle EAM | $100K+ deployments, on-premise, zero AI integration, locked schemas |
| **Mid-Market CMMS** | Fiix, MaintainX, UpKeep | Single-industry focus, closed-source, AI is a marketing afterthought |
| **Open-Source CMMS** | openMAINT, Fracttal | No AI layer, no plugin ecosystem, limited adoption |
| **Horizontal Tools** | Jira, ServiceNow, Monday.com | Not maintenance-native; work orders need the asset context that these tools lack |
| **AI Wrappers** | CustomGPT, ChatPDF | Surface-level RAG over unstructured data. No permission model, no citation tracking, no audit trail, no state machine enforcement. |

---

## Table of Contents

- [Why SIP Is the Future](#why-sip-is-the-future)
- [Why Existing CMMS and CRM Tools Fall Short](#why-existing-cmms-and-crm-tools-fall-short)
- [Architecture](#architecture)
- [Plugin Architecture](#plugin-architecture)
- [Quick Start](#quick-start)
- [Demo Login](#demo-login)
- [MVP Features](#mvp-features)
- [API Reference](#api-reference)
- [Project Structure](#project-structure)
- [Development](#development)
- [SIPmem · Hybrid Memory System](#sipmem--hybrid-memory-system)
- [Service Domain Model](#service-domain-model)
- [Documentation](#documentation)
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

The SIP architecture treats the platform as a **harness layer** between humans, AI agents, plugins, documents, databases, APIs, and physical assets. This Harness-as-a-Service model makes service workflows observable, auditable, extensible, and safe for both human operators and AI-driven systems.

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Backend** | Rust (Axum, Tokio, SQLx) | API server, domain logic, auth, scheduling |
| **Frontend** | TypeScript (Next.js 15) | Web UI, SSE streaming, PWA-capable |
| **Database** | PostgreSQL 16 + pgvector | Operational truth, vector search, RLS |
| **Vector Search** | pgvector (HNSW) | Semantic similarity over WO notes, document chunks |
| **Graph** | Recursive CTEs + ltree | Asset hierarchy, location paths |
| **AI / Memory** | SIPmem engine (Rust) | Typed memory, fact ledger, verification pipeline, pluggable retrievers |
| **Cache** | Redis/Valkey | Session store, rate limiting |
| **Object Storage** | MinIO (self-hosted) | Document uploads, extracted text |
| **Plugins** | TOML manifests + Rust registry | UI, functional, and hybrid plugins with declarative navigation |
| **UI Components** | React + Tailwind + design tokens | Shared SIP UI kit (11 components), theme tokens, dark mode |
| **Deployment** | Docker Compose | 5 default containers + optional Ollama profile |

### Plugin Architecture

**The official SIP frontend is itself a UI plugin.** It is not a hardcoded special case. It ships as `sip-core-ui`, registered and discovered through the same plugin framework that future plugins will use.

Plugins should extend SIP's **Harness-as-a-Service model** rather than bypass it. Functional plugins, UI plugins, migration plugins, automation plugins, and AI plugins all interact through documented APIs, shared permissions, event logs, and auditable service workflows. No plugin gets a back door.

SIP supports three plugin types:
- **UI plugins** — contribute navigation, routes, dashboards, forms, and extension points
- **Functional plugins** — contribute backend logic, API routes, jobs, automations, integrations, AI tools, and MCP tools
- **Hybrid plugins** — contribute both UI and backend capabilities

UI plugins declare a **compatibility level**: `native` (uses SIP shell, tokens, and components), `compatible` (uses SIP navigation but may have custom styling), or `standalone` (externally hosted, visually isolated). See [UI Plugins docs](./docs/plugins/ui-plugins.md).

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

**`sip-migration-studio`** — A hybrid plugin that imports customer data from external CRMs and CMMS systems. CSV and JSON file upload with client-side parsing → field mapping wizard → validation → dry run → execute import → rollback. Built on the Migration Core Framework. Consumes the same `/api/v1/migrations/*` API that any plugin can use.

📖 **Plugin Development Docs:**
- [Plugin Architecture Overview](./docs/plugins/overview.md)
- [Manifest Reference](./docs/plugins/manifest-reference.md). Every field in `plugin.toml`.
- [API Reference](./docs/plugins/api-reference.md). Plugin discovery endpoints.
- [Extension Points](./docs/plugins/extension-points.md). 35 UI injection slots.
- [Examples](./docs/plugins/examples.md). Walkthroughs for common plugin types.

📖 **Migration Docs:**
- [Migration API Reference](./docs/migrations/api-reference.md)
- [External ID Mapping](./docs/migrations/external-id-mapping.md)
- [Migration Studio Plugin](./plugins/sip-migration-studio/README.md)

📖 **Implementation Tracker:** [PROGRESS.md](./docs/PROGRESS.md). Full PRD implementation status.

---

## Quick Start

### Prerequisites

- **Docker** and **Docker Compose v2+**
- 16 GB RAM recommended (12 minimum; Ollama needs ~4 GB)

### Start the Full Stack

```bash
# 1. Clone the repo
git clone https://github.com/jesseakc/sip.git
cd sip

# 2. Copy environment template
cp .env.example .env

# 3. Start everything (API, frontend, PostgreSQL, Redis, MinIO — AI disabled by default)
docker compose up --build

# 4. To include Ollama for local AI:
docker compose --profile ollama up --build

# 5. Or use a hosted AI provider by setting env vars in .env
#    (see "Hosted OpenAI Mode" below)

# 6. Wait for migrations (~30s first run; plus Ollama model pull ~2 min if using --profile ollama)
#    Then access:
#    - Frontend:  http://localhost:3000
#    - API:       http://localhost:8000
#    - MinIO:     http://localhost:9001
#    - Ollama:    http://localhost:11434  (only with --profile ollama)
```

### Verify the Install

```bash
./scripts/preflight.sh    # Check prerequisites and configuration
./scripts/smoke.sh        # End-to-end API test (health → login → migration → rollback)
```

`preflight.sh` checks Docker, ports, `.env`, and compose config. `smoke.sh` verifies health, login, plugin discovery, migration creation, validation, dry run, import, and rollback.

### Services (docker-compose.yml)

| Service | Port | Description |
|---------|------|-------------|
| `sip-frontend` | 3000 | Next.js 15 production build |
| `sip-api` | 8000 | Rust Axum API server |
| `postgres` | 5432 | PostgreSQL 16 + pgvector + postgis + ltree + pg_trgm |
| `redis` | 6379 | Redis 7 (Valkey) |
| `minio` | 9000/9001 | S3-compatible object storage |
| `ollama` | 11434 | Local LLM (default: `llama3.1:8b`) — **optional** (requires `--profile ollama`) |

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
Shows active environment, AI provider, enabled features, validation status, and redacted config. Never exposes secrets.

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

## Test a Sample Migration

After the stack is running and you are logged in as `admin@acme.local`, open **Migration Studio** and use the sample files in `plugins/sip-migration-studio/examples/generic-cmms/`.

**First test:**
1. Create a new migration job (source: CSV, object: Asset).
2. Upload `assets.csv`.
3. Map source fields to SIP asset fields (name → name, serial_number → serial_number, status → status).
4. Run validation — should produce 0 errors.
5. Run dry run — should show 4 records ready to create.
6. Execute import — 4 assets created.
7. Confirm imported assets appear in the asset registry (`/assets`).
8. Roll back the import — 4 assets archived.

For the full weekend test path: [`docs/testing-weekend.md`](./docs/testing-weekend.md)

---

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
| 16 | **Docker Compose deployment** | 5 default services plus optional Ollama profile. Auto-migration. Health checks on core services. |
| 17 | **Seed data** | Demo org, 35 assets, 50 work orders across all states, 8 users across all roles, 2 teams, 5 schedules, 2 inspections, 3 documents, AI conversation. |
| 18 | **Plugin framework** | TOML manifest registry. UI/functional/hybrid plugin types. First-party frontend shipped as `sip-core-ui` plugin. Plugin discovery API endpoints. Sidebar navigation driven by plugin registry. Declarative extension points (35 defined). Manifest validation (unique IDs, semver, dependency checks, navigation validation). |
| 19 | **Migration Core** | Full migration pipeline: 11 entities, canonical import DTOs, 16 API endpoints, field mappings, source→SIP external ID mapping, dry-run/import/rollback, 8 plugin extension points. RLS on all tables. Gated behind plugins feature flag. |
| 20 | **SIPmem evidence engine** | 14 typed memory categories, Fact Ledger (atomic claims), TemporalResolver (valid_from/to/stale), VerificationEngine (pluggable verifiers), 10 retrieval recipes, ContradictionRecord persistence, SipmemPipeline orchestration. 50 tests. |
| 21 | **UI Plugin Platform** | Manifest contract: UiCompatibility (native/compatible/standalone), UiThemeConfig, UiRouteDef, UiActionDef. Design tokens v1.0 (30+ CSS custom properties, dark mode, density modes). 11 shared SIP UI components (SipPage, SipCard, SipButton, SipBadge, SipAlert, etc.). /api/v1/ui/theme endpoint. Example native plugin manifest. |

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
├── Cargo.toml                    # Rust workspace (21 crates)
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
│   ├── sipmem-core/              # SIPmem types, traits, recipes (14 memory categories)
│   ├── sipmem-adapters/          # SIPmem retrievers, pipeline, verifiers
│   ├── sip-plugins/              # Plugin registry, manifest validation
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
│   ├── 004_seed_data.sql         # Demo data (org, 8 users, 35 assets, etc.)
│   ├── 005_fix_schema_issues.sql # Document enum fix, asset version, outbox RLS
│   ├── 006_sipmem_tables.sql     # Verification traces, feedback, enum extensions
│   ├── 007_migration_core.sql    # Migration job/run tables
│   ├── 008_convert_enums_to_text.sql # Convert 31 PG enums to TEXT for sqlx compat
│   └── 009_fix_certifications_type.sql # JSONB[] -> JSONB column type fix
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

## SIPmem — Verified Hybrid Temporal Memory Engine

SIPmem is SIP's memory system for physical assets and service operations. It is not a RAG system bolted onto a database. It is an **evidence engine** built from first principles.

### Architecture

```
Query → Auth Gate → Recipe Selection → Hybrid Retrieval → Evidence Scoring
  → Claim Extraction → Verification → Contradiction Detection → Synthesize
```

Evidence is NOT truth. Every retrieved record is a **candidate** that must survive verification before it can be used in an answer.

### Typed Memory (14 Categories)

SIPmem does not store generic "documents." Every memory entry has an explicit type:

| Category | Example Content |
|----------|----------------|
| Asset | Asset record, serial number, specification, criticality |
| ServiceCase | Work order, service ticket, repair request |
| TechnicianNote | Resolution notes, field observations, troubleshooting steps |
| ManualSection | Excerpt from maintenance/procedure manuals |
| ServiceBulletin | OEM bulletin, recall notice, advisory |
| FirmwareFact | Software/firmware version, update history |
| PartReplacement | Part swapped, replacement date, reason |
| KnownIssue | Pattern of failures across similar assets |
| Procedure | Step-by-step repair/maintenance procedure |
| FailureMode | How an asset class fails |
| RootCause | Why a specific failure occurred |
| TelemetrySummary | Aggregated sensor/telemetry insight |
| VerifiedFact | Atomic claim that passed verification |
| ContradictionRecord | Documented conflict between sources |

### Fact Ledger

All knowledge reduces to **atomic facts.** Each fact tracks:
- What entity it describes (asset, work order, part)
- When it was observed vs. when it was recorded
- Its validity window (valid_from → valid_to)
- Which evidence supports it
- Whether it has been verified or contradicted

### Retrieval Router & Recipes

Queries are routed to one of 10 **retrieval recipes**, each defining which retrievers to use, what verification level is needed, what temporal constraints apply, and the cost budget.

| Recipe | Retriever Mix | Verification | Example Query |
|--------|--------------|-------------|---------------|
| Exact Fact Lookup | SQL | Basic | "What is the serial number?" |
| State at Time | SQL + Temporal | Standard | "What was the status on June 1?" |
| Asset History Summary | SQL + Temporal + Graph | Standard | "Summarize the service history" |
| Troubleshooting Similarity | Vector + SQL | Standard | "Have we seen this symptom before?" |
| Known Issue Investigation | Vector + Graph | Strict | "Is this a known failure pattern?" |
| Firmware Specific | SQL | Basic | "What firmware version is installed?" |
| Service Bulletin Check | Vector + SQL | Standard | "Are there active bulletins?" |
| Contradiction Detection | All | Full | "Do any sources conflict?" |
| Root Cause Candidate | Graph + Temporal + Vector | Strict | "What is the most likely cause?" |
| Stale Evidence Detection | Temporal + SQL | Basic | "What facts are out of date?" |

### Pipeline (Rust-native)

The `SipmemPipeline` (in `sipmem-adapters`) orchestrates the full flow. The pipeline is assembled from pluggable components implementing these traits:

| Trait | Crate | Purpose |
|-------|-------|---------|
| `Retriever` | `sipmem-core` | Retrieves candidate evidence (SQL, Vector, Graph, Temporal) |
| `MemoryRouter` | `sipmem-core` | Selects the best retrieval recipe for a query |
| `Verifier` | `sipmem-core` | Cross-references claims against evidence, detects contradictions |
| `EvidenceScorer` | `sipmem-core` | Scores evidence relevance and confidence |

📖 **SIPmem Docs:**
- [Architecture](./docs/sipmem/architecture.md) — full pipeline design
- [Verification Loop](./docs/sipmem/verification-loop.md) — evidence engine philosophy
- [Temporal Memory](./docs/sipmem/temporal-memory.md) — time-scoped facts and pruning
- [Retrieval Recipes](./docs/sipmem/retrieval-recipes.md) — recipe selection and custom recipes
- [Plugin Contracts](./docs/sipmem/plugin-contracts.md) — implementing custom retrievers and verifiers

### Principles

| # | Principle |
|---|-----------|
| **P1** | Evidence is not truth. Every record must be verified before use. |
| **P2** | Typed memory beats generic documents. Categories constrain what can be retrieved. |
| **P3** | Time is a first-class dimension. Every fact has a validity window. |
| **P4** | Contradictions are data, not errors. Conflicts are persisted and tracked. |
| **P5** | The pipeline is pluggable. Retrievers, verifiers, and scorers are traits.

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

For developers, SIP provides the **harness**: canonical service objects, permissioned APIs, event-driven workflows, structured memory, plugin boundaries, and documentation patterns. Build new interfaces, automations, integrations, and AI capabilities without rebuilding the service intelligence foundation from scratch.

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
| `SIP__DATABASE_URL` | `postgres://sip:sip@localhost:5432/sip` | Yes |
| `SIP__SERVER__HOST` | `0.0.0.0` | Yes |
| `SIP__SERVER__PORT` | `8000` | Yes |
| `SIP__AUTH__JWT_SECRET` | (required, min 32 chars) | **Change for production** |
| `SIP__REDIS_URL` | `redis://localhost:6379` | Yes |
| `SIP__OBJECT_STORAGE__ENDPOINT` | `http://localhost:9000` | Yes |
| `SIP__OBJECT_STORAGE__BUCKET` | `sip-documents` | Yes |
| `SIP__AI__OLLAMA__URL` | `http://localhost:11434` | Yes |
| `SIP__AI__OLLAMA__MODEL` | `llama3.1:8b` | Yes |
| `NEXT_PUBLIC_API_URL` | `http://localhost:8000/api/v1` | Yes |

> **Note:** SIP uses `__` (double underscore) as a nesting separator for environment variables. `SIP__AUTH__JWT_SECRET` maps to `auth.jwt_secret` in the nested config. See [`Sip.toml`](./Sip.toml) for a TOML-based config file alternative.

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

## Documentation

| Document | Description |
|----------|-------------|
| [PROGRESS.md](./docs/PROGRESS.md) | Full PRD implementation tracker |
| [PRD-v2.md](./docs/PRD-v2.md) | Product Requirements Document v5.7 |
| [openapi.yaml](./docs/openapi.yaml) | OpenAPI 3.0 specification |
| **Plugin Engine** | |
| [Plugin Overview](./docs/plugins/overview.md) | Plugin architecture, lifecycle, types |
| [Manifest Reference](./docs/plugins/manifest-reference.md) | Complete `plugin.toml` field reference |
| [Extension Points](./docs/plugins/extension-points.md) | 35 UI injection slots + 8 migration slots |
| [Plugin Examples](./docs/plugins/examples.md) | 5 walkthroughs for common plugin types |
| [Plugin API](./docs/plugins/api-reference.md) | Plugin discovery REST endpoints |
| [UI Plugins](./docs/plugins/ui-plugins.md) | Building SIP-native UI plugins |
| [Theme Contract](./docs/plugins/theme-contract.md) | Design tokens, dark mode, density |
| [Navigation](./docs/plugins/navigation.md) | Plugin-driven sidebar navigation |
| [UI Checklist](./docs/plugins/sip-native-ui-checklist.md) | Pre-ship checklist for UI plugins |
| **SIPmem** | |
| [SIPmem Architecture](./docs/sipmem/architecture.md) | Evidence engine pipeline |
| [Verification Loop](./docs/sipmem/verification-loop.md) | Claim verification + contradiction detection |
| [Temporal Memory](./docs/sipmem/temporal-memory.md) | Time-scoped facts + pruning |
| [Retrieval Recipes](./docs/sipmem/retrieval-recipes.md) | 10 query recipes + router |
| [Plugin Contracts](./docs/sipmem/plugin-contracts.md) | Implementing retrievers + verifiers |
| **Migration** | |
| [Migration Overview](./docs/migrations/overview.md) | Migration Core Framework overview |
| [Migration API](./docs/migrations/api-reference.md) | Migration REST endpoints |
| [External ID Mapping](./docs/migrations/external-id-mapping.md) | Source→SIP traceability |
| [Migration Studio](./plugins/sip-migration-studio/README.md) | Reference hybrid import plugin |
| [Transform Functions](./plugins/sip-migration-studio/docs/transform-functions.md) | 14 field transform functions |

## License

**GNU Affero General Public License v3.0** — see [LICENSE](./LICENSE).

SIP is open source. Users can self-host forever. Data portability is guaranteed. Every feature in the hosted version is available in the open-source version.
