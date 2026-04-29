# SIP (Service Intelligence Platform) — Product Requirements Document

**Version:** 5.0
**Status:** Draft
**Date:** 2026-04-29
**Type:** Greenfield, Open Source (AGPLv3), AI-First Service Intelligence Platform for Physical Assets

---

## 1. Executive Summary

**SIP** is an AI-first service intelligence platform for physical assets. It builds a **maintenance intelligence layer** on top of a **service-domain canonical model** — a structured, queryable representation of every asset, every work order, and every repair decision an organization has ever made. Unlike legacy maintenance platforms that treat operational data as a passive ledger, SIP structures it as an **AI-readable operational history** — a **structured maintenance substrate** that both humans and AI agents consume through the same public API.

**Core thesis:** Maintenance data should be structured for machine reasoning first, human consumption second. Every entity in SIP exposes **agent-accessible service data** through a machine-readable interface that both the UI and AI agents consume through the same public API.

**Business model:** Open-source core (AGPLv3) with optional enterprise hosting and support. The **service-domain canonical model** is the moat — competitors can copy features but cannot replicate a coherent, cross-industry domain model evolved through real-world usage. Plugins extend the platform into domain-specific territory without compromising the core model.

**MVP scope:** Organization and tenant model, user model and basic RBAC, location hierarchy, AssetType with JSON Schema custom attributes, asset registry with manufacturer and model support, work order creation/assignment/lifecycle/history, activity audit log, basic document upload and linking, and limited AI Q&A over asset and work order history with citations from source records. Shippable as Docker Compose local deployment with initial seed data.

**Phased roadmap:** MVP Core → AI and Plugin Expansion → Core Operations → Advanced AI → Enterprise Scale. Each phase adds capabilities without compromising the **service-domain canonical model** that serves as the platform's foundation.

---

## 2. Problem and Market Context

### 2.1 Problem Statement
Organizations that depend on physical assets for revenue lack a unified **maintenance intelligence layer**. Existing maintenance platforms are rigid monoliths locked to single industries. They treat operational data as a passive ledger — not as an **AI-readable operational history** that could power proactive decisions, failure prediction, and autonomous service workflows.

**SIP must solve:** How do you build a **physical asset intelligence** platform that works for *any* physical asset — from a robotic arm to a dishwasher to a lawn — and structures every service event into an **agent-accessible** format consumable by both humans and AI agents?

### 2.2 Market Landscape

| Category | Examples | Gaps SIP Exploits |
|----------|---------|-------------------|
| **Enterprise Platforms** | IBM Maximo, SAP PM, Oracle EAM | Expensive, on-premise, zero AI integration |
| **Mid-Market Platforms** | Fiix, MaintainX, UpKeep | Single-industry focus, closed-source, AI is bolt-on marketing |
| **Open-Source Alternatives** | Fracttal (freemium), openMAINT | No AI, no plugin ecosystem, limited adoption |
| **Horizontal Tools** | Jira, ServiceNow, Monday.com | Not maintenance-native; work orders need the asset context |

**Market size:** The market for maintenance and service software is $1.2B in 2025, growing at 9% CAGR toward $2.3B by 2032. No dominant open-source player exists. The **physical asset intelligence** category — AI-first platforms that treat service data as a reasoning substrate — is unoccupied.

### 2.3 Current Limitations of Existing Maintenance Platforms
1. **Single-industry schemas** — A manufacturing system can't model an apartment complex. A fleet platform can't handle kitchen equipment.
2. **No AI-native data layer** — Data is stored for PDF reports, not for embedding, vector search, or LLM context windows. There is no **structured maintenance substrate** that an AI agent can reason over.
3. **No plugin ecosystem** — Extending the system means vendor professional services or forking.
4. **Human-only interfaces** — No MCP endpoints, no function-calling tools, no agentic interfaces. **Agent-accessible service data** is an afterthought at best.
5. **Closed-source lock-in** — Maintenance data outlives the software vendor. Organizations risk data hostage situations. An open-source **service-domain canonical model** guarantees data portability.

---

## 3. Product Principles

These principles govern every architectural and product decision in SIP. They are ranked by priority — when two principles conflict, the higher-ranked principle wins.

| # | Principle | What It Means |
|---|-----------|---------------|
| P1 | **Machine-readable before human-readable** | Every entity must expose a structured API contract before any UI is built. The API is the canonical representation of data. UIs are consumers of the API — one client among many. If a human can see it, an AI must be able to query it. |
| P2 | **The public API is the AI surface** | AI agents consume the same public REST API as the frontend. There is no internal AI backdoor, no separate AI-only endpoints, no direct database access for LLMs. Every AI retrieval goes through the same authorization, rate limiting, and audit logging as a human API call. AI is not a privileged user — it is an authenticated, authorized, audited user. |
| P3 | **Every AI answer must be grounded** | Every AI response about customer data must cite retrievable source records. The AI must never speculate beyond what exists in the database. If the answer is not in the context, the AI says "I don't have enough information." Responses include citations: work order IDs, document names, timestamps. This is not optional — it is the defining quality bar for the AI layer. |
| P4 | **AI retrieval enforces authorization boundaries** | When the AI retrieves context for a query, it must enforce the same tenant isolation and RBAC permissions as direct API access. An AI query from a technician must never surface data from another organization or data the technician lacks permission to see. The RAG pipeline queries through the same service layer — not raw database queries. |
| P5 | **Operational actions create reusable intelligence** | Every completed work order, every inspection finding, every resolution note becomes part of an **AI-readable operational history**. When a technician writes "replaced bearing, found inner race spalling due to contamination," that note is immediately available as retrievable intelligence for every future query about that asset type. The platform treats maintenance data as a compounding asset — a **structured maintenance substrate** that grows more valuable with every service event. |
| P6 | **The service-domain canonical model is the moat** | The service-domain model (entities, relationships, events, state machines) is the platform's defensible intellectual property. It encodes decades of cross-industry maintenance knowledge into a coherent, queryable structure. Plugins extend the model — they do not bypass it. Competitors can copy features; they cannot copy a well-reasoned service-domain model evolved through real-world usage. |
| P7 | **Plugins extend without compromising** | Plugins must extend SIP without compromising tenant isolation, system reliability, or auditability. A plugin crash must not crash the core. A plugin's data access must be scoped and auditable. A plugin's AI tools must go through the same authorization gates as core AI tools. If a plugin degrades these guarantees, it is rejected. |
| P8 | **Modular monolith until scale demands otherwise** | SIP starts as a single deployable unit with well-defined module boundaries. Services are extracted only when there is a measurable bottleneck — not because the diagram looks better. Module boundaries are enforced by interface contracts, not network boundaries. This keeps operational complexity low while preserving the option to split later. |
| P9 | **Modular at the data layer** | Asset types are schemas, not hardcoded tables. Any physical object can be modeled without code changes. The system ships with reference types, but the schema lives in the database, not in the codebase. |
| P10 | **Audit everything, immutably** | Every state change, every AI action, every human decision is logged in an append-only Activity table. The audit trail is the system of record. Nothing is ever deleted — only marked as superseded or deactivated. |
| P11 | **Open source as a feature, not a marketing tactic** | Users can self-host forever. Data portability is guaranteed. The business model is hosting + support, not data lock-in. Every feature usable in the hosted version must be available in the open-source version, with the exception of operational concerns (SLA guarantees, managed backups, SSO integration). |
| P12 | **Progressive AI autonomy** | AI starts advisory (read-only Q&A), graduates to assisted actions (confirmed by human), and eventually reaches autonomous operation — all with human overrides. At no point does the AI operate without an audit trail or a kill switch.

---

## 4. Users and Personas

### 4.1 Human Personas

#### P1: Maria — Maintenance Technician
> *"I need to know what I'm walking into before I get there."*

- **Context:** Field technician at a facility management company. Services 20-30 assets per week across multiple sites. Often in basements or rooftops with poor connectivity.
- **Goals:** See assigned work immediately, access asset history and manuals on mobile, log completion with photos in under 60 seconds.
- **Pain points:** Switching between paper work orders, PDF manuals, and a desktop-only maintenance system. No way to query "what fixed this last time?" without calling a senior tech. No **agent-accessible service data** — every lookup is manual.
- **Core actions in SIP:** View work queue → Accept assignment → Access AI for asset history → Execute checklist → Attach photos → Mark complete.

#### P2: David — Asset/Facility Manager
> *"I need to stop fighting fires and start preventing them."*

- **Context:** Manages 500+ assets across 3 facilities. Responsible for uptime KPIs, maintenance budget, and compliance.
- **Goals:** See asset health at a glance, schedule preventive maintenance intelligently, know which assets are at risk of failure before they break.
- **Pain points:** Spreadsheet-based scheduling, no cross-asset trend analysis, reactive maintenance culture driven by lack of visibility.
- **Core actions in SIP:** View asset health dashboard → Approve AI-recommended schedule adjustments → Prioritize backlog by criticality → Review compliance gaps.

#### P3: James — Third-Party Service Vendor
> *"I service 10 different clients. I need to prove I did the work and get paid."*

- **Context:** Independent contractor servicing HVAC systems for multiple building owners.
- **Goals:** Receive work orders from clients in one place, document completed work with timestamps and photos, generate compliance evidence packages.
- **Pain points:** Each client uses a different system (or no system). Duplicate data entry. Hard to prove maintenance was done on schedule.
- **Core actions in SIP:** Accept work order across tenant boundary → Execute work → Upload evidence → Generate service report.

#### P4: Sarah — Compliance Officer
> *"I need to prove to auditors that every inspection was done, on time, by a qualified person."*

- **Context:** Responsible for FDA/OSHA/ISO compliance at a food processing plant.
- **Goals:** Pull audit-ready evidence packages on demand, verify technician certifications are current, identify compliance gaps before auditors do.
- **Pain points:** Paper records, missing signatures, expired certifications filed away in a drawer. Audit prep takes weeks.
- **Core actions in SIP:** Query audit log by asset/date range → Verify certification validity at time of work → Export compliance report → Flag gaps for remediation.

### 4.2 AI Agent Personas

| Agent | Capability | Autonomy | Example Query | Ships In |
|-------|-----------|----------|---------------|----------|
| **Knowledge Agent** | Q&A over asset history, work orders, documents | Read-only | *"What's the maintenance history of Pump A?"* | Phase 1 |
| **Scheduling Agent** | Optimizes PM schedules, detects conflicts | Assisted | *"Reschedule next week to balance technician load."* | Phase 3 |
| **Diagnostic Agent** | Predicts failures from patterns, recommends fixes | Advisory | *"Compressor B shows bearing wear pattern — suggest inspection."* | Phase 3 |
| **Compliance Agent** | Gap analysis, evidence packaging | Advisory | *"Are we audit-ready for ISO 55000 section 7.1?"* | Phase 3 |
| **Dispatch Agent** | Auto-assigns work orders by skills, location, availability | Semi-auto | *"Assign the next corrective WO to the nearest qualified tech."* | Phase 4 |

### 4.3 AI as a First-Class User — Formal Rules
1. Each AI agent type has an **AgentIdentity record** defining its type, provider, model, and permission scope.
2. Every AI action is **logged in the Activity table** with `actor_type: AI_AGENT` and a reference to the specific `agent_identity_id`.
3. AI agents **cannot escalate their own permissions**. They operate within the same RBAC framework as humans.
4. For MVP, AI agents may draft mutations (create work order, assign technician) but may not execute high-impact mutations without human confirmation. Full tool execution ships in Phase 2. Autonomous mutation (no human in the loop) is gated behind Phase 4 trust and safety validation.
5. AI agents query through the **same public API** — they have no internal database access, no bypass of RLS, and no privileged data paths.

---

## 5. Core Use Cases

### UC1: Register a New Asset Type
1. Admin navigates to Asset Types → Create New.
2. Defines name ("Industrial Robot Arm"), category ("ROBOTICS"), and JSON Schema for custom attributes (`{ "payload_kg": "number", "axes": "number", "controller_model": "string" }`).
3. Optionally attaches default PM schedule templates and inspection checklists.
4. Asset Type is immediately available for asset creation. No code deployment needed.

### UC2: Create a Preventive Maintenance Schedule
1. Manager selects an asset → Schedules → Create.
2. Chooses trigger type: time-based (`0 8 * * MON`) or meter-based (every 500 operating hours).
3. Defines work order template: title, description, estimated hours, default assignee, parts list.
4. System calculates `next_due` timestamp.
5. When trigger fires, system generates a new WorkOrder from the template.

### UC3: Execute a Work Order (Technician)
1. Technician opens mobile view → sees assigned work orders sorted by priority and due date.
2. Taps a work order → sees asset details, location, prior work history (via AI summary), attached manuals.
3. Travels to asset → taps "Start Work" → timer begins.
4. Executes checklist items (pass/fail, numeric readings, photos).
5. Logs parts used (decrements inventory).
6. Writes resolution notes → taps "Complete" → system logs elapsed time, updates asset status, recalculates schedule `next_due`.

### UC4: AI Query — Asset History
1. Technician types or speaks: *"What's been going on with Conveyor C-12?"*
2. AI Service retrieves context:
   - **Relational:** Current status, recent work orders (last 90 days), open work orders.
   - **Vector:** Semantic search over work order resolution notes for similar issues.
   - **Graph:** Parent/child asset relationships, functional dependencies.
3. LLM generates a structured response with citations: *"Conveyor C-12 is OPERATIONAL. Last 3 work orders: #452 (belt replaced, Apr 10), #389 (motor overheating, Mar 22 — root cause: dust accumulation), #301 (bearing lubrication, Mar 1). The motor overheating is a recurring pattern. Would you like me to suggest a preventive schedule?"*

### UC5: Inspection with Automatic Corrective Work Order
1. Technician executes an inspection work order with checklist items.
2. One checklist item returns `result: FAIL` with finding: *"Belt tension at 30 PSI — spec is 45-55 PSI."*
3. On inspection completion, system detects the failure and auto-generates a CORRECTIVE work order linked to the finding.
4. Manager is notified of the new corrective work order.

### UC6: Install and Configure a Plugin
1. Admin navigates to Plugins → Install.
2. Provides plugin package (Docker image + manifest) or selects from registry.
3. System validates manifest, checks compatibility with current SIP version, verifies declared permissions.
4. Plugin starts as a sidecar container. Registers its extension points (API routes, AI tools, event subscribers).
5. Admin configures plugin-specific settings via the manifest's `config_schema`.

---

## 6. MVP Scope

**Phase 1: MVP Core ships** with the foundational data model, asset and work order management, basic AI Q&A, and Docker Compose deployment. Everything else is deferred to a later phase.

### 6.1 In Scope for MVP

| # | Capability | Details |
|---|-----------|---------|
| 1 | **Organization and tenant model** | Organization CRUD. Slug-based tenancy. RLS isolation. Tenant settings (timezone, default currency, feature flags). |
| 2 | **User model and basic RBAC** | User CRUD with 6 roles: ADMIN, MANAGER, TECHNICIAN, VIEWER, VENDOR, AUDITOR. JWT authentication. Skills and certifications with expiry tracking. Team management with lead assignment. |
| 3 | **Location hierarchy** | Hierarchical locations (Site → Building → Floor → Room). CRUD with parent/child relationships. Geo coordinates support. |
| 4 | **AssetType with JSON Schema custom attributes** | Define asset types with custom attribute schemas (JSON Schema draft-2020-12). 6 reference types shipped as seed data. Schema validation on asset create/update. |
| 5 | **Asset registry** | Full CRUD for assets. Dynamic attributes validated against AssetType schema. Parent/child hierarchy. Status lifecycle with state machine validation. Tagging. Location assignment. |
| 6 | **Manufacturer and AssetModel support** | Manufacturer CRUD (system defaults + tenant-created). AssetModel CRUD linking manufacturer to AssetType. Asset creation pre-populates type and attributes when a model is selected. Seed data with known manufacturers and models for reference AssetTypes. |
| 7 | **Work order creation, assignment, lifecycle, and history** | Full lifecycle state machine (OPEN → ASSIGNED → IN_PROGRESS → ON_HOLD → COMPLETED/CANCELLED). Types: PREVENTIVE, CORRECTIVE, INSPECTION, EMERGENCY. Priority levels. User/team assignment. Time tracking (estimated vs actual). Resolution notes (required on completion). Parts consumption logging. Source tracking (MANUAL, SCHEDULE, API). |
| 8 | **Preventive maintenance scheduling** | Time-based cron schedules. Auto-generate work orders on trigger. Deduplication (skip if open WO already exists). Enable/disable. Next due calculation. |
| 9 | **Minimal inspections** | Work orders may have checklist items. Items support PASS_FAIL, NUMERIC, TEXT, and PHOTO metadata. Failed checklist items are recorded. Automatic corrective work order generation from failed inspection items is deferred to Phase 1.5 automation support. |
| 10 | **Activity audit log** | Immutable append-only record of all state changes. Tracks actor (human, AI agent, system, plugin), entity, action, before/after diff. Filterable by entity, date range, actor. |
| 11 | **Basic document upload and linking** | Upload documents (PDF, images) to MinIO/S3. Link to assets and work orders via DocumentLink. Text extraction for AI indexing. |
| 12 | **Document processing status** | Track document processing lifecycle: PENDING → EXTRACTING → EXTRACTED → CHUNKING → EMBEDDING → INDEXED (or FAILED). API exposes status so UI can show "this document is still being indexed." |
| 13 | **Limited AI Q&A over asset and work order history** | Natural language queries via POST /api/v1/ai/chat with SSE streaming. Answers grounded in relational data + vector search over work order notes and document text. Read-only in MVP — no tool execution for mutations. |
| 14 | **AI citations from source records** | Every AI response includes source citations: work order ID, document name, timestamp. "I don't know" response when answer is not in context. |
| 15 | **Minimal parts inventory** | Part CRUD. Manual part usage logging against a work order. Atomic decrement of quantity_on_hand. Low stock event emission exists internally, but notification, reorder recommendation, and automation handling are deferred to Phase 1.5 or Phase 2. |
| 16 | **Docker Compose local deployment** | Single `docker compose up` command. Includes: SIP API, SIP Frontend, PostgreSQL + pgvector, Redis, MinIO, Ollama (for local AI). |
| 16 | **Initial seed data** | Docker image ships with 6 reference AssetTypes, known Manufacturers and AssetModels for each type, a demo organization, sample assets, and sample work orders to demonstrate the platform immediately after startup. |

### 6.2 MVP Technical Stack

| Layer | Choice | Rationale |
|-------|--------|-----------|
| Backend | Python (FastAPI) | AI/ML ecosystem, async performance, type safety |
| Frontend | TypeScript (Next.js 15) | React ecosystem, SSR, PWA-capable |
| Relational DB | PostgreSQL 16+ | pgvector, RLS, JSONB, ltree, PostGIS |
| Vector DB | pgvector | Stack consolidation, no separate service needed for MVP |
| Graph | Recursive CTEs + ltree materialized paths | Defer Neo4j to Phase 4 |
| Event Bus | In-process outbox poller | Single-node simplicity for MVP; defer NATS to Phase 1.5 |
| Cache | Redis/Valkey | Session store, rate limiting |
| Object Storage | MinIO (self-hosted) | |
| LLM | Ollama (default local provider) | MVP includes a minimal LLM adapter layer. LiteLLM may be used internally to normalize provider calls, but per-tenant provider configuration, fallback chains, token usage dashboards, and provider swapping through the UI are deferred to Phase 1.5. |
| Containerization | Docker Compose | Single-command deployment |

### 6.3 MVP API Endpoints

```
POST   /api/v1/auth/login
POST   /api/v1/auth/refresh
POST   /api/v1/auth/logout

GET    /api/v1/organizations
POST   /api/v1/organizations
GET    /api/v1/organizations/{id}
PATCH  /api/v1/organizations/{id}

GET    /api/v1/locations
POST   /api/v1/locations
GET    /api/v1/locations/{id}
PATCH  /api/v1/locations/{id}
GET    /api/v1/locations/{id}/children
GET    /api/v1/locations/{id}/assets

GET    /api/v1/asset-types
POST   /api/v1/asset-types
GET    /api/v1/asset-types/{id}
PATCH  /api/v1/asset-types/{id}

GET    /api/v1/manufacturers
POST   /api/v1/manufacturers
GET    /api/v1/manufacturers/{id}
GET    /api/v1/manufacturers/{id}/models

GET    /api/v1/models
POST   /api/v1/models
GET    /api/v1/models/{id}

GET    /api/v1/assets
POST   /api/v1/assets
GET    /api/v1/assets/{id}
PATCH  /api/v1/assets/{id}
DELETE /api/v1/assets/{id}
GET    /api/v1/assets/{id}/children
GET    /api/v1/assets/{id}/work-orders

GET    /api/v1/work-orders
POST   /api/v1/work-orders
GET    /api/v1/work-orders/{id}
PATCH  /api/v1/work-orders/{id}
DELETE /api/v1/work-orders/{id}
GET    /api/v1/work-orders/{id}/assignments
POST   /api/v1/work-orders/{id}/assignments
PATCH  /api/v1/work-orders/{id}/assignments/{assignment_id}
POST   /api/v1/work-orders/{id}/parts

GET    /api/v1/schedules
POST   /api/v1/schedules
GET    /api/v1/schedules/{id}
PATCH  /api/v1/schedules/{id}
DELETE /api/v1/schedules/{id}

GET    /api/v1/inspections
GET    /api/v1/inspections/{id}
PATCH  /api/v1/inspections/{id}/items

GET    /api/v1/parts
POST   /api/v1/parts
GET    /api/v1/parts/{id}
PATCH  /api/v1/parts/{id}

GET    /api/v1/users
POST   /api/v1/users
GET    /api/v1/users/{id}
PATCH  /api/v1/users/{id}

GET    /api/v1/teams
POST   /api/v1/teams
GET    /api/v1/teams/{id}
PATCH  /api/v1/teams/{id}

POST   /api/v1/documents
GET    /api/v1/documents/{id}
DELETE /api/v1/documents/{id}

GET    /api/v1/activities

POST   /api/v1/ai/chat              (SSE streaming response)
GET    /api/v1/ai/conversations

GET    /api/v1/health
GET    /api/v1/health/ready
```

### 6.4 MVP Seed Data

The Docker image ships with a complete demo dataset so the system is immediately usable:

| Seed Data | Contents |
|-----------|----------|
| **Reference AssetTypes** | Pump, Motor, Conveyor, HVAC Unit, Vehicle, Generic Equipment (with JSON schemas, default PM templates, default inspection checklists) |
| **Manufacturers** | Fictional reference manufacturers (e.g., "Acme Industrial", "Northstar Motors", "Summit HVAC", "Vector Conveyance", "Atlas Fleet Vehicles") — each with 2-5 models. Real manufacturer catalogs can be added as community-maintained data in a separate repository. |
| **AssetModels** | 25+ models across all manufacturers, each linked to the appropriate AssetType |
| **Demo Organization** | "Acme Manufacturing" with sample locations, 30+ assets, 8 users across all roles, 2 teams |
| **Demo Work Orders** | 50+ work orders in various states (some completed with resolution notes, some open, some in progress) for immediate AI query testing |

---

## 7. Out of Scope for MVP

These capabilities are intentionally deferred. Each has a target phase where it ships.

| Feature | Target Phase | Rationale |
|---------|-------------|-----------|
| Plugin manifest and lifecycle foundation | Phase 1.5 | Core data model must stabilize before extension points are meaningful |
| Reference plugins (CSV import, dashboard) | Phase 1.5 | Depends on plugin framework |
| Basic UI extension points | Phase 1.5 | Depends on plugin framework |
| Per-tenant LLM provider config, fallback chains, token dashboards | Phase 1.5 | Minimal adapter with Ollama default is sufficient for MVP |
| Improved RAG pipeline (re-ranking, hybrid search) | Phase 1.5 | Basic vector search sufficient for MVP Q&A |
| Document chunking and embedding refinements | Phase 1.5 | MVP embeds per-document; chunking for large docs in 1.5 |
| Optional external event bus (NATS JetStream) | Phase 1.5 | In-process outbox is sufficient for single-node MVP |
| Basic automation engine | Phase 1.5 | Time-based scheduling is in MVP; event-based automation in 1.5 |
| Meter-based scheduling (runtime/cycle triggers) | Phase 2 | Requires meter reading API |
| Parts/inventory management (full) | Phase 2 | Basic CRUD only in MVP |
| Notification service (email, push, webhook) | Phase 2 | |
| Dashboard & analytics (MTTR, MTBF) | Phase 2 | |
| AI tool execution (create WO, update status with confirmation) | Phase 2 | Read-only AI in MVP |
| Predictive maintenance ML models | Phase 3 | Requires training data accumulation |
| Smart scheduling optimization | Phase 3 | |
| Compliance agent (ISO/OSHA/FDA) | Phase 3 | |
| Full plugin marketplace | Phase 4 | Community must build plugins first |
| Advanced plugin sandboxing (WASM, vulnerability scanning) | Phase 4 | |
| Multi-modal AI (inspection photo analysis) | Phase 4 | Requires separate ML pipeline |
| Native mobile applications | Phase 4 | PWA covers mobile needs until Phase 4 |
| Advanced offline-first technician experience | Phase 4 | |
| Multi-organization vendor portal | Phase 4 | Complex cross-tenant auth |
| Dedicated graph database (Neo4j) | Phase 4 | Recursive CTEs sufficient through Phase 3; the full knowledge graph becomes necessary when multi-hop impact analysis moves from nice-to-have to enterprise requirement |
| Enterprise billing and hosted SaaS control plane | Phase 4 | |
| Advanced regulatory compliance packages | Phase 4 | Plugin territory |
| Cross-tenant shared analytics | Phase 4 | |
| Autonomous AI mutations (no human confirmation) | Phase 4 | Gated behind trust and safety validation; the **auditable AI-assisted maintenance workflows** of Phases 1-3 build the evidence base |
| Enterprise SSO (SAML/OIDC) | Phase 4 | |
| Federation / cross-org data sharing | Phase 4 | Requires the **service-domain canonical model** to be proven at single-tenant scale before multi-org complexity |

---

## 8. Service-Domain Canonical Model

The **service-domain canonical model** is the platform's foundation. It is a cross-industry, AI-consumable representation of physical assets, their maintenance history, and the people and processes that service them. Every entity, relationship, event, and state machine defined here is available to both humans and AI agents through the same public API.

### 8.1 Core Entities

#### Organization
Top-level tenancy boundary. All data scoped to exactly one Organization.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| name | string | Required, max 255 |
| slug | string | Required, unique, URL-safe, immutable after creation |
| settings | JSONB | Tenant-level configuration defaults |
| created_at | timestamptz | Auto-set |
| updated_at | timestamptz | Auto-set |

#### Location
Physical or logical site. Supports hierarchy (Site → Building → Floor → Room).

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| parent_id | UUID (FK, self) | Null for top level |
| name | string | Required |
| type | enum | SITE, BUILDING, FLOOR, ROOM, AREA, OTHER |
| geo | GEOGRAPHY(POINT) | Nullable lat/lon |
| metadata | JSONB | Free-form location attributes |
| created_at | timestamptz | |
| updated_at | timestamptz | |

#### Asset
**The core entity.** Any physical thing requiring maintenance.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| location_id | UUID (FK) | Nullable |
| parent_id | UUID (FK, self) | Null for top-level asset |
| asset_type_id | UUID (FK) | Required |
| model_id | UUID (FK → AssetModel) | Nullable, references the specific manufacturer model |
| name | string | Required |
| description | text | Nullable, AI-indexed |
| serial_number | string | Nullable, index for search |
| firmware_version | string | Nullable — critical for recalls, known issues, AI diagnostics |
| software_version | string | Nullable |
| hardware_revision | string | Nullable |
| status | enum | OPERATIONAL, DEGRADED, DOWN, MAINTENANCE, RETIRED |
| criticality | enum | LOW, MEDIUM, HIGH, CRITICAL |
| installed_date | date | Nullable |
| warranty_expiry | date | Nullable |
| attributes | JSONB | Validated against AssetType.schema |
| tags | text[] | GIN-indexed for search |
| metadata | JSONB | System-managed (e.g., `first_seen`, `last_maintenance_date`) |
| created_at | timestamptz | |
| updated_at | timestamptz | |

When a user creates an asset and selects a model, the `asset_type_id` and initial `attributes` are pre-populated from the AssetModel. The user can override any value.

SIP must support queries across manufacturer, model, revision, firmware version, software version, and hardware revision. This is critical for service intelligence, recalls, known issues, and AI-assisted troubleshooting.

**Status state machine:**
```
OPERATIONAL ──► DEGRADED ──► DOWN
     ▲              │           │
     │              ▼           │
     └────── MAINTENANCE ◄──────┘
                    │
                    ▼
                RETIRED (terminal state)
```

Transitions are validated: DEGRADED → RETIRED is illegal (must go through DOWN or MAINTENANCE first).

#### AssetType
Classification template. Defines the schema for Asset.attributes and default maintenance patterns.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Null = system default, shared across tenants |
| name | string | Required, e.g., "Industrial Robot" |
| category | string | Required, e.g., "ROBOTICS" |
| description | text | Nullable, AI-indexed |
| schema | JSONB | JSON Schema (draft-2020-12) for Asset.attributes |
| default_pm_schedules | JSONB[] | Array of schedule templates |
| default_inspection_template | JSONB | Checklist template |
| icon | string | Icon identifier for UI |
| is_system | boolean | True = shipped with SIP, cannot be deleted |
| created_at | timestamptz | |
| updated_at | timestamptz | |

#### Manufacturer
Represents a company that manufactures physical assets.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Null = system-level, shared across tenants |
| name | string | Required, e.g., "Siemens", "Carrier", "Fanuc" |
| website | string | Nullable |
| support_url | string | Nullable |
| created_at | timestamptz | |
| updated_at | timestamptz | |

#### AssetModel
A specific product model from a manufacturer. Templates an asset's make/model/variant.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Null = system-level, shared across tenants |
| manufacturer_id | UUID (FK) | Required |
| name | string | Required, e.g., "Simotics GP 1LE0", "AquaSnap 30RB" |
| model_number | string | Required, manufacturer's identifier |
| revision | string | Nullable |
| lifecycle_status | enum | ACTIVE, DEPRECATED, END_OF_SUPPORT, RETIRED |
| asset_type_id | UUID (FK) | Required — the AssetType this model maps to |
| documentation_url | string | Nullable |
| default_attributes | JSONB | Default values for Asset.attributes when this model is selected |
| metadata | JSONB | Extensible |
| created_at | timestamptz | |
| updated_at | timestamptz | |

#### WorkOrder
A unit of maintenance work. The primary action entity in the platform. Work orders power **auditable AI-assisted maintenance workflows** — every status transition, assignment, and completion is recorded immutably and available for AI reasoning.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| asset_id | UUID (FK) | Required |
| parent_id | UUID (FK, self) | Nullable, for sub-tasks |
| schedule_id | UUID (FK) | Nullable, set if generated by a schedule |
| type | enum | PREVENTIVE, CORRECTIVE, INSPECTION, EMERGENCY |
| priority | enum | LOW, MEDIUM, HIGH, CRITICAL |
| status | enum | DRAFT, OPEN, ASSIGNED, ACCEPTED, IN_PROGRESS, ON_HOLD, COMPLETED, REVIEWED, CLOSED, CANCELLED |
| title | string | Required, max 500 |
| description | text | Required, AI-indexed |
| scheduled_start | timestamptz | Nullable |
| scheduled_end | timestamptz | Nullable |
| actual_start | timestamptz | Set when status → IN_PROGRESS |
| actual_end | timestamptz | Set when status → COMPLETED |
| due_at | timestamptz | Nullable — target completion time for SLA tracking and overdue detection |
| estimated_hours | decimal(5,2) | Nullable |
| actual_hours | decimal(5,2) | Computed on completion from actual_start - actual_end |
| resolution_notes | text | Required on completion, AI-indexed |
| failure_code | string | Nullable, corrective/emergency only |
| root_cause | text | Nullable, AI-indexed |
| created_by_id | UUID (FK → User) | Required |
| source | enum | MANUAL, SCHEDULE, AI_AGENT, API |
| reopened_count | integer | Default 0 — tracks how many times this WO has been reopened |
| last_reopened_at | timestamptz | Nullable |
| last_reopened_by_id | UUID (FK → User) | Nullable |
| version | integer | Default 1 — optimistic locking |
| archived_at | timestamptz | Nullable — soft-delete |
| archived_by_id | UUID (FK → User) | Nullable |
| archive_reason | text | Nullable |
| sla_policy_id | UUID (FK → SLAPolicy) | Nullable — deferred to Phase 2 |
| response_due_at | timestamptz | Nullable — deferred to Phase 2 |
| resolution_due_at | timestamptz | Nullable — deferred to Phase 2 |
| metadata | JSONB | Extensible |
| created_at | timestamptz | |
| updated_at | timestamptz | |

**Status state machine:**
```
DRAFT ──► OPEN ──► ASSIGNED ──► ACCEPTED ──► IN_PROGRESS ──► COMPLETED ──► REVIEWED ──► CLOSED
  │        │         │             │            │                │              │          │
  │        │         │             │            ▼                │              │          │
  │        │         │             │         ON_HOLD ──► IN_PROGRESS             │          │
  │        │         │             │            │                                 │          │
  │        ▼         ▼             ▼            ▼                                 ▼          │
  └────── CANCELLED (any non-terminal state can be cancelled with reason)    reopen ◄──┘
                                                                                 │
                                                                                 └──► OPEN or IN_PROGRESS
```

**Status definitions:**
- **DRAFT:** Work order created but not yet ready to be actioned. Not visible in technician queues.
- **OPEN:** Ready for assignment. Visible in dispatch queues.
- **ASSIGNED:** A technician/team has been assigned. Awaiting acceptance.
- **ACCEPTED:** Technician acknowledged the assignment. Not yet started.
- **IN_PROGRESS:** Technician is actively working on the asset.
- **ON_HOLD:** Work paused (waiting for parts, weather, access). Can resume to IN_PROGRESS.
- **COMPLETED:** Technician believes the work is done. Resolution notes required. Awaiting review.
- **REVIEWED:** A manager has reviewed and accepted the completion.
- **CLOSED:** The business has accepted the work as complete. Terminal state for successful work.
- **CANCELLED:** Work order cancelled with reason in resolution_notes. Terminal state.

**Reopen behavior (action, not a status):**
When a CLOSED work order is reopened:
1. System validates the user has `work_order:reopen` permission.
2. System increments `reopened_count`.
3. System sets `last_reopened_at` and `last_reopened_by_id`.
4. System changes status from CLOSED to OPEN (or IN_PROGRESS if the work should resume immediately).
5. System emits `work_order.reopened` event.
6. Original completion, review, and closure Activity records are preserved unchanged.

Reopen may also apply to CANCELLED work orders in a future phase.

**Validation rules:**
- Only ASSIGNED, ACCEPTED, or IN_PROGRESS can transition to ON_HOLD.
- COMPLETED requires `resolution_notes` (non-empty).
- REVIEWED requires a reviewer identity (captured in Activity).
- CANCELLED requires a cancellation reason in `resolution_notes`.
- Every status transition creates an Activity record and a WorkOrderStatusHistory record.
- Status transitions are validated by a state machine.

**Delete/archive semantics:**
DELETE endpoints perform soft-delete only. They set `archived_at`, `archived_by_id`, and `archive_reason`. No tenant-owned operational record is hard-deleted through the public API. Archived records remain queryable by admins and auditors.

API endpoints for archive:
```
POST /api/v1/assets/{id}/archive
POST /api/v1/work-orders/{id}/cancel    (cancellation is the archive path for work orders)
POST /api/v1/documents/{id}/archive
```

**Common archive fields** (applied to Asset, Schedule, WorkOrder, Document, Automation, Plugin):
| Attribute | Type | Description |
|-----------|------|-------------|
| archived_at | timestamptz | Nullable, set on archive |
| archived_by_id | UUID (FK → User) | Nullable |
| archive_reason | text | Nullable |

Activity records are never archived or deleted.

#### WorkOrderStatusHistory
Query-optimized lifecycle table for reporting, timelines, SLA calculations, and AI context retrieval. Complementary to Activity (the general-purpose immutable audit log).

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| work_order_id | UUID (FK) | Required |
| from_status | enum | Nullable (null for initial creation) |
| to_status | enum | Required |
| changed_by_id | UUID (FK → User) | Nullable |
| actor_type | enum | HUMAN, AI_AGENT, SYSTEM, PLUGIN |
| agent_identity_id | UUID (FK → AgentIdentity) | Nullable |
| plugin_id | UUID (FK → Plugin) | Nullable |
| reason | text | Nullable |
| created_at | timestamptz | |

**Indexes:**
```sql
CREATE INDEX idx_wosh_work_order ON work_order_status_history (work_order_id, created_at DESC);
CREATE INDEX idx_wosh_status ON work_order_status_history (organization_id, to_status, created_at DESC);
```

#### SLAPolicy (Phase 2 — model reserved now)

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| name | string | Required |
| description | text | Nullable |
| applies_to_priority | enum | Nullable |
| applies_to_asset_criticality | enum | Nullable |
| response_time_minutes | integer | Nullable |
| resolution_time_minutes | integer | Nullable |
| business_hours_only | boolean | Default false |
| enabled | boolean | Default true |
| created_at | timestamptz | |
| updated_at | timestamptz | |

**MVP SLA support:** MVP supports simple `due_at` tracking and overdue detection. Full SLAPolicy engine is deferred to Phase 2.

#### WorkOrderAssignment
Replaces the simple `assigned_to`/`assigned_team` pattern. A work order may involve multiple people or groups, including field technicians, remote support, vendors, approvers, managers, and AI agents — each with a specific role.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| work_order_id | UUID (FK) | Required |
| assignee_type | enum | USER, TEAM, VENDOR, AI_AGENT |
| assignee_id | UUID | References the assignee based on assignee_type |
| role | enum | PRIMARY, SECONDARY, OBSERVER, APPROVER, DISPATCHED_TECH, REMOTE_SUPPORT |
| assigned_at | timestamptz | Required |
| assigned_by | UUID (FK → User) | Required |
| accepted_at | timestamptz | Nullable |
| removed_at | timestamptz | Nullable |
| status | enum | ASSIGNED, ACCEPTED, DECLINED, REMOVED, COMPLETED |

**Role definitions:**
- **PRIMARY:** The lead technician responsible for completion.
- **SECONDARY:** Assistant or backup technician.
- **OBSERVER:** Manager or stakeholder who wants visibility but doesn't act.
- **APPROVER:** Person who must sign off before the work order can close.
- **DISPATCHED_TECH:** Assigned via dispatch workflow (links to Dispatch entity).
- **REMOTE_SUPPORT:** Off-site expert providing guidance.

#### Dispatch
An operational event tracking a technician's trip to perform work. One trip may support one or more work orders.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| work_order_id | UUID (FK) | Required |
| technician_id | UUID (FK → User) | Required |
| dispatch_group_id | UUID (FK) | Nullable, groups multi-WO trips |
| status | enum | PLANNED, DISPATCHED, ACCEPTED, EN_ROUTE, ARRIVED, WORK_STARTED, WORK_COMPLETED, CANCELLED |
| dispatched_at | timestamptz | Nullable |
| accepted_at | timestamptz | Nullable |
| departed_at | timestamptz | Nullable |
| arrived_at | timestamptz | Nullable |
| work_started_at | timestamptz | Nullable |
| work_completed_at | timestamptz | Nullable |
| cancelled_at | timestamptz | Nullable |
| travel_start_location | text | Nullable |
| travel_end_location | text | Nullable |
| mileage | decimal(8,2) | Nullable |
| travel_time_minutes | integer | Nullable |
| notes | text | Nullable |

Dispatch is an operational event, not just an assignment. Travel time and mileage may need to be allocated across multiple work orders in a DispatchGroup.

#### DispatchGroup
Groups multiple Dispatches into a single technician trip. One trip may cover multiple work orders at nearby locations.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| technician_id | UUID (FK → User) | Required |
| dispatch_date | date | Required |
| route_status | enum | PLANNED, IN_PROGRESS, COMPLETED, CANCELLED |
| notes | text | Nullable |
| created_at | timestamptz | |
| updated_at | timestamptz | |

#### AgentIdentity
AI agents are first-class users with traceable identities. Each agent type has a system identity record.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Nullable (system-level agents) |
| name | string | Required, e.g., "Knowledge Agent", "Scheduling Agent" |
| type | enum | KNOWLEDGE, SCHEDULING, DIAGNOSTIC, DISPATCH, COMPLIANCE, INVENTORY, CUSTOM |
| provider | string | Nullable, e.g., "openai", "anthropic", "ollama" |
| model | string | Nullable, e.g., "gpt-4o", "claude-sonnet-4-20250514" |
| permissions | JSONB | Scoped permission set |
| enabled | boolean | Default true |
| created_at | timestamptz | |

#### Automation
AI-native workflow rules. Equivalent of traditional maintenance workflow rules, defined as a core SIP concept.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| name | string | Required |
| description | text | Nullable |
| trigger_type | enum | EVENT, SCHEDULE, MANUAL, AI_RECOMMENDED |
| trigger_config | JSONB | Defines what triggers this automation |
| conditions | JSONB | Conditional filters before executing |
| actions | JSONB | What the automation does |
| enabled | boolean | Default true |
| created_by | UUID (FK → User) | Required |
| created_at | timestamptz | |
| updated_at | timestamptz | |
| last_run_at | timestamptz | Nullable |

**Built-in automations (shipped as system defaults):**
- When `part.quantity_on_hand < quantity_minimum`, create reorder recommendation.
- When inspection item fails, create corrective work order.
- When work order is overdue, notify manager.
- When asset status changes to DOWN, alert assigned support team.
- When a document is indexed, make it available to authorized AI retrieval.

#### AutomationRun
Tracks individual execution of an Automation. Provides auditability and debugging.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| automation_id | UUID (FK) | Required |
| status | enum | PENDING, RUNNING, SUCCEEDED, FAILED, CANCELLED |
| input_event_id | UUID | Nullable, FK to outbox or Activity |
| started_at | timestamptz | Required |
| completed_at | timestamptz | Nullable |
| logs | JSONB | Execution log entries |
| error | text | Nullable, set if status = FAILED |

#### AssetRelationship (Phase 2 — model reserved now)
Defines functional relationships between assets beyond parent/child hierarchy. Essential for dependency analysis and impact assessment.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| source_asset_id | UUID (FK → Asset) | Required |
| target_asset_id | UUID (FK → Asset) | Required |
| relationship_type | enum | PART_OF, DEPENDS_ON, POWERS, CONTROLS, CONNECTED_TO, FEEDS, PROTECTS, MONITORS, BACKS_UP |
| description | text | Nullable |
| created_by_id | UUID (FK → User) | Nullable |
| created_at | timestamptz | |

**MVP approach:** MVP only requires `parent_id` for simple physical hierarchy. AssetRelationship is deferred to Phase 2 but included here as a planned part of the **service-domain canonical model**. This preserves the path toward dependency analysis without requiring Neo4j early.

#### ServiceBulletin (Phase 2 — model reserved now)
Manufacturer-issued notices about known issues, recalls, or recommended actions for specific asset models. Central to AI-assisted troubleshooting and recall management.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Nullable (system-level bulletins) |
| manufacturer_id | UUID (FK) | Nullable |
| asset_model_id | UUID (FK) | Nullable |
| title | string | Required |
| bulletin_number | string | Nullable — manufacturer's bulletin identifier |
| severity | enum | INFO, LOW, MEDIUM, HIGH, CRITICAL |
| summary | text | Required |
| recommended_action | text | Nullable |
| affected_firmware_versions | text[] | Nullable — automatically matched against Asset.firmware_version |
| affected_software_versions | text[] | Nullable |
| affected_hardware_revisions | text[] | Nullable |
| effective_date | date | Nullable |
| source_document_id | UUID (FK → Document) | Nullable |
| visibility | enum | PRIVATE_TENANT, SHARED_VENDOR, PUBLIC, SYSTEM_DEFAULT |
| created_at | timestamptz | |
| updated_at | timestamptz | |

#### OrganizationSettings
Typed settings for per-tenant configuration. Replaces free-form JSONB on Organization.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| organization_id | UUID (PK + FK → Organization) | Required |
| timezone | string | Default "UTC" |
| default_currency | string | Default "USD" |
| unit_system | enum | IMPERIAL, METRIC, MIXED (default METRIC) |
| ai_enabled | boolean | Default true |
| ai_provider_config | JSONB | Nullable — deferred to Phase 1.5 |
| ai_prompt_retention_policy | enum | NONE, METADATA_ONLY, FULL_PROMPT (default METADATA_ONLY) |
| ai_message_retention_days | integer | Nullable |
| document_retention_days | integer | Nullable |
| work_order_display_prefix | string | Default "WO" |
| feature_flags | JSONB | Admin-toggleable features |
| created_at | timestamptz | |
| updated_at | timestamptz | |

#### IdempotencyKey
Prevents duplicate mutations when clients retry failed requests.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| user_id | UUID (FK → User) | Nullable |
| key | string | Required — the Idempotency-Key header value |
| request_hash | string | Required — SHA-256 hash of the request body |
| response_status | integer | Required — the original response status |
| response_body | JSONB | Required — the original response body |
| created_at | timestamptz | |
| expires_at | timestamptz | Required |

**Requirement:** All mutating POST endpoints must support the `Idempotency-Key` header. This prevents duplicate work orders, duplicate part consumption, duplicate document uploads, and duplicate AI conversation records.

#### Schedule
Recurring maintenance definition. Generates WorkOrders on trigger.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| asset_id | UUID (FK) | Required |
| name | string | Required |
| trigger_type | enum | CRON, METER (meter deferred to Phase 2) |
| trigger_config | JSONB | Cron: `{ "expression": "0 8 * * MON" }`. Meter: `{ "field": "operating_hours", "interval": 500 }` |
| work_order_template | JSONB | Template: `{ "title", "description", "type", "priority", "estimated_hours", "assignments": [{"assignee_type": "USER|TEAM", "assignee_id": "...", "role": "PRIMARY"}], "checklist_template" }` |
| next_due | timestamptz | Calculated on create and after each trigger |
| last_triggered | timestamptz | Set when scheduled WO is generated |
| enabled | boolean | Default true |
| created_at | timestamptz | |
| updated_at | timestamptz | |

Schedule triggers do NOT generate a new WorkOrder if a non-terminal work order from the same schedule already exists for the asset (deduplication — checked against statuses: DRAFT, OPEN, ASSIGNED, ACCEPTED, IN_PROGRESS, ON_HOLD, COMPLETED, REVIEWED).

#### Inspection
A WorkOrder of type INSPECTION with structured checklist.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| work_order_id | UUID (FK) | Required, unique (1:1 with WorkOrder) |
| template_name | string | Reference to the template used |

#### InspectionChecklistItem

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| inspection_id | UUID (FK) | Required |
| ordinal | integer | Display order |
| question | text | Required, e.g., "Is the belt tension within specification?" |
| response_type | enum | PASS_FAIL, NUMERIC, TEXT, PHOTO |
| expected_value | string | Nullable, e.g., "45-55 PSI" |
| actual_value | string | Nullable, filled by technician |
| result | enum | PASS, FAIL, N/A (nullable until answered) |
| finding | text | Nullable, technician notes especially on FAIL |
| photo_url | string | Nullable, for PHOTO type |

#### Part
Consumable parts and inventory.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| name | string | Required |
| part_number | string | Nullable |
| description | text | Nullable |
| quantity_on_hand | decimal(10,2) | Default 0 |
| quantity_minimum | decimal(10,2) | Nullable, triggers low-stock alert |
| unit | string | Required, e.g., "each", "liters", "meters" |
| unit_cost | decimal(10,4) | Nullable, in organization's default currency |
| storage_location | string | Nullable, e.g., "Warehouse A, Shelf 3B" |
| created_at | timestamptz | |
| updated_at | timestamptz | |

#### PartUsage
Junction table: parts consumed by a work order.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| work_order_id | UUID (FK) | Required |
| part_id | UUID (FK) | Required |
| quantity | decimal(10,2) | Required, > 0 |
| used_by_id | UUID (FK → User) | Required |

On insert, `part.quantity_on_hand` is decremented atomically. If below `quantity_minimum`, a `part.low_stock` event is emitted.

#### AssetPart
BOM (Bill of Materials): parts associated with an AssetType.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| asset_type_id | UUID (FK) | Required |
| part_id | UUID (FK) | Required |
| default_quantity | decimal(10,2) | Default quantity per asset of this type |

Used to pre-populate expected parts when creating work orders.

#### User

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| email | string | Required, unique within organization |
| name | string | Required |
| role | enum | ADMIN, MANAGER, TECHNICIAN, VIEWER, VENDOR, AUDITOR |
| skills | text[] | e.g., `["electrical", "HVAC-certified"]` |
| certifications | JSONB[] | Array of `{ "name", "issuing_body", "expiry_date", "document_url" }` |
| working_hours | JSONB | e.g., `{ "timezone": "America/Chicago", "shifts": [{"days": [1,2,3,4,5], "start": "07:00", "end": "15:30"}] }` |
| is_active | boolean | Default true. Soft-disable for offboarding (preserves audit trail). |
| created_at | timestamptz | |
| updated_at | timestamptz | |

#### Team

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| name | string | Required |
| description | text | Nullable |
| lead_id | UUID (FK → User) | Nullable |
| created_at | timestamptz | |
| updated_at | timestamptz | |

Team membership managed via `team_members` junction table: `(team_id, user_id)`.

#### Document

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| name | string | Required (original filename or user-provided) |
| type | enum | MANUAL, PROCEDURE, DIAGRAM, WARRANTY, CERTIFICATE, PHOTO, OTHER |
| mime_type | string | Required, e.g., "application/pdf" |
| size_bytes | bigint | Required |
| version | string | Nullable, document version identifier |
| checksum | string | Required, SHA-256 for integrity verification |
| source | enum | UPLOAD, API, PLUGIN, SYSTEM, VENDOR, PUBLIC_IMPORT |
| storage_path | string | Object storage key |
| visibility | enum | PRIVATE_TENANT, SHARED_VENDOR, PUBLIC, SYSTEM_DEFAULT |
| processing_status | enum | PENDING, EXTRACTING, EXTRACTED, CHUNKING, EMBEDDING, INDEXED, FAILED |
| processing_error | text | Nullable, set if processing_status = FAILED |
| extracted_text_path | string | Nullable, object storage key for extracted text |
| text_content | text | Extracted text for AI indexing (null until indexed) |
| effective_date | date | Nullable |
| expiration_date | date | Nullable |
| supersedes_document_id | UUID (FK, self) | Nullable, links to a document this one replaces |
| version | integer | Default 1 — optimistic locking |
| archived_at | timestamptz | Nullable |
| archived_by_id | UUID (FK → User) | Nullable |
| archive_reason | text | Nullable |
| metadata | JSONB | Extensible |
| uploaded_by_id | UUID (FK → User) | Required |
| created_at | timestamptz | |

**Knowledge scope (visibility):**
- **PRIVATE_TENANT:** Only visible within the owning organization.
- **SHARED_VENDOR:** Visible to the organization and specific vendors.
- **PUBLIC:** Publicly accessible knowledge (e.g., manufacturer manuals, regulatory standards).
- **SYSTEM_DEFAULT:** Shipped with SIP, available to all tenants as reference knowledge.

SIP must distinguish customer-specific private records from public manuals, vendor-shared documentation, system defaults, and community knowledge.

#### DocumentLink
Junction table replacing the polymorphic `entity_type`/`entity_id` pattern on Document. A single document can relate to multiple entities.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| document_id | UUID (FK) | Required |
| entity_type | enum | ASSET, WORK_ORDER, ASSET_MODEL, MANUFACTURER, PART, INSPECTION |
| entity_id | UUID | Required |
| relationship_type | enum | MANUAL_FOR, PHOTO_OF, WARRANTY_FOR, PROCEDURE_FOR, EVIDENCE_FOR, ATTACHMENT |
| created_by_id | UUID (FK → User) | Required |
| created_at | timestamptz | |

**Rationale:** A manual may apply to an AssetModel and also be attached to a specific Asset. A photo may be evidence for a WorkOrder and associated with an InspectionChecklistItem.

#### DocumentChunk
Document-specific chunking structure for embedding and citation.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| document_id | UUID (FK) | Required |
| chunk_index | integer | Required |
| content | text | Required |
| token_count | integer | Nullable |
| page_number | integer | Nullable |
| section_title | string | Nullable |
| metadata | JSONB | |
| created_at | timestamptz | |

**Indexes:**
```sql
CREATE INDEX idx_dc_document ON document_chunks (document_id, chunk_index);
CREATE INDEX idx_dc_org_doc ON document_chunks (organization_id, document_id);
```

#### EmbeddingRecord
Generic embedding table for any source entity. Stores vectors for semantic search across all indexed content.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| source_type | enum | ASSET, WORK_ORDER, DOCUMENT_CHUNK, INSPECTION_FINDING |
| source_id | UUID | Required |
| collection | string | Required, e.g., "work_order_notes", "asset_metadata", "document_chunks" |
| content | text | Required — the source text that was embedded |
| embedding | vector(1024) | Required |
| embedding_model | string | Required, e.g., "BGE-M3", "text-embedding-3-small" |
| metadata | JSONB | |
| created_at | timestamptz | |
| updated_at | timestamptz | |

**Index:** HNSW index on `embedding`.

**Document ingestion flow (revised):**
1. User uploads a document → stored in object storage, SHA-256 checksum calculated.
2. Document record created with `processing_status = PENDING`.
3. Background worker extracts text → `processing_status = EXTRACTING`.
4. Text stored in object storage → `processing_status = EXTRACTED`.
5. Text chunked into segments (512 tokens, 64 overlap) → DocumentChunk records created → `processing_status = CHUNKING`.
6. Embedding generation queued → `processing_status = EMBEDDING`.
7. Embeddings created → EmbeddingRecord inserted for each chunk → `processing_status = INDEXED`.
8. `document.indexed` event emitted → AutomationEngine makes it available for authorized AI retrieval.
9. If any step fails → `processing_status = FAILED`, error recorded in `processing_error`.

#### Activity
Immutable audit log. Append-only, never updated or deleted.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| actor_id | UUID | Nullable, FK to users (null for system events) |
| actor_type | enum | HUMAN, AI_AGENT, SYSTEM, PLUGIN |
| agent_identity_id | UUID (FK → AgentIdentity) | Nullable, set if actor_type = AI_AGENT |
| plugin_id | UUID (FK → Plugin) | Nullable, set if actor_type = PLUGIN |
| entity_type | string | e.g., "asset", "work_order", "schedule" |
| entity_id | UUID | |
| action | string | e.g., "created", "status_changed", "assigned", "completed" |
| changes | JSONB | `{ "field": { "old": ..., "new": ... } }` diff of changed fields |
| request_id | string | Nullable — for tracing across services |
| correlation_id | string | Nullable — for grouping related events |
| source | enum | API, UI, AI, AUTOMATION, PLUGIN, SYSTEM |
| reason | text | Nullable — human-readable explanation |
| metadata | JSONB | Extensible |
| ip_address | inet | Nullable, client IP |
| user_agent | string | Nullable |
| created_at | timestamptz | |

Activity is written **atomically** with the business change and outbox event in the same database transaction for direct user/API mutations. Derived async events may create additional Activity records. The application must not expose update or delete endpoints for Activity. Database permissions should prevent UPDATE/DELETE by the application role.

#### AIConversation

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| user_id | UUID (FK → User) | Required |
| agent_identity_id | UUID (FK → AgentIdentity) | Required |
| title | string | Nullable — auto-generated from first message |
| entity_type | string | Nullable — context entity type |
| entity_id | UUID | Nullable — context entity (e.g., asset being discussed) |
| created_at | timestamptz | |
| updated_at | timestamptz | |
| archived_at | timestamptz | Nullable |

#### AIMessage

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| conversation_id | UUID (FK → AIConversation) | Required |
| role | enum | USER, ASSISTANT, SYSTEM, TOOL |
| content | text | Required |
| structured_response | JSONB | Nullable — the `{ answer, confidence, sources, ... }` object |
| sources | JSONB | Nullable — source citations for this message |
| token_input_count | integer | Nullable |
| token_output_count | integer | Nullable |
| model | string | Nullable |
| provider | string | Nullable |
| created_at | timestamptz | |

#### AIRetrievalTrace

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required |
| message_id | UUID (FK → AIMessage) | Required |
| retriever_type | enum | RELATIONAL, VECTOR, TOOL |
| source_type | string | e.g., "work_order", "document_chunk" |
| source_id | UUID | |
| score | decimal(5,4) | Nullable — relevance/similarity score |
| included_in_context | boolean | Default false — was this result included in the prompt? |
| created_at | timestamptz | |

**AI logging and privacy:** Activity stores AI action metadata and links to AIConversation/AIMessage records. Full prompt context is stored only if tenant settings permit. Prompt retention is configurable per organization via `ai_prompt_retention_policy`:

| Setting | Value | Behavior |
|---------|-------|----------|
| `ai_prompt_retention_policy` | NONE | No prompt or message content retained |
| | METADATA_ONLY (default) | Conversation metadata + structured responses retained; raw prompts discarded |
| | FULL_PROMPT | Full prompt context retained for debugging |
| `ai_message_retention_days` | integer | Days before messages are deleted (null = indefinite) |

Sensitive auth data, secrets, access tokens, and raw passwords must never be logged in AIConversation, AIMessage, Activity, traces, or eval datasets.

#### Plugin
Plugin registry record.

| Attribute | Type | Constraints |
|-----------|------|-------------|
| id | UUID (PK) | |
| organization_id | UUID (FK) | Required (which tenant installed it) |
| name | string | Required, matches manifest |
| version | string | Required, semver |
| trust_level | enum | UI_ONLY, READ_ONLY_DATA, MUTATING_WORKFLOW, AI_TOOL, ENTERPRISE_PRIVATE |
| manifest | JSONB | Full manifest snapshot at install time |
| config | JSONB | Plugin-specific config values |
| status | enum | INSTALLED, CONFIGURED, ENABLED, DISABLED, FAILED, NEEDS_UPDATE, BLOCKED_BY_VERSION, UNINSTALLED |
| installed_at | timestamptz | |
| updated_at | timestamptz | |

### 8.2 Entity Relationship Map

```
Organization
 ├── 1:N Location          (hierarchy: parent_id FK to self)
 ├── 1:N Asset
 ├── 1:N AssetType         (null org_id = system default)
 ├── 1:N Manufacturer      (null org_id = system default)
 ├── 1:N AssetModel        (null org_id = system default)
 ├── 1:N WorkOrder
 ├── 1:N WorkOrderAssignment
 ├── 1:N Schedule
 ├── 1:N Dispatch
 ├── 1:N DispatchGroup
 ├── 1:N User
 ├── 1:N Team
 ├── 1:N Part
 ├── 1:N Document
 ├── 1:N Activity
 ├── 1:N Automation
 ├── 1:N AutomationRun
 ├── 1:N AgentIdentity     (null org_id = system-level)
 └── 1:N Plugin

Location   1:N Asset
Location   1:N Location    (parent/child)

Asset      1:N WorkOrder
Asset      1:N Schedule
Asset      1:N Document    (polymorphic)
Asset      N:1 AssetType
Asset      N:1 AssetModel  (optional, pre-populates type+attributes)
Asset      N:1 Location
Asset      1:N Asset       (parent/child hierarchy)

AssetType  1:N Asset
AssetType  1:N AssetModel
AssetType  1:N AssetPart   (BOM)

Manufacturer 1:N AssetModel

AssetModel N:1 Manufacturer
AssetModel N:1 AssetType
AssetModel 1:N Asset

WorkOrder  N:1 Asset
WorkOrder  N:1 Schedule    (generated from)
WorkOrder  1:N WorkOrderAssignment
WorkOrder  1:N Dispatch
WorkOrder  1:N PartUsage
WorkOrder  1:N Document    (polymorphic)
WorkOrder  1:N Activity
WorkOrder  1:1 Inspection

WorkOrderAssignment N:1 WorkOrder
WorkOrderAssignment N:1 User | Team (polymorphic by assignee_type)

DispatchGroup 1:N Dispatch
Dispatch       N:1 WorkOrder
Dispatch       N:1 User        (technician)
Dispatch       N:1 DispatchGroup

Schedule   N:1 Asset
Schedule   1:N WorkOrder   (generates)

Inspection 1:N InspectionChecklistItem

Part       1:N PartUsage
Part       1:N AssetPart

Team       N:N User        (team_members junction)

User       1:N WorkOrderAssignment
User       1:N WorkOrder   (created_by)
User       1:N Dispatch    (technician)
User       1:N Activity    (actor)
User       1:N Document    (uploaded_by)

Document   N:1 Asset | WorkOrder (polymorphic)
Document   1:1 Document    (supersedes)

AgentIdentity 1:N Activity

Automation   1:N AutomationRun

Plugin       1:N Activity
```

### 8.3 Critical Events

| Event | Trigger | Primary Consumers |
|-------|---------|-------------------|
| `asset.created` | POST /assets | VectorIndexer (embed name+description+attributes), ActivityLog |
| `asset.status_changed` | PATCH /assets/{id} | NotificationService, ActivityLog |
| `asset.decommissioned` | status → RETIRED | ActivityLog |
| `manufacturer.created` | POST /manufacturers | ActivityLog |
| `model.created` | POST /models | ActivityLog (system seed data or admin-created) |
| `work_order.created` | POST /work-orders or Schedule trigger | NotificationService, AutomationEngine, ActivityLog |
| `work_order.status_changed` | Any status transition | AutomationEngine (event-driven automations), ActivityLog |
| `work_order.assigned` | INSERT into WorkOrderAssignment | NotificationService (push to assignee), ActivityLog |
| `work_order.assignment_accepted` | assignment status → ACCEPTED | NotificationService, ActivityLog |
| `work_order.assignment_declined` | assignment status → DECLINED | NotificationService (alert manager), ActivityLog |
| `work_order.started` | PATCH status → IN_PROGRESS | ActivityLog |
| `work_order.completed` | PATCH status → COMPLETED | VectorIndexer (embed resolution_notes), ScheduleService (recalculate next_due), ActivityLog |
| `work_order.reviewed` | PATCH status → REVIEWED | ActivityLog |
| `work_order.closed` | PATCH status → CLOSED | VectorIndexer (final indexing), ActivityLog |
| `work_order.reopened` | PATCH status → REOPENED | ActivityLog |
| `work_order.cancelled` | PATCH status → CANCELLED | ActivityLog |
| `work_order.overdue` | Scheduled job (poll past-due WOs) | NotificationService (escalate to manager), AutomationEngine, ActivityLog |
| `schedule.triggered` | Cron tick or meter check | WorkOrder generator, ActivityLog |
| `inspection.failed` | Checklist item result = FAIL on inspection completion | WorkOrder generator (auto-create corrective WO via Automation), NotificationService |
| `part.consumed` | INSERT into PartUsage | InventoryService (decrement stock), ActivityLog |
| `part.low_stock` | quantity_on_hand < quantity_minimum | AutomationEngine (create reorder rec), NotificationService |
| `document.uploaded` | POST /documents | VectorIndexer (extract text, embed, index), ActivityLog |
| `document.indexed` | processing_status → INDEXED | AutomationEngine (make available for AI retrieval), ActivityLog |
| `document.processing_failed` | processing_status → FAILED | Admin notification, retry logic |
| `dispatch.created` | POST /dispatches | NotificationService (push to technician), ActivityLog |
| `dispatch.status_changed` | PATCH /dispatches/{id} | NotificationService, ActivityLog |
| `dispatch_group.completed` | route_status → COMPLETED | ActivityLog |
| `automation.triggered` | Matching event or schedule | AutomationRun executor |
| `automation_run.completed` | Run finishes | ActivityLog |
| `plugin.installed` | POST /plugins | PluginRegistry (start sidecar), ActivityLog |
| `plugin.uninstalled` | DELETE /plugins/{id} | PluginRegistry (stop sidecar, cleanup), ActivityLog |

---

## 9. Key Workflows

### WF1: Asset Creation (End-to-End)

```
1. Admin creates or selects an AssetType.
2. Admin defines required custom fields through JSON Schema on the AssetType.
3. User creates an Asset from that AssetType (POST /api/v1/assets).
4. User optionally links the Asset to Location, Manufacturer, and AssetModel.
5. If AssetModel is selected, asset_type_id and default attributes are pre-populated.
6. System validates the asset attributes against the AssetType schema.
7. System creates the asset record.
8. System emits asset.created.
9. System writes an Activity record.
10. System queues asset metadata for vector indexing.
11. Asset becomes available through API, UI, and authorized AI retrieval.
```

### WF2: Work Order Full Lifecycle

```
1. Work order is created (DRAFT or OPEN):
   - Manually via POST /api/v1/work-orders
   - Automatically via schedule trigger (source = SCHEDULE)
   - Via AI recommendation converted by human
   - Work order created in DRAFT is not yet visible in technician queues.

2. Work order is published (DRAFT → OPEN):
   - PATCH /api/v1/work-orders/{id} { status: "OPEN" }
   - Now visible in dispatch queues.

3. Technicians/teams are assigned via WorkOrderAssignment:
   POST /api/v1/work-orders/{id}/assignments
   → { assignee_type: "USER", assignee_id: "...", role: "PRIMARY" }
   → Emits event: work_order.assigned
   → Work order status transitions to ASSIGNED.

4. Technician accepts:
   PATCH /api/v1/work-orders/{id}/assignments/{assignment_id}
   → { status: "ACCEPTED" }
   → Emits event: work_order.assignment_accepted
   → Work order status transitions to ACCEPTED.

5. Technician starts work:
   PATCH /api/v1/work-orders/{id} { status: "IN_PROGRESS" }
   → Sets actual_start = NOW()
   → Emits event: work_order.started

6. Technician executes work:
   - Completes inspection checklist items if applicable
   - Logs parts used (POST /api/v1/work-orders/{id}/parts)
   - Parts inventory decremented atomically
   - If below threshold → emits part.low_stock → AutomationEngine evaluates

7. Technician marks complete:
   PATCH /api/v1/work-orders/{id} {
     status: "COMPLETED",
     resolution_notes: "Replaced bearing...",
     actual_end: NOW()
   }
   → Validates resolution_notes is non-empty
   → Sets actual_hours = actual_end - actual_start
   → Emits event: work_order.completed

8. Manager reviews and closes:
   PATCH /api/v1/work-orders/{id} { status: "REVIEWED" }
   → Review activity logged with reviewer identity
   PATCH /api/v1/work-orders/{id} { status: "CLOSED" }
   → Terminal state for successful work

9. If reopened:
   PATCH /api/v1/work-orders/{id} { status: "REOPENED" }
   → Transitions directly to IN_PROGRESS
   → Original completion and closure history preserved

10. If cancelled at any point:
    PATCH /api/v1/work-orders/{id} { status: "CANCELLED", resolution_notes: "Reason..." }
    → Cancellation reason required

Every status transition creates an Activity record.
```

### WF3: Dispatch Workflow (One Trip, Multiple Work Orders)

```
1. Manager creates a DispatchGroup for a technician's day:
   POST /api/v1/dispatch-groups
   → { technician_id, dispatch_date, route_status: "PLANNED" }

2. Manager adds dispatches to the group:
   POST /api/v1/dispatches
   → { work_order_id, technician_id, dispatch_group_id, status: "PLANNED" }

3. Technician receives dispatch notifications.

4. Dispatcher marks dispatch as sent:
   PATCH /api/v1/dispatches/{id} { status: "DISPATCHED", dispatched_at: NOW() }

5. Technician accepts and departs:
   PATCH { status: "ACCEPTED", accepted_at: NOW() }
   PATCH { status: "EN_ROUTE", departed_at: NOW(), travel_start_location: "..." }

6. Technician arrives on site:
   PATCH { status: "ARRIVED", arrived_at: NOW(), travel_end_location: "..." }
   → System records mileage and travel_time_minutes

7. Technician starts work:
   PATCH { status: "WORK_STARTED", work_started_at: NOW() }
   → Corresponding work order transitions to IN_PROGRESS

8. Technician completes work:
   PATCH { status: "WORK_COMPLETED", work_completed_at: NOW() }
   → Corresponding work order transitions to COMPLETED

9. When all dispatches in the group are completed:
   PATCH /api/v1/dispatch-groups/{id} { route_status: "COMPLETED" }
```

### WF4: AI Query with Citations

```
1. User asks a question about an asset, work order, symptom, or document.
   POST /api/v1/ai/chat { message: "What's the history of Pump A?" }

2. System identifies the user and their permission scope.

3. System retrieves only authorized records:
   - Relational: asset metadata, work order history within user's organization/scope
   - Vector: semantic search over work order notes and documents (scope-filtered)
   - All retrieval goes through the service layer, not raw DB queries

4. System gathers relevant context:
   - Asset metadata and status
   - Recent work orders with resolution notes
   - Related document chunks (manuals, procedures)
   - Similar failure patterns on comparable assets

5. AI generates a cited, structured response:
   {
     "answer": "Pump A has had three bearing-related failures...",
     "confidence": 0.87,
     "sources": [{ "type": "work_order", "id": "wo_123", ... }],
     "unsupported_claims": [],
     "recommended_actions": []
   }

6. User may convert the recommendation into a draft work order.

7. Human confirms before any mutation occurs.

8. System records the AI interaction in Activity with agent_identity_id and full context.
```

### WF5: Document Ingestion

```
1. User uploads a document (POST /api/v1/documents).

2. System stores the original file in object storage.

3. System calculates SHA-256 checksum.

4. System creates Document record with processing_status = PENDING.

5. Background worker picks up the document:
   a. Extracts text content from the file.
   b. Stores extracted text in object storage, sets extracted_text_path.
   c. Chunks extracted text (512 tokens, 64 overlap).
   d. Sets processing_status = EXTRACTING during extraction.

6. Background worker generates embeddings:
   a. Sets embedding_status = QUEUED.
   b. Generates vector embeddings for each chunk.
   c. Inserts chunks + embeddings into pgvector collection.
   d. Links chunks to source document and entity.

7. System updates:
   processing_status = INDEXED
   embedding_status = EMBEDDED

8. System emits document.indexed event.

9. AutomationEngine triggers: make document available for authorized AI retrieval.

10. Authorized AI queries can now cite this document.
```

### WF6: Plugin Installation with Trust Levels

```
1. Admin uploads plugin package (Docker image + plugin.json manifest).

2. System reads plugin manifest:
   - Validates required fields (name, version, sip_version, trust_level, permissions)
   - Checks plugin trust_level against allowed levels for the organization

3. System displays requested permissions:
   - extension_points the plugin wants to hook into
   - API permissions the plugin requires
   - AI tools the plugin wants to register (if trust_level = AI_TOOL)

4. Admin approves or rejects each permission scope.

5. System validates compatibility:
   - sip_version semver range check
   - Plugin name + version not already installed
   - All declared permission identifiers exist in the registry

6. Plugin enters INSTALLED → CONFIGURED → ENABLED lifecycle.

7. Plugin registers extension points.

8. Plugin actions are written to Activity with plugin_id.

9. Admin can disable (ENABLED → DISABLED), re-enable, or uninstall (→ UNINSTALLED).

10. Uninstall triggers safe cleanup: sidecar stopped, data policy applied.
```

### WF7: Automation Execution

```
1. An event is emitted (e.g., part.low_stock, inspection.failed, work_order.overdue).

2. AutomationEngine receives the event.

3. Engine queries enabled automations matching the event trigger_type = EVENT:
   SELECT * FROM automations
   WHERE enabled = true
     AND trigger_type = 'EVENT'
     AND trigger_config->>'event_type' = $event.type

4. For each matching automation, engine evaluates conditions:
   - If conditions match → create AutomationRun with status = PENDING
   - If conditions don't match → skip

5. Engine executes the automation actions defined in actions JSONB:
   - Create work order
   - Send notification
   - Update entity status
   - Call webhook

6. AutomationRun transitions:
   - RUNNING during execution
   - SUCCEEDED on success, with logs
   - FAILED on error, with error message

7. Activity record written for the run.
```

---

## 10. AI Requirements

The AI layer operates on the **structured maintenance substrate** — the **AI-readable operational history** built from every work order, inspection, document, and resolution note in the platform. AI retrieval goes through the same authorization boundaries as direct API access. No backdoors, no raw database queries, no privileged data paths.

### 10.1 Knowledge Agent (MVP)

**Capability:** Natural language Q&A over all ingested SIP data.

**Required behaviors:**

| # | Requirement | Detail |
|---|-------------|--------|
| AI1 | **Source citation** | Every factual claim in the response must cite its source (work order ID, document name, timestamp). |
| AI2 | **Honesty on gaps** | If the answer is not in the available context, the agent must say "I don't have enough information to answer that" rather than hallucinate. |
| AI3 | **Authorization-bound retrieval** | AI retrieval must enforce the same authorization boundary as direct API access. If a user cannot retrieve a record through the API, the AI must not be able to retrieve, summarize, embed, cite, or infer from that record for that user. |
| AI4 | **Scope of query** | The agent can query: asset metadata, work order history and resolution notes, document text, inspection findings, parts inventory (via tool call). |
| AI5 | **No high-impact mutations in MVP** | For MVP, AI agents may draft mutations (create work order, assign technician) but may not execute high-impact mutations without human confirmation. High-impact mutations requiring confirmation: create work order, assign technician, close work order, change asset status, create purchase order, update inventory quantity, share data across tenant boundaries. |
| AI6 | **Structured response format** | Every AI response is returned as a structured JSON object: `{ "answer", "confidence", "sources": [...], "unsupported_claims": [], "recommended_actions": [] }`. |
| AI7 | **Streaming responses** | Responses are streamed via Server-Sent Events (SSE) for low perceived latency. |
| AI8 | **Conversation context** | Multi-turn conversations are supported. The agent remembers prior turns within a session (configurable window). |
| AI9 | **Multi-lingual** | Query and response in the user's language. No language restriction. |
| AI10 | **Auditability** | Every query and response is logged in Activity with agent_identity_id and the full prompt context for debugging. |

**Retrieval strategy (RAG):**

```
1. Embed query text using the configured embedding model.
2. Parallel retrieval:
   a. Vector similarity search on work_order_notes collection (top_k=5)
   b. Vector similarity search on asset_documents collection (top_k=3)
   c. Vector similarity search on asset_metadata collection (top_k=3)
   d. Structured query to relational DB for "hard facts":
      - Current asset status
      - Last 10 work orders (chronological)
      - Open work orders
      - Upcoming scheduled maintenance
3. Merge results into a context window.
4. Re-rank merged results by cross-encoder relevance score (if configured).
5. Build final prompt: system prompt + formatted context + user message + conversation history.
```

**Hallucination mitigation:**
- System prompt instructs: "If the information is not in the provided context, respond with 'I don't have enough information to answer that question.' Do not fabricate."
- Context is always provided in the prompt — the LLM never queries databases directly.
- In Phase 1.5, add a "faithfulness evaluator" that checks the response against retrieved context and flags potential hallucinations.
- Every response includes an `unsupported_claims` array that calls out any statement the LLM cannot source.

**Structured AI response format (mandatory for all Knowledge Agent responses):**

```json
{
  "answer": "Pump A has had three bearing-related failures in the last 90 days.",
  "confidence": 0.87,
  "sources": [
    {
      "type": "work_order",
      "id": "wo_123",
      "field": "resolution_notes",
      "quote": "Bearing assembly replaced due to inner race spalling...",
      "timestamp": "2026-04-21T15:00:00Z"
    },
    {
      "type": "document",
      "id": "doc_456",
      "name": "Pump A Maintenance Manual",
      "section": "Section 4.2 - Bearing Specifications",
      "timestamp": "2025-11-01T00:00:00Z"
    }
  ],
  "unsupported_claims": [],
  "recommended_actions": [
    {
      "action": "create_work_order",
      "title": "Inspect bearing assembly on Pump A",
      "justification": "Three bearing failures in 90 days suggests systemic issue",
      "requires_confirmation": true
    }
  ]
}
```

**Field definitions:**
- `answer`: The natural language response to the user's query.
- `confidence`: 0.0–1.0 score reflecting the LLM's confidence in the factual accuracy.
- `sources`: Array of source records the answer is grounded in. No AI answer about customer data should be considered valid unless it can cite the records it used.
- `unsupported_claims`: Array of statements the LLM believes but cannot directly cite. Should be empty in practice; populated as a safety valve for transparency.
- `recommended_actions`: Suggested next steps the user may want to take (create WO, schedule inspection, order part). Each action has `requires_confirmation` flag.

**MVP AI eval harness requirement:** The MVP must include an AI evaluation harness before public release, testing:
- Factual accuracy on asset history questions
- Citation accuracy (are citations real and correct?)
- Refusal when data is unavailable
- Tenant isolation (does AI leak cross-org data?)
- Permission boundary enforcement (does AI respect RBAC?)
- Work order summarization quality
- Document Q&A accuracy
- Regression tests across model/provider changes

**Tool calling (Phase 2):**
- The Knowledge Agent will support function calling for structured queries (e.g., "List all work orders for asset X in the last 30 days" executes a SQL query via tool).
- Tools are registered via the `ai.tools` extension point.
- Mutating tools (create WO, update status) are gated behind human confirmation in the UI.
- The `ai.tools` extension point ships in Phase 2, but tool definitions appear in Phase 1 manifest format for forward compatibility.

### 10.2 AI Service Architecture

```
┌─────────────────────────────────────────────┐
│                AI Service                     │
│                                               │
│  POST /api/v1/ai/chat                         │
│       │                                       │
│       ▼                                       │
│  ┌─────────────┐    ┌──────────────────────┐ │
│  │ Intent      │    │ Context Builder      │ │
│  │ Classifier  │    │                      │ │
│  │             │    │ ┌──────────────────┐ │ │
│  │ "asset_     │    │ │ Relational       │ │ │
│  │  history"   │    │ │ Fetcher          │ │ │
│  │ "document_  │    │ └──────────────────┘ │ │
│  │  qa"        │    │ ┌──────────────────┐ │ │
│  │ "general"   │    │ │ Vector Retriever │ │ │
│  └─────────────┘    │ └──────────────────┘ │ │
│                      │ ┌──────────────────┐ │ │
│                      │ │ Tool Executor    │ │ │
│                      │ │ (Phase 2)        │ │ │
│                      │ └──────────────────┘ │ │
│                      └──────────────────────┘ │
│                               │               │
│                               ▼               │
│                      ┌──────────────────────┐ │
│                      │ Prompt Assembler     │ │
│                      │ (system + context    │ │
│                      │  + history + query)  │ │
│                      └──────────────────────┘ │
│                               │               │
│                               ▼               │
│                      ┌──────────────────────┐ │
│                      │ LLM Provider         │ │
│                      │ (LiteLLM → OpenAI /  │ │
│                      │  Anthropic / Ollama) │ │
│                      └──────────────────────┘ │
│                               │               │
│                               ▼               │
│                      ┌──────────────────────┐ │
│                      │ Response Streamer    │ │
│                      │ (SSE → client)       │ │
│                      └──────────────────────┘ │
└─────────────────────────────────────────────┘
```

### 10.3 LLM Provider Abstraction

SIP must support multiple LLM providers through an abstraction layer (LiteLLM):

| Requirement | Detail |
|-------------|--------|
| **Supported providers** | OpenAI-compatible APIs, Anthropic, Google, local models (Ollama, vLLM) via LiteLLM proxy |
| **Configurable per tenant** | Each organization can configure its own LLM provider and API key |
| **Fallback chain** | If primary provider fails, fall back to secondary (e.g., OpenAI → Ollama) |
| **Cost tracking** | Log token usage per query for cost attribution |
| **Rate limiting** | Per-tenant rate limiting on AI queries (configurable in org settings) |
| **Provider swap** | Changing provider requires no code changes — configuration only |

### 10.4 Embedding Requirements

| Requirement | Detail |
|-------------|--------|
| Model | Configurable. Default: `BGE-M3` (open-source, 1024-dim, multilingual). Support OpenAI `text-embedding-3-small` as alternative. |
| Dimensions | Configurable per model. Store dimension in collection metadata. |
| Chunking | Documents: 512-token chunks with 64-token overlap. Work orders: per work order (no chunking needed for short notes). |
| Re-indexing | Full re-index on embedding model change. Triggered via admin API. |
| Async processing | Embedding generation happens in background workers. Document upload returns immediately; text is available after indexing completes (typically < 5 seconds). |

---

## 11. Plugin and Extension Requirements

### 11.1 Plugin Trust Levels

Every plugin is classified into one of five trust levels, each with increasing scrutiny:

| Level | Name | Risk | Capabilities | Approval |
|-------|------|------|-------------|----------|
| 1 | **UI-only** | Lowest | Adds frontend panels, tabs, dashboards, or visualizations. No direct backend mutation. | Auto-approved (read-only surface) |
| 2 | **Read-only data** | Low | Can read scoped data through approved APIs. Cannot mutate operational records. | Admin review of data scope |
| 3 | **Mutating workflow** | Medium | Can create or update records through scoped APIs. | Explicit admin approval per permission |
| 4 | **AI tool** | High | Registers callable tools for AI agents. AI may invoke the tool directly. | Explicit permission review + audit logging + dry-run support |
| 5 | **Enterprise/private** | Variable | Tenant-controlled. May have custom deployment and review process. | Custom per-tenant approval workflow |

Trust levels are declarative — set in the manifest. The system enforces the level's constraints. A plugin cannot escalate its trust level at runtime.

### 11.2 Plugin Manifest Specification

```json
{
  "name": "iot-connector",
  "version": "1.0.0",
  "description": "Ingest MQTT sensor data into SIP",
  "author": "Example Developer",
  "license": "AGPL-3.0",
  "sip_version": ">=0.1.0",
  "trust_level": "MUTATING_WORKFLOW",
  "extension_points": [
    "entity.custom_entity",
    "events.subscribers",
    "ai.context_providers"
  ],
  "permissions": [
    "read:asset",
    "write:asset",
    "read:work_order"
  ],
  "config_schema": {
    "mqtt_broker_url": "string",
    "topic_map": "object"
  },
  "migrations": [],
  "ui_extensions": [],
  "ai_tools": []
}
```

**Manifest field requirements:**

| Field | Required | Description |
|-------|----------|-------------|
| name | Yes | Unique identifier. Lowercase, hyphenated. Max 64 chars. |
| version | Yes | Semver. |
| sip_version | Yes | Semver range for compatible SIP core versions. |
| description | Yes | Max 500 chars. |
| author | No | Author/org name. |
| license | Yes | SPDX identifier. |
| trust_level | Yes | One of: UI_ONLY, READ_ONLY_DATA, MUTATING_WORKFLOW, AI_TOOL, ENTERPRISE_PRIVATE. |
| extension_points | Yes | Array of extension point identifiers this plugin hooks into. |
| permissions | Yes | Array of permission identifiers. Must be subset of available permissions. |
| config_schema | No | JSON Schema for plugin configuration. |
| resources | No | Minimum compute resources for the sidecar container. |
| dependencies | No | Array of `{ "plugin_name": ">=version" }` for plugin-to-plugin dependencies. |
| migrations | No | Array of migration scripts for the plugin's custom entities. |
| ui_extensions | No | Array of UI extension declarations. |
| ai_tools | No | Array of AI tool definitions (only valid if trust_level = AI_TOOL). |

### 11.3 Plugin Lifecycle

```
INSTALLED ──► CONFIGURED ──► ENABLED
                                 │
                    ┌────────────┼────────────┐
                    ▼            ▼            ▼
                DISABLED     FAILED    NEEDS_UPDATE
                    │            │            │
                    ▼            ▼            ▼
                ENABLED     DISABLED    BLOCKED_BY_VERSION
                                              │
                    ┌─────────────────────────┼────────────────┐
                    ▼                         ▼                ▼
              UNINSTALLED              (version fixed)   (stay blocked)
```

**Status definitions:**
- **INSTALLED:** Plugin record created. Manifest validated. Not yet configured or active.
- **CONFIGURED:** Admin has set required config values.
- **ENABLED:** Plugin is running. Extension points active. Actions auditable.
- **DISABLED:** Plugin stopped by admin. Extension points deactivated. Data preserved.
- **FAILED:** Plugin crashed or health check failed. Auto-disabled after N failures.
- **NEEDS_UPDATE:** Newer version available. Compatible plugin version exists.
- **BLOCKED_BY_VERSION:** Plugin incompatible with current SIP core version.
- **UNINSTALLED:** Plugin removed. Sidecar destroyed. Data cleanup policy applied.

### 11.4 Plugin Security Model

| Requirement | Detail |
|-------------|--------|
| **Permission declaration** | Plugins must declare permissions before installation. Cannot exceed declared scope at runtime. |
| **Admin approval** | Admins must approve requested permissions during the INSTALLED → CONFIGURED step. |
| **Auditability** | Plugin actions must be written to Activity with plugin_id. |
| **Tenant isolation** | Plugins must not bypass tenant isolation. RLS enforced at API layer regardless of caller. |
| **Secret isolation** | Plugins must not access secrets outside their configured scope. |
| **Crash isolation** | Failed plugins must not crash the core SIP process. Sidecar isolation (Docker containers). |
| **Cleanup on uninstall** | Plugin uninstall must define safe cleanup behavior (data retention policy). |
| **Version compatibility** | Plugin sip_version must be checked against SIP core version on install and startup. |
| **AI tool dry-run** | AI tool plugins (trust_level 4) must support dry-run mode where possible for safety verification. |
| **Vulnerability scanning** | Plugin images scanned for known CVEs before installation (Phase 4 enterprise). |

### 11.5 Extension Points

| Extension Point | Purpose | Ships In |
|----------------|---------|----------|
| `entity.custom_fields` | Add JSON Schema-validated fields to any core entity | Phase 1.5 |
| `entity.custom_entity` | Register new entity types with their own table, API, and AI indexing | Phase 1.5 |
| `api.routes` | Mount custom HTTP endpoints under `/api/v1/plugins/{plugin_name}/` | Phase 1.5 |
| `ai.tools` | Register function-calling tools the AI agent can invoke | Phase 2 |
| `ai.context_providers` | Inject additional context into RAG prompt assembly | Phase 1.5 |
| `events.subscribers` | Subscribe to event bus topics for async processing | Phase 1.5 |
| `scheduler.jobs` | Register cron jobs | Phase 1.5 |
| `auth.permissions` | Define custom RBAC permission scopes | Phase 1.5 |
| `ui.panels` | Add panels/tabs to asset and work order detail views | Phase 1.5 |
| `ui.dashboards` | Add custom dashboard widgets | Phase 2 |
| `reports.templates` | Add report definitions (PDF/CSV export) | Phase 2 |
| `notifications.channels` | Register new notification channels (Slack, Teams, SMS) | Phase 2 |
| `workflows.hooks` | Intercept work order state transitions (approval gates, custom validation) | Phase 2 |

---

## 12. Security, Permissions, and Multi-Tenancy

### 12.1 Security Architecture

SIP uses layered authorization:

| Layer | Mechanism | Description |
|-------|-----------|-------------|
| **RBAC** | Role-based permissions | 6 roles with defined permission sets (see grid below) |
| **ABAC** | Attribute-based permissions | Region-scoped access, certification-gated work orders |
| **Record-level sharing** | Explicit sharing | Specific records shared with users, vendors, or teams |
| **Field-level security** | Sensitive field masking | Cost fields, PII hidden from unauthorized users |
| **Plugin permission scopes** | Scoped API keys | Each plugin operates within an approved permission boundary |
| **AI retrieval scopes** | Authorization-bound retrieval | AI only retrieves records the requesting user can access |

**Example authorization scenarios:**
- A technician can view only assigned work orders.
- A vendor can view only work orders explicitly shared with that vendor.
- A manager can view all assets in their region.
- An auditor can view activity logs but cannot mutate operational records.
- An AI agent can only retrieve records available to the user invoking it.
- A plugin can only access API scopes approved during installation.

### 12.2 Multi-Tenancy Security

| Requirement | Detail |
|-------------|--------|
| **organization_id on all tables** | All tenant-owned tables must include organization_id unless explicitly global (e.g., system AssetTypes) |
| **RLS mandatory** | RLS must be enabled for tenant-owned tables. Integration tests must verify tenant isolation. |
| **API key scoping** | API keys scoped to organization + permission set |
| **Secret encryption** | Secrets encrypted at rest. Passwords never stored directly (bcrypt). Auth tokens never exposed to AI context. |
| **PII exclusion** | PII excluded from embeddings unless explicitly required and authorized |
| **Audit append-only** | Activity records never updated or deleted |
| **Admin auditability** | All admin actions auditable |

### 12.3 Authentication

| Requirement | Detail |
|-------------|--------|
| **Human auth** | Email + password with bcrypt hashing. JWT access tokens (15 min) + refresh tokens (7 days). |
| **API key auth** | Per-user API keys with configurable expiry and scope. Used by plugins and integrations. |
| **OAuth2/OIDC** | Phase 4 for enterprise SSO. |
| **MFA** | Phase 4. TOTP-based. |
| **Session management** | Token blacklist on logout. Refresh token rotation (new refresh token issued with each access token refresh; old refresh token invalidated). |
| **Brute force protection** | Rate-limit login attempts per IP and per email. Account lockout after N failed attempts (configurable, default 5). |

### 12.4 Authorization (RBAC)

**Roles and their permissions:**

| Permission | ADMIN | MANAGER | TECHNICIAN | VIEWER | VENDOR | AUDITOR |
|-----------|-------|---------|------------|--------|--------|---------|
| `org:manage` | ✓ | | | | | |
| `org:read` | ✓ | ✓ | ✓ | ✓ | | ✓ |
| `asset:create` | ✓ | ✓ | | | | |
| `asset:update` | ✓ | ✓ | | | | |
| `asset:delete` | ✓ | ✓ | | | | |
| `asset:read` | ✓ | ✓ | ✓ | ✓ | ✓* | ✓ |
| `work_order:create` | ✓ | ✓ | ✓ | | | |
| `work_order:update` | ✓ | ✓ | ✓** | | | |
| `work_order:delete` | ✓ | ✓ | | | | |
| `work_order:read` | ✓ | ✓ | ✓ | ✓ | ✓* | ✓ |
| `work_order:assign` | ✓ | ✓ | | | | |
| `work_order:review` | ✓ | ✓ | | | | |
| `schedule:manage` | ✓ | ✓ | | | | |
| `schedule:read` | ✓ | ✓ | ✓ | ✓ | | ✓ |
| `dispatch:manage` | ✓ | ✓ | | | | |
| `dispatch:read` | ✓ | ✓ | ✓ | ✓ | | ✓ |
| `user:manage` | ✓ | | | | | |
| `user:read` | ✓ | ✓ | ✓ | ✓ | | ✓ |
| `team:manage` | ✓ | ✓ | | | | |
| `part:manage` | ✓ | ✓ | | | | |
| `part:consume` | ✓ | ✓ | ✓ | | | |
| `part:read` | ✓ | ✓ | ✓ | ✓ | ✓* | ✓ |
| `document:upload` | ✓ | ✓ | ✓ | | ✓* | |
| `document:read` | ✓ | ✓ | ✓ | ✓ | ✓* | ✓ |
| `automation:manage` | ✓ | ✓ | | | | |
| `automation:read` | ✓ | ✓ | | | | ✓ |
| `plugin:manage` | ✓ | | | | | |
| `activity:read` | ✓ | ✓ | | | ✓ | ✓ |
| `ai:query` | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

\* Scoped to assets/work orders the vendor is assigned to via WorkOrderAssignment or explicitly shared.
\** Technicians can update work order status (accept, start, complete) and add notes/parts to assigned work orders. Cannot change type, priority, or approve completion (REVIEWED/CLOSED states restricted to MANAGER/ADMIN).

### 12.5 Multi-Tenancy Operations

| Requirement | Detail |
|-------------|--------|
| **Isolation strategy** | Row-Level Security (RLS) with `organization_id` on every table. |
| **RLS enforcement** | `SET app.current_organization_id = '<uuid>'` at the start of every request context. All queries implicitly filtered. |
| **Tenant creation** | Self-service via registration OR admin-created (configurable). |
| **Data export** | Tenants can export all their data at any time (JSON/CSV). Required for open-source trust. |
| **Tenant deletion** | Soft-delete with configurable retention period. Hard-delete after retention expires. |
| **Isolation testing** | Integration test suite must include tenant isolation tests: user in Org A must not see Org B data. |

### 12.6 Data Security

| Requirement | Detail |
|-------------|--------|
| **Encryption at rest** | PostgreSQL TDE (transparent data encryption) or filesystem-level encryption. MinIO server-side encryption for objects. |
| **Encryption in transit** | TLS 1.3 for all HTTP connections. Internal service-to-service mTLS (Phase 4 enterprise). |
| **PII handling** | User email, name, and IP addresses must not be included in AI context windows. AI agents use pseudonymous references (`User #1234`). |
| **Secret management** | API keys, LLM provider keys, DB credentials managed via environment variables or a secret manager (Vault in enterprise). Never logged. |
| **Input validation** | All API inputs validated against schemas. SQL injection prevented by parameterized queries (ORM). XSS prevented by output encoding. |
| **CORS** | Configurable per-tenant CORS policy. Default: same-origin only. |
| **Security headers** | Content-Security-Policy, X-Content-Type-Options, X-Frame-Options, Strict-Transport-Security. |
| **Dependency scanning** | CI pipeline includes `pip-audit` / `npm audit` on every PR. |

---

## 13. Data Architecture

### 13.1 Relational Database (PostgreSQL)

**Schema design principles:**
- Every table has `organization_id` with RLS policy.
- Every table has `created_at` and `updated_at` timestamps.
- UUIDs for all primary keys (no sequential IDs — prevents enumeration attacks).
- JSONB for extensible attributes, validated at the application layer.
- Foreign keys are enforced with `ON DELETE RESTRICT` (no cascading deletes — historical integrity).

**Indexing strategy:**

| Table | Index | Type | Purpose |
|-------|-------|------|---------|
| assets | (organization_id, status) | B-tree | Status filter per tenant |
| assets | (organization_id, location_id) | B-tree | Assets at a location |
| assets | (organization_id, asset_type_id) | B-tree | Assets by type |
| assets | tags | GIN | Tag search |
| assets | attributes | GIN (jsonb_path_ops) | Custom attribute queries |
| assets | parent_id | B-tree | Hierarchy traversal |
| work_orders | (organization_id, asset_id, status) | B-tree | Work orders for an asset |
| work_order_assignments | (organization_id, work_order_id) | B-tree | Assignments for a work order |
| work_order_assignments | (organization_id, assignee_type, assignee_id, status) | B-tree | Work queue for a user/team/vendor/agent |
| work_order_assignments | (organization_id, assignee_type, assignee_id, role, status) | B-tree | Role-specific assignment queries |
| work_orders | (organization_id, status, scheduled_start) | B-tree (partial: WHERE status IN ('OPEN','ASSIGNED','IN_PROGRESS')) | Upcoming work queue |
| activities | (organization_id, entity_type, entity_id, created_at DESC) | B-tree | Audit trail queries |
| documents | (entity_type, entity_id) | B-tree | Document attachments |
| schedules | (organization_id, next_due) | B-tree (partial: WHERE enabled = true) | Due schedule polling |
| outbox | (published, created_at) | B-tree (partial: WHERE published = false) | Outbox poller |

### 13.2 Vector Database (pgvector)

**Collections (MVP):**

| Collection | Source | Chunking | Dims | Use Case |
|-----------|--------|----------|------|----------|
| `work_order_notes` | WorkOrder.resolution_notes + WorkOrder.description | Per work order | 1024 | "What fixed this issue last time?" |
| `asset_metadata` | Asset.name + Asset.description + flattened Asset.attributes | Per asset | 1024 | "Find all pumps with flow rate > 100 GPM" |
| `document_chunks` | Document.text_content | 512 tokens, 64 overlap | 1024 | "What is the torque spec?" |
| `inspection_findings` | InspectionChecklistItem.finding (non-null) | Per finding | 1024 | "What recurring inspection failures exist?" |

**Index:** HNSW index on each collection's embedding column. `m = 16`, `ef_construction = 200`.

### 13.3 Graph Strategy

**Phase 1–3:** No dedicated graph database. Use relational adjacency lists + recursive CTEs for asset and location hierarchy traversal. Materialized path column (ltree) added for fast subtree queries:

```sql
-- Add to assets table
ALTER TABLE assets ADD COLUMN path ltree;

-- Query all descendants of Asset X
SELECT * FROM assets WHERE path <@ 'root.parent.child';
```

**Phase 4:** Neo4j for full knowledge graph + dependency graph. Multi-hop impact analysis. Dependency visualization. Asset relationship discovery. Deferred until multi-hop impact analysis becomes a required enterprise feature.

### 13.4 Transactional Outbox Pattern

**How it works:**

```
1. Application service begins a database transaction.
2. Performs the business write (INSERT/UPDATE on asset, work_order, etc.).
3. In the SAME transaction, INSERTs into the `outbox` table:
   { id, aggregate_type, aggregate_id, event_type, payload (JSONB), published=false }
4. Commits transaction. Both writes succeed or both fail — atomic.
5. OutboxPoller (background process, runs every 100ms):
   SELECT * FROM outbox WHERE published = false ORDER BY created_at LIMIT 100
   FOR EACH row:
     Publish to event bus (in-process queue in MVP, NATS JetStream post-MVP)
     UPDATE outbox SET published = true WHERE id = row.id
6. Sync Workers consume events:
   - VectorIndexer: Embeds new/changed text, upserts into pgvector collections.
   - ActivityLogger: Writes Activity record.
   - NotificationWorker: Sends email/push notifications.
```

**Idempotency:** All sync workers are idempotent — they upsert by entity ID, so replaying an event produces the same result.

**Atomic activity writes:** For direct user/API mutations, Activity records are written in the same database transaction as the business change and outbox event. This guarantees the audit trail exists even if the outbox worker fails.

Business transaction writes (all atomic):
1. Business entity change (INSERT/UPDATE).
2. Activity record.
3. Outbox event.

All three succeed or fail atomically. Derived async events from workers may create additional Activity records.

**Outbox table:**
```sql
CREATE TABLE outbox (
    id BIGSERIAL PRIMARY KEY,
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_id UUID NOT NULL,
    event_type VARCHAR(200) NOT NULL,
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING',
    published BOOLEAN NOT NULL DEFAULT false,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    next_attempt_at TIMESTAMPTZ,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_outbox_published ON outbox (status, next_attempt_at)
    WHERE status IN ('PENDING', 'FAILED');
```

**Outbox statuses:**
- **PENDING:** Ready for processing.
- **PROCESSING:** Worker has claimed the event.
- **PUBLISHED:** Successfully delivered (terminal).
- **FAILED:** Processing failed, will retry.
- **DEAD_LETTER:** Exceeded max attempts, requires operator review.

Outbox workers retry failed events with exponential backoff. After max attempts (default 10), events move to DEAD_LETTER.

### 13.5 Data Classification for AI Access

| Category | AI Access | Constraint |
|----------|-----------|------------|
| Asset metadata | Direct (vector search) | No restrictions |
| Work order notes/descriptions | Direct (vector search) | Organization-scoped only |
| Document text | Direct (vector search) | Organization-scoped + visibility-filtered (PRIVATE_TENANT, SHARED_VENDOR, PUBLIC, SYSTEM_DEFAULT) |
| Inspection findings | Direct (vector search) | Organization-scoped only |
| Parts inventory (name, stock) | Via tool call | Read-only |
| User names/emails | Restricted | Anonymized in AI context (`User #1234`) |
| Activity log | Via tool call | Read-only, scoped |
| Auth data (passwords, tokens) | Never | Firewalled at API layer |
| Financial/cost data | Via tool call (Phase 2) | Permission-gated |
| Cross-tenant data | Never | RLS prevents even accidental access |

---

## 14. System Architecture

### 14.1 Deployment Architecture (MVP)

```
┌──────────────────────────────────────────────────────────┐
│                    Docker Host (single node)               │
│                                                           │
│  ┌─────────────────┐  ┌─────────────────┐                │
│  │  SIP API        │  │  SIP Frontend   │                │
│  │  (FastAPI)      │  │  (Next.js)      │                │
│  │  Port 8000      │  │  Port 3000      │                │
│  └────────┬────────┘  └─────────────────┘                │
│           │                                               │
│  ┌────────┴────────────────────────────────────────┐     │
│  │              SIP Core Services                    │     │
│  │  ┌──────────┐ ┌─────────┐ ┌──────────────────┐  │     │
│  │  │ Scheduler │ │ Outbox  │ │ AI Service       │  │     │
│  │  │ (cron)    │ │ Poller  │ │ (RAG + LLM)      │  │     │
│  │  └──────────┘ └─────────┘ └──────────────────┘  │     │
│  └──────────────────────────────────────────────────┘     │
│           │                                               │
│  ┌────────┴────────────────────────────────────────┐     │
│  │            Data Layer                             │     │
│  │  ┌──────────┐ ┌────────┐ ┌──────────────────┐   │     │
│  │  │PostgreSQL│ │ Redis  │ │ MinIO            │   │     │
│  │  │+ pgvector│ │        │ │ (Docs/Photos)    │   │     │
│  │  └──────────┘ └────────┘ └──────────────────┘   │     │
│  └──────────────────────────────────────────────────┘     │
│                                                           │
│  ┌──────────────────────────────────────────────────┐     │
│  │            Plugin Sidecars                         │     │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐        │     │
│  │  │ Plugin A │  │ Plugin B │  │ Plugin C │        │     │
│  │  └──────────┘  └──────────┘  └──────────┘        │     │
│  └──────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────┘
```

### 14.2 API Design

**Base URL:** `/api/v1`

**Resource endpoints (MVP):**

```
POST   /api/v1/auth/login
POST   /api/v1/auth/refresh
POST   /api/v1/auth/logout

GET    /api/v1/assets
POST   /api/v1/assets
GET    /api/v1/assets/{id}
PATCH  /api/v1/assets/{id}
DELETE /api/v1/assets/{id}
GET    /api/v1/assets/{id}/children
GET    /api/v1/assets/{id}/work-orders

GET    /api/v1/asset-types
POST   /api/v1/asset-types
GET    /api/v1/asset-types/{id}
PATCH  /api/v1/asset-types/{id}

GET    /api/v1/locations
POST   /api/v1/locations
GET    /api/v1/locations/{id}
PATCH  /api/v1/locations/{id}
GET    /api/v1/locations/{id}/assets

GET    /api/v1/work-orders
POST   /api/v1/work-orders
GET    /api/v1/work-orders/{id}
PATCH  /api/v1/work-orders/{id}
DELETE /api/v1/work-orders/{id}
POST   /api/v1/work-orders/{id}/parts

GET    /api/v1/schedules
POST   /api/v1/schedules
GET    /api/v1/schedules/{id}
PATCH  /api/v1/schedules/{id}
DELETE /api/v1/schedules/{id}

GET    /api/v1/inspections
GET    /api/v1/inspections/{id}
PATCH  /api/v1/inspections/{id}/items

GET    /api/v1/parts
POST   /api/v1/parts
GET    /api/v1/parts/{id}
PATCH  /api/v1/parts/{id}

GET    /api/v1/users
POST   /api/v1/users
GET    /api/v1/users/{id}
PATCH  /api/v1/users/{id}

GET    /api/v1/teams
POST   /api/v1/teams
GET    /api/v1/teams/{id}
PATCH  /api/v1/teams/{id}

POST   /api/v1/documents
GET    /api/v1/documents/{id}
DELETE /api/v1/documents/{id}

GET    /api/v1/activities

POST   /api/v1/ai/chat              (SSE streaming response)
GET    /api/v1/ai/conversations

GET    /api/v1/plugins
POST   /api/v1/plugins
GET    /api/v1/plugins/{id}
PATCH  /api/v1/plugins/{id}
DELETE /api/v1/plugins/{id}

GET    /api/v1/health
GET    /api/v1/health/ready
```

**API conventions:**
- All list endpoints support: `?page=1&per_page=50&sort=created_at&order=desc`
- All list endpoints support: `?search=<query>` for basic text search
- All list endpoints support: `?filter[status]=OPEN&filter[priority]=HIGH` for field filtering
- POST/PATCH bodies validated against JSON Schema
- Error responses: `{ "error": { "code": "VALIDATION_ERROR", "message": "...", "details": [...] } }`

### 14.3 Service Boundaries (Monolith with Clear Module Separation)

```
sip/
├── core/
│   ├── auth/           # Authentication, JWT, API keys, RBAC
│   ├── assets/         # Asset + AssetType + Location CRUD
│   ├── work_orders/    # WorkOrder + Inspection + Checklist CRUD
│   ├── schedules/      # Schedule CRUD + Trigger evaluation
│   ├── parts/          # Part CRUD + Stock tracking + BOM
│   ├── users/          # User + Team CRUD + Certifications
│   ├── documents/      # Upload, storage, text extraction
│   ├── activities/     # Immutable audit log
│   └── plugins/        # Plugin registry, lifecycle, extension points
├── ai/
│   ├── embeddings/     # Embedding generation pipeline
│   ├── retriever/      # Vector + relational context retrieval
│   ├── prompt_builder/ # Prompt assembly + formatting
│   ├── llm/            # LiteLLM integration, streaming
│   └── tools/          # Tool registry + execution (Phase 2)
├── infrastructure/
│   ├── db/             # Database models, migrations, RLS
│   ├── events/         # Outbox publisher + consumer framework
│   ├── cache/          # Redis client
│   ├── storage/        # MinIO/S3 client
│   └── notifications/  # Email/push dispatch
├── api/
│   └── v1/             # API route definitions
└── shared/
    ├── models/         # Pydantic models shared across modules
    ├── errors/         # Error codes, exception classes
    └── utils/          # Common utilities
```

Each module:
- Has its own service layer, repository layer, and API routes.
- Depends on other modules only through their public service interfaces (not their database tables directly).
- Registers its own database migrations in a module-specific migrations directory.

---

## 15. Non-Functional Requirements

SIP must serve as a reliable **maintenance intelligence layer** across deployment scales — from a single-node Docker Compose instance for a small shop to a horizontally-scaled multi-tenant platform for enterprise service providers.

### 15.1 Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| API latency (p95) for CRUD operations | < 300ms | Server-side timing, excludes network |
| Search queries initial results | < 1 second for normal tenant data sizes | From query to first page |
| AI query latency (p95, time to first token) | < 2 seconds | From POST to first SSE chunk |
| Work order list (100 items) | < 300ms | With sorting and basic filters |
| Asset hierarchy (100 nodes) | < 500ms | Full recursive tree |
| AI responses | Stream partial output as soon as available | SSE chunk interval |
| Concurrent users (MVP) | 50 simultaneous active users | Load test with k6 |
| Work order throughput | 1,000 work orders created per minute | Bulk schedule trigger scenario |

**MVP scale targets:**
- 1 organization per local deployment
- 10k assets
- 100k work orders
- 10k documents
- 100 users

**Enterprise scale targets (Phase 4):**
- Multi-tenant hosted deployment
- 1M+ assets
- 10M+ work orders
- 1M+ documents

### 15.2 Availability

| Metric | Target |
|--------|--------|
| Uptime (self-hosted) | Best-effort for open-source. 99.9% for enterprise hosting. |
| Recovery Time Objective (RTO) | < 1 hour (single-node Docker Compose restart) |
| Recovery Point Objective (RPO) | < 5 minutes (daily + WAL backups) |
| Graceful degradation | AI queries return "AI service unavailable" instead of 500 errors if LLM unreachable. Core CRUD remains functional. |
| Single-node operation | Local OSS deployment must support single-node operation. |
| Horizontal scaling | Hosted enterprise deployment must support horizontal scaling (Phase 4). |
| Background workers | Background workers must be restart-safe and idempotent. |

### 15.3 Backup and Recovery

| Requirement | Detail |
|-------------|--------|
| Database backups | Provide documented backup and restore process for Postgres. Daily full backup + WAL archiving. |
| Object storage backups | Provide documented backup process for object storage. MinIO mirror or S3 replication. |
| Vector index rebuild | Document how to rebuild vector indexes from source records. |
| Graph index rebuild | Document how to rebuild graph indexes from source records (Phase 4). |
| Backup verification | Automated restore test weekly. |
| Tenant self-service export | Admin can export full tenant data as JSON archive. |

### 15.4 Observability

| Requirement | Detail |
|-------------|--------|
| Structured logging | JSON-formatted logs. Include: timestamp, level, service, trace_id, user_id (pseudonymized), org_id, message. |
| Distributed tracing | OpenTelemetry spans across API → Service → DB → AI Service → LLM → Plugin execution. Trace IDs propagated in headers. |
| Metrics | Prometheus endpoint at `/metrics`. Core metrics: request count, latency percentiles, error rate, AI token usage, outbox lag, DB connection pool utilization, embedding failures, plugin failures, document indexing status, queue depth, failed jobs. |
| Request IDs | Provide request IDs across API, worker, AI, and plugin execution. |
| Health checks | `/api/v1/health` (liveness). `/api/v1/health/ready` (readiness: DB + Redis + MinIO + Ollama reachable). |

### 15.5 Accessibility and Internationalization

| Requirement | Detail |
|-------------|--------|
| **Accessibility** | Web UI should target WCAG 2.1 AA. Keyboard navigation supported for core workflows. Color not the only communication channel for status. |
| **Time zones** | Configurable per organization and per user. All timestamps stored as timestamptz (UTC). |
| **Units of measure** | Configurable per organization. Support imperial and metric. |
| **Locale-aware formatting** | Support locale-aware date and number formatting. |
| **Language** | UI and AI responses in configurable language. Multi-lingual AI from Phase 1. |

### 15.6 Compliance Readiness

| Standard | SIP Capability |
|----------|---------------|
| SOC 2 | Design toward SOC 2 readiness. Support audit exports. Immutable Activity history. Retention policies. Tenant data export. |
| ISO 55000 (Asset Management) | Asset lifecycle tracking, audit trail, maintenance scheduling, terminology alignment |
| ISO 27001 (InfoSec) | RLS, encryption, audit logging, RBAC |
| FDA 21 CFR Part 11 / HIPAA | Electronic signatures (Phase 4), immutable audit trail, PII handling. Keep HIPAA/FDA-adjacent auditability in mind even if MVP does not claim compliance. |
| OSHA | Inspection checklists, compliance evidence storage |
| GDPR | Data export, tenant deletion, PII pseudonymization in AI context |

### 15.7 Testing Requirements

| Test Type | Coverage Target | Tool |
|-----------|----------------|------|
| Unit tests (domain logic, state machines) | > 80% line coverage on service layer | pytest |
| Integration tests (API endpoints) | All API endpoints with test database | pytest + httpx |
| Tenant isolation tests | Every tenant-owned entity: prove Org A cannot access Org B records through API, AI retrieval, plugin execution, search, or export | pytest |
| Permission boundary tests | Verify each role can/cannot perform expected actions | pytest |
| Work order lifecycle tests | All valid and invalid state transitions | pytest |
| Outbox idempotency tests | Replay events and verify no duplicate side effects | pytest |
| Document ingestion tests | Full pipeline: upload → extract → embed → index | pytest |
| AI citation accuracy tests | Verify citations reference real, accessible records | Custom eval harness |
| AI authorization tests | Verify AI does not surface data outside user's permission scope | Custom eval harness |
| Plugin compatibility tests | Manifest validation, permission scoping | pytest |
| Plugin permission tests | Verify plugin cannot exceed declared permissions | pytest |
| Migration tests | Forward and rollback migrations | pytest |
| Load tests | Asset and work order queries at scale | k6 |

**Tenant isolation test requirement (mandatory):**
Every tenant-owned entity must have automated tests proving that users from Organization A cannot access Organization B records through API, AI retrieval, plugin execution, search, or export. CI/CD blocks merge on failure.

---

## 16. Success Metrics

### 16.1 Phase 1: MVP Core Success Criteria

| # | Criterion | Target |
|---|-----------|--------|
| SM1 | A new AssetType can be created and used | < 5 minutes, no code changes |
| SM2 | End-to-end asset + work order flow | Create asset, create WO, complete WO, view full history |
| SM3 | AI answers basic asset history questions | With cited source records |
| SM4 | AI citation accuracy on eval set | > 95% |
| SM5 | Tenant isolation tests pass | Across API and AI retrieval paths |
| SM6 | Docker Compose deployment | Works from a clean checkout |

### 16.2 Phase 1.5 Success Criteria

| # | Criterion | Target |
|---|-----------|--------|
| SM7 | Build and register a simple read-only plugin | < 1 day |
| SM8 | Document lifecycle: upload → extract → index → AI cite | Fully automated pipeline |
| SM9 | Basic automations run from system events | At least 3 built-in automations active |
| SM10 | LLM provider swap | Through configuration only, no code changes |

### 16.3 Phase 2 Success Criteria

| # | Criterion | Target |
|---|-----------|--------|
| SM11 | Inspection templates support structured checklist completion | Drag-and-drop builder shipped |
| SM12 | Inventory tracks part consumption against work orders | Real-time stock updates |
| SM13 | Basic PM schedules generate work orders reliably | < 1% missed triggers over 30 days |
| SM14 | Dashboard shows MTTR, overdue work, work order status, asset status | All widgets functional |

### 16.4 Long-Term Success Metrics

| # | Metric | Target |
|---|--------|--------|
| SM15 | GitHub stars | 1,000 within 6 months of public release |
| SM16 | Community contributors | 20 unique contributors within 6 months |
| SM17 | Community-contributed plugins | 10 plugins within 12 months |
| SM18 | Enterprise deployments | 5+ paying enterprise customers within 18 months |
| SM19 | AI-driven reduction in downtime | Measurable reduction for pilot customers vs baseline |
| SM20 | Repeat failure reduction | Reduction in repeat work orders for same failure pattern |

---

## 17. Execution Plan

SIP ships in five phases. Each phase builds on the previous one without compromising the **service-domain canonical model**. The model is designed upfront; features are layered onto it progressively.

### Phase 1: MVP Core (Weeks 1-8)

**Goal:** A working, self-hostable open-source service intelligence platform with limited AI Q&A that a single organization can deploy via Docker Compose and immediately explore with seed data.

**Deliverables:**

| # | Deliverable | Details |
|---|-------------|---------|
| D1 | Database schema + RLS | Full relational schema for all MVP entities. Row-level security for multi-tenancy. UUIDv7 primary keys. ltree materialized paths for asset hierarchy. |
| D2 | Auth service | JWT login/refresh/logout. 6 RBAC roles. API key support for future integrations. |
| D3 | Organization + Location CRUD | Tenant-aware CRUD with hierarchy. Geo support. |
| D4 | AssetType + Manufacturer + AssetModel CRUD | JSON Schema validation for custom attributes. Seed data: 6 AssetTypes, 9+ manufacturers, 25+ models. |
| D5 | Asset registry | Full CRUD with dynamic attributes, hierarchy, status state machine, tagging, location assignment. |
| D6 | User + Team CRUD | User management with role assignment, skills, certifications with expiry. Team management with lead. |
| D7 | Work order management | Full lifecycle state machine. Status transitions with validation. Assignment to user/team. Time tracking. Resolution notes. Parts consumption. |
| D8 | Schedule engine | Cron-based trigger evaluation (polling every 60s). Auto-generate work orders. Deduplication. Next due calculation. |
| D9 | Inspection engine | Checklist templates. Pass/fail/numeric/text/photo items. Auto-generate corrective WO on failure. |
| D10 | Basic parts inventory | Part CRUD. Stock tracking. Low stock alerts. Consumption logging against work orders. |
| D11 | Document upload + processing | Upload to MinIO. Text extraction via Tika or similar. Processing status tracking (PENDING → EXTRACTING → EMBEDDING → READY/FAILED). |
| D12 | AI Knowledge Agent v1 | Embedding pipeline with pgvector. RAG retrieval over work order notes + document text. Source citations in responses. SSE streaming. "I don't know" honesty. Read-only. |
| D13 | Activity audit log | Immutable append-only log. Actor identity (human/AI agent/system). Before/after diffs. Filterable query API. |
| D14 | Docker Compose deployment | Single `docker compose up`. Includes all services. Seed data auto-loads on first run. |
| D15 | Open-source release | AGPLv3 license. README, CONTRIBUTING guide, API documentation. Docker image published. |

**Phase 1 weekly breakdown:**

| Week | Focus |
|------|-------|
| 1-2 | DB schema, migrations, RLS policies. Auth service (JWT + RBAC). API skeleton. Docker Compose setup. |
| 3-4 | Organization, Location, AssetType, Manufacturer, AssetModel CRUD. JSON Schema validation. Seed data. User + Team CRUD. Tenant isolation tests. |
| 5-6 | Asset registry. WorkOrder CRUD + state machine. Schedule engine (cron). Inspection + checklist. Parts CRUD + consumption. |
| 7 | Document upload + processing status. AI embedding pipeline + pgvector. RAG retrieval + SSE streaming. AI eval harness. |
| 8 | Activity audit log. Polish, bug fixes, documentation. Docker Compose release. Seed data finalization. |

**Phase 1 risks:**
- AI accuracy below acceptable threshold → Mitigation: eval harness from Day 1 of AI work (Week 7). Stretch goal to add re-ranking in Week 8 if accuracy is weak.
- Multi-tenant data leak → Mitigation: dedicated tenant isolation test suite from Week 3 onward. CI/CD blocks merge on failure.
- Scope creep on asset type flexibility → Mitigation: 6 reference AssetTypes ship as seed data. Custom AssetType builder is UI-only work; the schema system is built in Weeks 3-4.

---

### Phase 1.5: AI and Plugin Expansion (Weeks 9-14)

**Goal:** Harden the AI layer, ship the plugin framework Foundation, and add basic automation. The platform transitions from "AI as a demo feature" to "AI as a reliable tool." Plugins become possible.

**Deliverables:**

| # | Deliverable | Details |
|---|-------------|---------|
| D16 | Plugin manifest and lifecycle foundation | Manifest validation (plugin.json). Plugin registry. Sidecar deployment model. Permission scoping. Lifecycle: INSTALL → RUNNING → STOPPED → UNINSTALLED. |
| D17 | Reference plugins (2) | CSV import plugin (bulk asset + work order import). Basic asset dashboard plugin (health overview, status distribution). |
| D18 | Basic UI extension points | `ui.panels` (tabs on asset/WO detail). `ui.dashboards` (custom widgets). Plugin-mounted API routes under `/api/v1/plugins/{name}/`. |
| D19 | LLM provider abstraction | LiteLLM integration for multi-provider swap. Per-tenant LLM configuration. Fallback chain (Ollama → OpenAI → Anthropic). Token usage tracking. |
| D20 | Improved RAG pipeline | Hybrid search (keyword + vector). Cross-encoder re-ranking. Expanded context retrieval (relational facts + vector + graph). |
| D21 | Document chunking and embedding | 512-token chunks with overlap. Per-chunk embeddings for large documents. Recursive chunking for structured documents. |
| D22 | Optional external event bus | NATS JetStream integration (optional — in-process outbox still supported for single-node). Enables multi-node deployments with distributed event processing. |
| D23 | Basic automation engine | Event-driven automation rules: "When asset status changes to DOWN, create corrective WO." "When inspection fails, notify manager." Configurable per tenant. |
| D24 | API key scoping improvements | Fine-grained API key permissions. Key rotation. Usage audit. Plugin-specific keys auto-generated on install. |

**Phase 1.5 risks:**
- Plugin sidecar isolation complexity → Mitigation: ship with Docker Compose network isolation. Minimal attack surface from day one.
- RAG improvements may not significantly boost accuracy → Mitigation: benchmark before/after with eval set. Only ship improvements that move the needle.

---

### Phase 2: Core Operations (Weeks 15-22)

**Goal:** Complete the core service intelligence feature set. The platform is now fully capable of running a real maintenance operation with **auditable AI-assisted maintenance workflows**.

**Deliverables:**

| # | Deliverable | Details |
|---|-------------|---------|
| D25 | Meter-based scheduling | Runtime/cycle meter triggers. Meter reading API for manual or automated entry. Meter-based next_due calculation. |
| D26 | Full parts/inventory management | Advanced inventory: reorder workflows, vendor management, cost tracking, stock adjustments, cycle counts. |
| D27 | Notification service | Email, push (web push), webhook notifications. Configurable per user. Templates for common events (WO assigned, WO overdue, part low stock). |
| D28 | Dashboard & analytics | MTTR (Mean Time to Repair), MTBF (Mean Time Between Failures), asset health scores, work order completion rates, backlog trends, technician productivity. |
| D29 | AI tool execution | AI can create work orders (with confirmation), update work order status (with confirmation), query parts inventory. Every mutating action requires explicit human approval in the UI. |
| D30 | Inspection template builder UI | Drag-and-drop checklist builder. Reusable inspection templates. Template versioning. |
| D31 | Basic mobile PWA | Offline-capable via service worker. Camera integration for inspection photos. Push notifications. Responsive design optimized for field use. |

---

### Phase 3: Advanced AI (Weeks 23-30)

**Goal:** AI becomes proactive, not just reactive. The platform starts generating insights that humans wouldn't catch on their own.

**Deliverables:**

| # | Deliverable | Details |
|---|-------------|---------|
| D32 | Predictive maintenance ML models | Failure prediction models trained on work order + inspection history. Anomaly detection on condition data. Confidence scoring. |
| D33 | Smart scheduling optimization | AI-optimized PM schedules balancing technician availability, part inventory, asset criticality, and predicted failure windows. Human approval required. |
| D34 | Failure pattern detection | AI identifies recurring failure patterns across asset types and recommends preventive schedule changes. Cross-tenant anonymized pattern sharing (opt-in). |
| D35 | Compliance agent | Automated regulatory gap analysis. Evidence package generation. Audit readiness scoring. Configurable rule sets (ISO 55000, OSHA, FDA Part 11). |
| D36 | AI coaching | During work order execution, AI suggests: "The last 3 times this issue occurred, the root cause was X. Check Y before proceeding." Contextual, non-intrusive. |
| D37 | Advanced RAG with function calling | Complex multi-step AI queries: "Analyze all pump failures in Building B over the last year and summarize root causes." AI executes multiple tool calls autonomously to assemble answer. |

---

### Phase 4: Enterprise Scale (Weeks 31-40)

**Goal:** SIP becomes a platform that can serve enterprise customers at scale with advanced features, dedicated infrastructure, and cross-organization capabilities.

**Deliverables:**

| # | Deliverable | Details |
|---|-------------|---------|
| D38 | Predictive maintenance ML models (production-grade) | Model versioning, A/B testing, automated retraining pipelines, model performance monitoring. |
| D39 | Autonomous dispatch | AI auto-assigns work orders by technician skills, location, availability, and current workload. Human override window before dispatch becomes active. |
| D40 | Full plugin marketplace | Plugin directory with ratings, reviews, version compatibility. Automated testing against current SIP version. Paid plugin support. |
| D41 | Native mobile applications | React Native or Flutter apps for iOS/Android. Full offline support with conflict resolution. Barcode/QR scanning. Advanced camera integration. |
| D42 | Advanced offline-first technician experience | Offline asset lookup. Offline work order execution with sync queue. Conflict resolution for concurrent edits. Background sync. |
| D43 | Multi-organization vendor portal | Vendors log into a single account and see work orders across all client organizations. Cross-tenant data sharing with explicit authorization. |
| D44 | Dedicated graph database (Neo4j) | Full knowledge graph + dependency graph. Multi-hop impact analysis. Dependency visualization. Asset relationship discovery. |
| D45 | Enterprise billing and hosted SaaS control plane | Multi-tenant orchestration. Usage-based billing. Organization provisioning and deprovisioning. Admin dashboard for SIP operators. |
| D46 | Advanced regulatory compliance packages | Pre-built compliance rule sets for ISO 55000, ISO 27001, FDA 21 CFR Part 11, OSHA 1910, NFPA 70E. Compliance scorecards. |
| D47 | Multi-modal inspection photo analysis | AI analyzes inspection photos: detects corrosion, wear, leaks, missing guards. Flags anomalies for human review. |
| D48 | Cross-tenant shared analytics | Anonymized benchmarking: "How does your MTTR compare to similar organizations?" Opt-in data sharing for industry-wide insights. |
| D49 | Enterprise SSO + advanced RBAC | SAML/OIDC integration. Custom role creation. Attribute-based access control (ABAC). Audit log export for SIEM integration. |
| D50 | Federation / cross-org data sharing | Explicit asset sharing between organizations. Shared work order visibility. Supply chain maintenance coordination. |

### Phase Dependency Graph

```
Phase 1: MVP Core
    │
    ▼
Phase 1.5: AI & Plugin Expansion
    │
    ├──────────────────────────┐
    ▼                          ▼
Phase 2: Core Operations    Phase 3: Advanced AI
    │                          │
    └──────────┬───────────────┘
               ▼
Phase 4: Enterprise Scale
```

Phases 2 and 3 can be developed in parallel by separate teams since they touch different parts of the system. Phase 4 depends on the foundations from both 2 and 3.

### Phase 1 Dependency Graph (Detailed)

```
Auth Service ──► Organization ──► Location
                    │
                    ├──► AssetType ──► Manufacturer ──► AssetModel
                    │                                        │
                    ├──► User ──► Team                       │
                    │                                        │
                    └────────────────────────────────────► Asset Registry
                                                               │
                                              ┌────────────────┤
                                              ▼                ▼
                                         WorkOrder         Document
                                         CRUD              Upload
                                              │                │
                                    ┌─────────┼────────┐       │
                                    ▼         ▼        ▼       ▼
                                 Schedule Inspection Parts  AI Embedding
                                    │         │        │       │
                                    └─────────┴────────┴───┬───┘
                                                           ▼
                                                    AI Knowledge Agent
                                                           │
                                                           ▼
                                                    Activity Audit Log
                                                           │
                                                           ▼
                                                    Docker Release
                                                    + Seed Data

---

## 18. Risks and Mitigations

### 18.1 Technical Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| **AI hallucination undermines trust** | Critical | Medium | Require citations, eval harness from Day 1, structured response with `unsupported_claims` field, confidence scoring, "I don't know" as valid response. |
| **AI leaks unauthorized data** | Critical | Medium | Enforce permission-aware retrieval. AI goes through service layer with RLS, not raw DB. Dedicated tenant isolation tests across AI paths. |
| **Multi-tenant data leak via RLS misconfiguration** | Critical | Medium | Dedicated tenant isolation test suite. Mandatory code review on any query that doesn't go through RLS-enabled ORM. Tests cover API, AI retrieval, plugin execution, search, and export. |
| **Plugin security vulnerability compromises host** | High | Medium | Trust levels enforce graduated risk. Sidecar isolation (Docker). Scoped API keys. Manifest permission declarations enforced at runtime. AI tool plugins require dry-run support. |
| **"Any asset" flexibility creates unmanageable data** | Medium | High | Ship 6 well-designed reference AssetTypes with manufacturers and models. Schema governance through JSON Schema validation. Community contributions expand catalog. |
| **pgvector performance at scale** | Medium | Low | pgvector HNSW scales to 10M+ vectors. Migration path to Qdrant/Milvus documented. |
| **Event bus and graph DB too much operational complexity** | Medium | Low | Phase 1: lite in-process outbox workers and relational hierarchy (ltree). Graph DB deferred to Phase 4. Event bus optional until Phase 1.5. |
| **Embedding model quality regresses on domain data** | Medium | Low | Configurable embedding model. Eval set benchmarks model quality before upgrades. Re-indexing API available. |

### 18.2 Product Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| **MVP scope too broad** | High | Low | MVP Core is narrowly defined (16 deliverables). Graph DB, marketplace, predictive AI, native mobile, advanced RAG all deferred. If behind at Week 4, cut inspections or parts to Phase 1.5. |
| **Too generic — no PMF in any single industry** | High | Medium | Ship reference AssetTypes, manufacturers, and models for 6 common domains. Partner with 2-3 design partners for real-world validation before public release. |
| **Open-source adoption without enterprise revenue** | High | Medium | AGPLv3 ensures hosted competitors must contribute back. Enterprise hosting + support as paid tier. Enterprise plugins can be proprietary. |
| **AI answers may hallucinate despite safeguards** | Medium | Medium | Eval harness must catch regressions before release. `unsupported_claims` field provides transparency for users and automated monitors. |
| **Plugin ecosystem never materializes** | Medium | High | Plugin SDK must be excellent. Ship 2 reference plugins in Phase 1.5. Trust level system makes plugin safety reviewable. Consider bounty program for early plugins. |

### 18.3 Execution Risks

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| **8-week Phase 1 timeline too aggressive** | Medium | High | Phase 1 scope intentionally narrow — 15 specific deliverables. If behind at Week 4, defer document processing status or parts consumption to Phase 1.5. |
| **Auth complexity underestimated** | Medium | High | Start auth in Week 1. Use well-tested libraries. Don't build custom auth. RBAC model defined in PRD; implement to spec. |
| **Frontend lags behind backend** | Low | Medium | API-first development. Frontend must ship functional but not polished. Polish in Phase 1.5. |
| **AssetType flexibility becomes chaos** | Medium | Medium | Ship 6 well-designed default AssetTypes with manufacturers and models. Schema governance through JSON Schema. Community-driven expansion later. |
| **Plugin framework premature without stable domain model** | Medium | Low | Plugin framework deferred to Phase 1.5 — after the **service-domain canonical model** has been exercised in Phase 1. Extension points designed against a proven API. |

---

## 19. Open Questions

These require explicit decisions before implementation begins:

| # | Question | Context |
|---|----------|---------|
| OQ1 | **Plugin execution runtime:** Should plugins use sidecars, WASM, or both? | Sidecar for Phase 1.5 flexibility. WASM for high-security environments in Phase 4. |
| OQ2 | **First supported backend language for plugins?** | Python (native). Sidecar model enables any language. |
| OQ3 | **AssetType schema versioning:** How are migrations handled when a schema changes? | Schema-on-read with soft validation warnings for Phase 1. Immutable versioning for Phase 2. |
| OQ4 | **Should AI embeddings include any PII by default?** | No. PII excluded from embeddings unless explicitly authorized. |
| OQ5 | **What is the minimum useful UI for Phase 1 MVP?** | Asset/WO creation, WO queue, basic dashboards. Mobile-responsive. PWA from the start? |
| OQ6 | **Should SIP ship with PWA from Phase 1?** | PWA considered for Phase 2 (Core Operations). Phase 1 is desktop-responsive web UI. |
| OQ7 | **What default AssetTypes + Manufacturers + Models ship with MVP?** | 6 AssetTypes, 9+ manufacturers, 25+ models defined. |
| OQ8 | **What document formats are supported in Phase 1?** | PDF, plain text, images (with OCR placeholder for Phase 1.5). |
| OQ9 | **What is the first target pilot industry?** | Manufacturing or facilities management — highest density of physical assets. |
| OQ10 | **How should public/shared knowledge be governed?** | Documents with visibility = PUBLIC available across tenants. Curated by SIP maintainers + community PRs. |
| OQ11 | **Should vendors have cross-tenant accounts or tenant-specific identities?** | Tenant-specific for Phase 1-3. Multi-org vendor portal in Phase 4. |
| OQ12 | **What data can be shared across organizations for community intelligence?** | Opt-in anonymized data sharing for benchmarking and ML training in Phase 4. Explicit authorization required. |
| OQ13 | **LLM provider default for open-source users?** | Ollama (free, local, no API key). LiteLLM config to swap. |
| OQ14 | **Should SIP support meter-based scheduling in Phase 1?** | No. Meter-based scheduling deferred to Phase 2. MVP is cron-only. |

---

## 20. Appendices and ADRs

### Appendix A: Architecture Decision Records

The following ADRs should be created in `docs/adr/` before implementation:

| ADR # | Title | Decision |
|-------|-------|----------|
| ADR-001 | Modular monolith before microservices | Start as single deployable unit with strict module boundaries; extract services only when scale demands it |
| ADR-002 | PostgreSQL as primary operational database | PostgreSQL 16+ for relational data, pgvector for vectors, ltree for hierarchy |
| ADR-003 | Row-level security for tenant isolation | RLS with organization_id on every table; dedicated tenant isolation test suite |
| ADR-004 | Transactional outbox over dual writes | Outbox pattern for reliable event publishing; atomic with business writes |
| ADR-005 | Lite outbox worker before required event bus | In-process outbox poller for Phase 1; NATS JetStream optional in Phase 1.5 |
| ADR-006 | pgvector for MVP vector search | Stack consolidation; migration path to Qdrant/Milvus documented |
| ADR-007 | Defer dedicated graph database | Recursive CTEs + ltree for Phase 1-3; Neo4j at Phase 4 when dependency graph use cases required |
| ADR-008 | Public API as AI surface | AI agents consume the same REST API as the frontend; no internal AI backdoors |
| ADR-009 | AI answers require citations | Structured response format with sources, confidence, unsupported_claims; eval harness required before release |
| ADR-010 | Plugin trust levels and permission scopes | 5-level trust classification; graduated approval process; sidecar isolation |
| ADR-011 | AGPLv3 licensing rationale | Open source with copyleft; enterprise hosting + support as business model |
| ADR-012 | LLM provider abstraction | LiteLLM proxy; multi-provider support; Ollama default for open-source; provider swap via config |

### Appendix B: Glossary

| Term | Definition |
|------|-----------|
| **CMMS** | Computerized Maintenance Management System — legacy category of software for managing maintenance operations. SIP is the next generation: a **physical asset intelligence** platform. |
| **PM** | Preventive Maintenance — scheduled, recurring maintenance to prevent failures |
| **CM** | Corrective Maintenance — repair work triggered by a failure or issue |
| **Work Order** | A unit of maintenance work with defined scope, assignee, and lifecycle |
| **RAG** | Retrieval-Augmented Generation — AI pattern that retrieves relevant context before generating a response |
| **RLS** | Row-Level Security — PostgreSQL feature that filters rows per query based on session variables |
| **Outbox Pattern** | Reliable event publishing: events are written to a DB table in the same transaction as data changes |
| **Extension Point** | A defined interface where plugins can hook into the SIP platform |
| **Sidecar** | A container that runs alongside the main application, providing supplementary capabilities |
| **SSE** | Server-Sent Events — a standard for streaming data from server to client over HTTP |
| **HNSW** | Hierarchical Navigable Small World — an algorithm for efficient approximate nearest neighbor search in vector databases |

### Appendix C: Reference AssetTypes (shipped with MVP)

| AssetType | Category | Key Custom Attributes |
|-----------|----------|----------------------|
| Pump | INDUSTRIAL | flow_rate_gpm, head_pressure_psi, motor_hp, impeller_type, fluid_type |
| Motor | ELECTRICAL | voltage, rpm, frame_size, bearing_type_front, bearing_type_rear, insulation_class |
| Conveyor | MATERIAL_HANDLING | belt_width_in, belt_length_ft, speed_fpm, load_capacity_lbs, belt_material |
| HVAC Unit | HVAC | cooling_capacity_ton, refrigerant_type, filter_size, eer, seer_rating |
| Vehicle | FLEET | vin, make, model, year, odometer_reading, fuel_type, cargo_capacity |
| Generic Equipment | OTHER | unbounded custom fields via JSON Schema |

### Appendix D: AI Eval Framework Specification

Before shipping the AI Knowledge Agent, implement an automated evaluation framework:

```
sip/
└── tests/
    └── ai_eval/
        ├── eval_set.json        # 100 grounded Q&A pairs
        ├── run_eval.py          # Automated eval runner
        └── metrics.py           # Accuracy, faithfulness, relevance scoring

eval_set.json example:
[
  {
    "question": "What is the maintenance history of Asset X?",
    "ground_truth_facts": [
      "Asset X had bearing replacement on 2026-03-15",
      "Asset X had oil change on 2026-01-10",
      "Asset X is currently OPERATIONAL"
    ],
    "context_entities": ["asset:X", "work_order:101", "work_order:89"],
    "min_facts_required": 3
  }
]
```

**Eval metrics:**
1. **Factual accuracy:** % of ground truth facts present in the response.
2. **Faithfulness:** % of claims in the response that are supported by the provided context (hallucination check).
3. **Relevance:** Binary — does the response address the question?
4. **Citation rate:** % of factual claims with source citations.

**Eval runs automatically on:**
- Every PR that changes AI service code
- Before every release
- Weekly on the production dataset snapshot

### Appendix E: ID Key Format

| Entity | ID Format | Example |
|--------|-----------|---------|
| Asset | UUIDv7 (time-ordered) | `0192d3a5-f1e8-7b4c-a3d2-c1f5e8a9b0c1` |
| WorkOrder | UUIDv7 + display number | UUID: `0192d3a5...`, Display: `WO-2026-00452` |
| All other entities | UUIDv7 | |
| Organization | UUIDv7 | |
| Plugin instance | UUIDv7 | |

Display numbers (WO-2026-00452) are human-friendly identifiers for work orders. UUIDs are used for all API operations and foreign keys. Display numbers are generated sequentially per organization per year.

### Appendix F: API Error Code Registry

All API error responses follow this format:
```json
{
  "error": {
    "code": "INVALID_STATE_TRANSITION",
    "message": "Work order cannot transition from CLOSED to IN_PROGRESS.",
    "details": {
      "from_status": "CLOSED",
      "to_status": "IN_PROGRESS",
      "allowed_transitions": ["OPEN"]
    },
    "request_id": "req_abc123"
  }
}
```

**Standard error codes:**

| Code | HTTP Status | Meaning |
|------|------------|---------|
| VALIDATION_ERROR | 422 | Request body failed schema validation |
| AUTHENTICATION_REQUIRED | 401 | Missing or invalid credentials |
| PERMISSION_DENIED | 403 | Insufficient RBAC permissions |
| TENANT_SCOPE_VIOLATION | 403 | Cross-tenant access attempted |
| NOT_FOUND | 404 | Entity does not exist or is not accessible |
| VERSION_CONFLICT | 409 | Optimistic lock failure — entity was modified |
| INVALID_STATE_TRANSITION | 422 | Requested status transition is not allowed |
| IDEMPOTENCY_CONFLICT | 409 | Idempotency key reused with different request body |
| RATE_LIMITED | 429 | Too many requests |
| AI_PROVIDER_UNAVAILABLE | 502 | LLM provider is unreachable |
| AI_CONTEXT_UNAVAILABLE | 503 | AI service is unavailable |
| DOCUMENT_PROCESSING_FAILED | 422 | Document could not be processed |
| PLUGIN_PERMISSION_DENIED | 403 | Plugin exceeds declared permissions |
| PLUGIN_VERSION_INCOMPATIBLE | 400 | Plugin requires different SIP core version |
| INTERNAL_ERROR | 500 | Unexpected server error |

### Appendix G: Concurrency and Optimistic Locking

All mutable entities include a `version` column (integer, default 1). PATCH endpoints must use optimistic locking:

1. Client reads the entity, capturing its `version`.
2. Client sends PATCH with `If-Match: <version>` header or `version` field in the body.
3. Server increments version on successful write.
4. If the server's version does not match, return `409 VERSION_CONFLICT` with the latest record.

**Entities using optimistic locking:**

| Entity | Version Column |
|--------|---------------|
| Asset | version |
| WorkOrder | version |
| Schedule | version |
| Part | version |
| Document | version |
| Automation | version |
| Plugin | version |

This is critical for dispatch, assignment, and field work scenarios where concurrent modifications are likely.

### Appendix H: Asset Hierarchy with ltree

Asset and Location hierarchies use PostgreSQL `ltree` extension for materialized paths:

```sql
ALTER TABLE assets ADD COLUMN path ltree;
ALTER TABLE locations ADD COLUMN path ltree;

CREATE INDEX idx_assets_path ON assets USING GIST (path);
CREATE INDEX idx_locations_path ON locations USING GIST (path);
```

**Update behavior:**
- Each Asset has a `path` column derived from its parent hierarchy.
- Root assets use their own ID slug as the root path segment.
- When an asset's `parent_id` changes, the system must update `path` for the asset and all descendants in a single transaction.
- Cycles are forbidden. The system must validate that an asset cannot be made a child of itself or any descendant before committing the change.
- Location hierarchy follows the same pattern.

### Appendix I: Search Requirements

**MVP search tiers:**

| Tier | Capability | Phase |
|------|-----------|-------|
| Tier 1: Structured filters | Filter by status, priority, asset_type, location, assigned user/team, due date, created date | Phase 1 |
| Tier 2: Keyword search | Text search on asset name, serial number, work order title/description, resolution notes, document name (per-resource) | Phase 1 |
| Tier 3: Global semantic search | Cross-entity semantic search via embeddings | Phase 1.5 |

MVP supports per-resource search and filtering. Global `/api/v1/search` is deferred to Phase 1.5.

### Appendix J: Frontend MVP Screens

The following screens must ship in Phase 1:

| # | Screen | Priority |
|---|--------|----------|
| 1 | Login | Must |
| 2 | Organization setup | Must |
| 3 | Dashboard home | Must |
| 4 | Asset list (with search, filter, pagination) | Must |
| 5 | Asset detail (with work order history, document links) | Must |
| 6 | Create/edit asset | Must |
| 7 | AssetType list | Must |
| 8 | Create/edit AssetType | Must |
| 9 | Work order list (with search, filter, pagination) | Must |
| 10 | Work order detail (with AI context panel, document links) | Must |
| 11 | Create/edit work order | Must |
| 12 | Technician work queue (mobile-responsive) | Must |
| 13 | Document upload modal/page | Must |
| 14 | AI chat panel (contextual on asset/work order pages) | Must |
| 15 | Activity log viewer | Should |
| 16 | User/team management | Should |

**MVP UI principles:**
- Mobile-responsive but not full PWA in Phase 1.
- Technician work queue must be usable on a phone browser.
- AI chat appears contextually on asset and work order detail pages.
- Tables support search, filter, sort, and pagination.
- Status badges include text labels, not just color indicators (accessibility).

### Appendix K: Import and Export

**MVP export:**
```
GET /api/v1/export
```
Exports all tenant data as a JSON archive with separate files per entity type. Content: Organization settings, Locations, AssetTypes, Manufacturers, AssetModels, Assets, WorkOrders, WorkOrderAssignments, Schedules, Parts, Documents metadata, Activity logs. Documents exported as files plus metadata manifest.

**Phase 1.5 import:**
CSV import plugin for assets and work orders. JSON archive import/restore deferred to Phase 2.

**Constraint:** Tenant data export must never include another tenant's system-private records. PUBLIC and SYSTEM_DEFAULT records may be referenced by ID and version rather than duplicated.

### Appendix L: Data Retention and Purging Policy

- Activity records are retained indefinitely by default.
- Work orders are retained indefinitely unless tenant `document_retention_days` is configured.
- Documents respect `document_retention_days` from OrganizationSettings.
- AI conversations follow `ai_message_retention_days` from OrganizationSettings.
- Archived records remain queryable by admins and auditors.
- Hard deletion is limited to tenant deletion workflows and legal/compliance requirements.
- Activity records are never hard-deleted through the API.

### Appendix M: Licensing and Commercial Boundaries

- SIP core is AGPLv3.
- Official hosted service is provided by SIP maintainers as a commercial offering.
- Enterprise support, managed backups, uptime SLAs, SSO integrations, and deployment assistance may be commercial services.
- Plugins may have independent licenses, but plugins distributed with SIP core must be AGPL-compatible.
- The plugin marketplace must display plugin licenses clearly.
- The repository must include a CONTRIBUTING.md with contributor license expectations.
- Seed data uses fictional reference data. Real manufacturer names, models, and catalogs should be shipped separately or as community-maintained data.

### Appendix N: Implementation Guidance for Phase 1

#### Implementation Priority Cut Line

**Must build first (in order):**
1. Database schema and migrations
2. RLS and tenant isolation tests
3. Auth and RBAC
4. AssetType, Manufacturer, AssetModel
5. Location and Asset CRUD (with ltree hierarchy)
6. WorkOrder CRUD and state machine
7. WorkOrderAssignment
8. Activity audit log
9. Document metadata and upload (with DocumentLink)
10. Basic AI Q&A over work orders and assets (Ollama default)

**Build only after the above works:**
1. Schedules (cron-based PM generation)
2. Inspections (minimal — checklist items, no auto-work-order generation)
3. Parts CRUD + manual consumption
4. Document text extraction and embedding
5. AI citations from documents
6. Export

**Defer unless explicitly requested:**
1. Plugins (Phase 1.5)
2. Automations (Phase 1.5)
3. Notifications (Phase 2)
4. Advanced dashboards (Phase 2)
5. Meter-based scheduling (Phase 2)
6. Dispatch (Phase 2)
7. Dedicated graph database (Phase 4)
8. Native mobile (Phase 4)

#### Do Not Build These in MVP Unless Explicitly Instructed

- Plugin marketplace or sidecar execution
- NATS JetStream
- Neo4j
- Predictive maintenance ML
- Smart scheduling optimization
- Native mobile apps
- Full offline sync
- Multi-organization vendor portal
- Billing
- Enterprise SSO
- Compliance packages
- Multi-modal image analysis
- Autonomous AI mutations (without human confirmation)
- Global semantic search
- Full SLA policy engine

#### Definition of Done for MVP

MVP is complete when all of the following are true:

- [ ] `docker compose up` works from a clean checkout.
- [ ] Seeded demo organization is accessible on startup.
- [ ] User can log into the demo organization.
- [ ] User can create an AssetType with a JSON Schema.
- [ ] User can create a Manufacturer and AssetModel.
- [ ] User can create an Asset from an AssetType or AssetModel.
- [ ] User can create a WorkOrder for an Asset.
- [ ] User can assign the WorkOrder to another user via WorkOrderAssignment.
- [ ] Technician can accept, start, and complete the WorkOrder.
- [ ] Completion requires non-empty resolution notes.
- [ ] Asset detail page shows work order history.
- [ ] Activity records exist for all major changes.
- [ ] Tenant isolation tests pass (API and AI retrieval paths).
- [ ] Document can be uploaded and linked to an Asset or WorkOrder.
- [ ] Document processing status is visible in the UI.
- [ ] AI can answer "What is the maintenance history of this asset?" using cited WorkOrder records.
- [ ] AI refuses or says "I don't have enough information" when records are unavailable.
- [ ] Docker seed data includes enough completed WorkOrders to demo AI Q&A.
- [ ] README explains setup, login credentials, and demo workflow.

### Appendix O: Updated Permissions

Additional permissions added in v4.0:

| Permission | ADMIN | MANAGER | TECHNICIAN | AUDITOR |
|-----------|-------|---------|------------|---------|
| `work_order:reopen` | ✓ | ✓ | | |
| `work_order:archive` | ✓ | ✓ | | |
| `asset:archive` | ✓ | ✓ | | |
| `document:archive` | ✓ | ✓ | | |
| `export:tenant_data` | ✓ | | | |
| `ai:manage_settings` | ✓ | | | |
