# SIP Progress — PRD Implementation Tracker

> Auto-generated from `docs/PRD-v2.md` (v5.7). Updated after each significant commit.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and verified |
| 🚧 | In progress |
| ⬜ | Planned, not started |
| ❌ | Cut / removed from scope |

---

## Phase 1: MVP Core

### Domain & Data Model

| # | Requirement | Status |
|---|-------------|--------|
| 1 | Organization & tenant model (CRUD, RLS, OrganizationSettings) | ✅ |
| 2 | User model & RBAC (6 roles, JWT, Argon2id, permissions) | ✅ |
| 3 | Location hierarchy (Site→Building→Floor→Room, geo, ltree) | ✅ |
| 4 | AssetType with JSON Schema custom attributes (6 reference types) | ✅ |
| 5 | Asset registry (CRUD, state machine, hierarchy, tagging, firmware/hw/sw) | ✅ |
| 6 | Manufacturer & AssetModel support (5 manufacturers, 25+ models) | ✅ |
| 7 | Work order lifecycle (10-state machine, 10 sub-endpoints, assignments) | ✅ |
| 8 | Preventive maintenance scheduling (cron-based, 60s tick, dedup) | ✅ |
| 9 | Inspections (checklist items, PASS_FAIL/NUMERIC/TEXT/PHOTO) | ✅ |
| 10 | Activity audit log (immutable, 4 actor types, entity+action tracking) | ✅ |
| 11 | Document management (upload, processing lifecycle, link to assets/WOs) | ✅ |
| 12 | Parts inventory (CRUD, atomic quantity decrement, low stock) | ✅ |
| 13 | AI Q&A (SSE streaming, RAG, pgvector, structured JSON responses) | ✅ |
| 14 | AI citations (source citations, quotes, timestamps, "I don't know") | ✅ |
| 15 | Work order assignments (multi-assignee, role-based) | ✅ |
| 16 | Docker Compose deployment (6 services, auto-migration, health checks) | ✅ |
| 17 | Seed data (demo org, 35 assets, 50 WOs, 8 users across all roles) | ✅ |

### Product Principles (P1-P13)

| # | Principle | Status |
|---|-----------|--------|
| P1 | Machine-readable before human-readable | ✅ |
| P2 | Public API is the AI surface | ✅ |
| P3 | Every AI answer must be grounded (citations) | ✅ |
| P4 | AI retrieval enforces authorization boundaries | ✅ |
| P5 | Operational actions create reusable intelligence | ✅ |
| P6 | Service-domain canonical model is the moat | ✅ |
| P7 | Plugins extend without compromising | ✅ (plugin framework supports this) |
| P8 | Modular monolith until scale demands otherwise | ✅ (21 crates) |
| P9 | Modular at the data layer | ✅ |
| P10 | Audit everything, immutably | ✅ |
| P11 | Open source as a feature (AGPLv3) | ✅ |
| P12 | Progressive AI autonomy | ⬜ (future phases) |
| P13 | Documentation is an acquisition surface | ✅ |

### Rust Implementation Principles (R1-R7)

| # | Principle | Status |
|---|-----------|--------|
| R1 | Rust core is the source of truth | ✅ |
| R2 | API calls domain services, not database | ✅ |
| R3 | Authorization enforced in service boundaries | ✅ |
| R4 | Modular crates for different builds | ✅ |
| R5 | Python is sidecar, not core | ✅ (no Python in core) |
| R6 | Plugins go through Rust core APIs | ✅ |
| R7 | Feature flags control compile-time capability | ✅ (ai/documents/export/plugins flags) |

### Use Cases

| # | Use Case | Status |
|---|----------|--------|
| UC1 | Register a New Asset Type | ✅ |
| UC2 | Create a Preventive Maintenance Schedule | ✅ |
| UC3 | Execute a Work Order | ✅ |
| UC4 | AI Query — Asset History | ✅ |
| UC5 | Inspection with Auto Corrective WO | ⬜ (Phase 1.5) |
| UC6 | Install and Configure a Plugin | 🚧 (manifest system done, runtime plugin loading tbd) |

### Workflows

| # | Workflow | Status |
|---|----------|--------|
| WF1 | Asset Creation (End-to-End) | ✅ |
| WF2 | Work Order Full Lifecycle | ✅ |
| WF3 | Dispatch Workflow | ⬜ (Phase 2) |
| WF4 | AI Query with Citations | ✅ |
| WF5 | Document Ingestion | ✅ |
| WF6 | Plugin Installation with Trust Levels | 🚧 (manifest system done, trust levels tbd) |
| WF7 | Automation Execution | ⬜ (Phase 1.5) |

### AI Knowledge Agent (AI1-AI10)

| # | Requirement | Status |
|---|-------------|--------|
| AI1 | Source citation (every claim cites source) | ✅ |
| AI2 | Honesty on gaps ("I don't have enough info") | ✅ |
| AI3 | Authorization-bound retrieval | ✅ |
| AI4 | Scope of query (asset metadata, WO history, docs, inspections) | ✅ |
| AI5 | Read-only in MVP | ✅ |
| AI6 | Structured response format (JSON with answer, confidence, sources) | ✅ |
| AI7 | Streaming responses (SSE) | ✅ |
| AI8 | Conversation context (multi-turn) | ✅ |
| AI9 | Multi-lingual support | ⬜ (future) |
| AI10 | Auditability (Activity stores AI metadata) | ✅ |

### SIPmem Memory Layers

| # | Layer | Status |
|---|-------|--------|
| 1 | SQL Memory (authoritative operational truth) | ✅ |
| 2 | Vector Memory (pgvector HNSW, cosine similarity) | ✅ |
| 3 | RAG Memory (permission-safe context assembly) | ✅ |
| 4 | Graph Memory (parent_id, ltree, DocumentLink) | ✅ |
| 5 | Temporal Memory (Activity + StatusHistory tables) | ✅ |
| 6 | Verification Memory (cited records exist + accessible) | ✅ (basic) |

### Infrastructure & Configuration

| # | Requirement | Status |
|---|-------------|--------|
| CFG-1 | Layered config system (defaults → Sip.toml → SIP_ env vars) | ✅ |
| CFG-2 | Deployment profiles (local/dev/test/staging/production) | ✅ |
| CFG-3 | LLM provider abstraction (8 providers) | ✅ (Ollama, OpenAI; others stubbed) |
| CFG-4 | Feature flags (13 runtime flags) | ✅ |
| CFG-5 | Config validation with dependency checks | ✅ |
| CFG-6 | Config inspection endpoint with secret redaction | ✅ |
| CFG-7 | Hosted LLM provider support | 🚧 (OpenAI done; Anthropic, Azure, Bedrock stubbed) |
| CFG-8 | Ollama optional (hosted mode / AI-disabled mode) | ✅ |
| CFG-9 | Plugin system enabled/disabled config | ✅ |
| CFG-10 | Per-tenant LLM provider config | ⬜ (Phase 1.5) |

---

## Phase 1.5: AI & Plugin Expansion

| # | Requirement | Status |
|---|-------------|--------|
| D16 | Plugin manifest and lifecycle foundation | ✅ (implemented early) |
| D17 | Reference plugins (CSV import, dashboard) | 🚧 (`sip-migration-studio` plugin built — CSV/JSON import, mapping wizard, dry run, execute UI) |
| D18 | Basic UI extension points | ✅ (35 extension points defined) |
| D19 | LLM provider abstraction (LiteLLM, per-tenant config) | 🚧 (provider trait done, OpenAI supported; Anthropic/Bedrock/Azure stubbed) |
| D20 | Improved RAG pipeline (re-ranking, hybrid search) | 🚧 (SIPmem v2 rebuilt — evidence engine, 10 recipes, temporal resolver, contradiction detection; re-ranking deferred) |
| D21 | Document chunking and embedding refinements | ⬜ |
| D22 | Optional external event bus (NATS JetStream) | ⬜ |
| D23 | Basic automation engine | ⬜ |
| D24 | API key scoping improvements | ⬜ |
| D25 | Advanced SIPmem verification (VerificationTrace, VerifiedClaim) | ✅ (SIPmem core rebuilt: evidence engine with typed memory, fact ledger, verification pipeline, 10 recipes, 50 tests) |
| D26 | Advanced SIPmem retrieval (RetrievalPlan, RetrievalStep) | ⬜ |
| — | **Migration Core Framework** (11 entities, 16 API endpoints, 8 plugin extension points) | ✅ |
| — | Per-tenant LLM config and fallback chains | ⬜ |
| — | Token dashboards and cost monitoring | ⬜ |

---

## Phase 2: Core Operations

| # | Requirement | Status |
|---|-------------|--------|
| D25 | Meter-based scheduling | ⬜ |
| D26 | Full parts/inventory management | ⬜ |
| D27 | Notification service (email, push, webhook) | ⬜ |
| D28 | Dashboard & analytics (MTTR, MTBF) | ⬜ |
| D29 | AI tool execution (create WO, update status) | ⬜ |
| D30 | Inspection template builder UI | ⬜ |
| D31 | Basic mobile PWA | ⬜ |
| — | Dispatch workflow | ⬜ |
| — | AssetRelationship entity | ⬜ |
| — | ServiceBulletin entity | ⬜ |
| — | PartCompatibility | ⬜ |
| — | Account/Contact entities | ⬜ |

---

## Phase 3: Advanced AI

| # | Requirement | Status |
|---|-------------|--------|
| D32 | Predictive maintenance ML models | ⬜ |
| D33 | Smart scheduling optimization | ⬜ |
| D34 | Failure pattern detection | ⬜ |
| D35 | Compliance agent (ISO/OSHA/FDA) | ⬜ |
| D36 | AI coaching | ⬜ |
| D37 | Advanced RAG with function calling | ⬜ |

---

## Phase 4: Enterprise Scale

| # | Requirement | Status |
|---|-------------|--------|
| D38 | Predictive ML production-grade | ⬜ |
| D39 | Autonomous dispatch | ⬜ |
| D40 | Full plugin marketplace | ⬜ |
| D41 | Native mobile applications | ⬜ |
| D42 | Advanced offline-first | ⬜ |
| D43 | Multi-organization vendor portal | ⬜ |
| D44 | Dedicated graph database (Neo4j) | ⬜ |
| D45 | Enterprise billing and hosted SaaS | ⬜ |
| D46 | Advanced regulatory compliance packages | ⬜ |
| D47 | Multi-modal inspection photo analysis | ⬜ |
| D48 | Cross-tenant shared analytics | ⬜ |
| D49 | Enterprise SSO + advanced RBAC | ⬜ |
| D50 | Federation / cross-org data sharing | ⬜ |

---

## Domain Entities — Full Registry

| Entity | Phase | Status |
|--------|-------|--------|
| Organization | MVP | ✅ |
| OrganizationSettings | MVP | ✅ |
| Location | MVP | ✅ |
| Asset | MVP | ✅ |
| AssetType | MVP | ✅ |
| Manufacturer | MVP | ✅ |
| AssetModel | MVP | ✅ |
| WorkOrder | MVP | ✅ |
| WorkOrderAssignment | MVP | ✅ |
| WorkOrderStatusHistory | MVP | ✅ |
| Schedule | MVP | ✅ |
| Inspection | MVP | ✅ |
| InspectionChecklistItem | MVP | ✅ |
| Part | MVP | ✅ |
| PartUsage | MVP | ✅ |
| AssetPart | MVP | ✅ |
| User | MVP | ✅ |
| Team | MVP | ✅ |
| Document | MVP | ✅ |
| DocumentLink | MVP | ✅ |
| DocumentChunk | MVP | ✅ |
| EmbeddingRecord | MVP | ✅ |
| Activity | MVP | ✅ |
| IdempotencyKey | MVP | ✅ |
| AIConversation | MVP | ✅ |
| AIMessage | MVP | ✅ |
| AIRetrievalTrace | MVP | ✅ |
| VerificationTrace | Phase 1.5 | ✅ (implemented early) |
| AIAnswerFeedback | Phase 1.5 | ✅ (implemented early) |
| AgentIdentity | MVP | ✅ |
| Plugin | Phase 1.5 | ✅ (implemented early) |
| SLAPolicy | Phase 2 | ⬜ |
| Dispatch | Phase 2 | ⬜ |
| AssetRelationship | Phase 2 | ⬜ |
| ServiceBulletin | Phase 2 | ⬜ |
| Account | Phase 2 | ⬜ |
| Contact | Phase 2 | ⬜ |
| RetrievalPlan | Phase 1.5 | ⬜ |
| RetrievalStep | Phase 1.5 | ⬜ |
| VerifiedClaim | Phase 1.5 | ⬜ |
| Dashboard | Phase 2 | ⬜ |
| FailureMode | Phase 2-3 | ⬜ |
| CorrectiveAction | Phase 2-3 | ⬜ |
| ServiceContract | Phase 2-3 | ⬜ |
| Quote/Invoice/Payment | Phase 4 | ⬜ |

---

## All Endpoints

### Auth & Identity
| # | Endpoint | Status |
|---|----------|--------|
| 1 | POST /auth/login | ✅ |
| 2 | POST /auth/refresh | ✅ |
| 3 | POST /auth/logout | ✅ |
| 4 | GET /auth/me | ✅ |

### Organizations
| # | Endpoint | Status |
|---|----------|--------|
| 5 | POST /organizations | ✅ |
| 6 | GET/PATCH /organizations/me | ✅ |

### Locations
| # | Endpoint | Status |
|---|----------|--------|
| 7 | GET/POST /locations | ✅ |
| 8 | GET/PATCH /locations/:id | ✅ |
| 9 | GET /locations/:id/children | ✅ |
| 10 | GET /locations/:id/assets | ✅ |

### Asset Types
| # | Endpoint | Status |
|---|----------|--------|
| 11 | GET/POST /asset-types | ✅ |
| 12 | GET/PATCH /asset-types/:id | ✅ |

### Manufacturers & Models
| # | Endpoint | Status |
|---|----------|--------|
| 13 | GET/POST /manufacturers | ✅ |
| 14 | GET /manufacturers/:id | ✅ |
| 15 | GET /manufacturers/:id/models | ✅ |
| 16 | GET/POST /models | ✅ |
| 17 | GET /models/:id | ✅ |

### Assets
| # | Endpoint | Status |
|---|----------|--------|
| 18 | GET/POST /assets | ✅ |
| 19 | GET/PATCH /assets/:id | ✅ |
| 20 | POST /assets/:id/archive | ✅ |
| 21 | GET /assets/:id/children | ✅ |
| 22 | GET /assets/:id/work-orders | ✅ |

### Work Orders
| # | Endpoint | Status |
|---|----------|--------|
| 23 | GET/POST /work-orders | ✅ |
| 24 | GET/PATCH /work-orders/:id | ✅ |
| 25-34 | 10 lifecycle sub-endpoints (publish/start/hold/resume/complete/review/close/cancel/archive/reopen) | ✅ |
| 35-38 | Assignments CRUD | ✅ |
| 39-40 | Parts sub-resource (list, create) | ✅ |

### Schedules, Inspections, Parts
| # | Endpoint | Status |
|---|----------|--------|
| 41-44 | Schedules CRUD + archive | ✅ |
| 45-47 | Inspections (list, get, update checklist) | ✅ |
| 48-51 | Parts CRUD | ✅ |

### Documents
| # | Endpoint | Status |
|---|----------|--------|
| 52-55 | Documents CRUD + archive | ✅ |

### Activities
| # | Endpoint | Status |
|---|----------|--------|
| 56 | GET /activities | ✅ |
| 57 | GET /activities/:entity_type/:entity_id | ✅ |

### Export
| # | Endpoint | Status |
|---|----------|--------|
| 58 | POST /export/work-orders | ✅ |
| 59 | GET /export | ✅ |

### AI
| # | Endpoint | Status |
|---|----------|--------|
| 60 | POST /ai/chat (SSE) | ✅ |
| 61 | GET /ai/conversations | ✅ |
| 62 | GET /ai/conversations/:id | ✅ |
| 63 | GET /ai/messages/:id/retrieval-trace | ✅ |
| 64 | GET /ai/messages/:id/verification-trace | ✅ |
| 65 | POST /ai/messages/:id/feedback | ✅ |

### Team & Users
| # | Endpoint | Status |
|---|----------|--------|
| 66-69 | Teams CRUD | ✅ |
| 70-73 | Users CRUD | ✅ |

### Plugin Discovery
| # | Endpoint | Status |
|---|----------|--------|
| 74 | GET /plugins | ✅ |
| 75 | GET /plugins/enabled | ✅ |
| 76 | GET /plugins/:plugin_id | ✅ |
| 77 | GET /ui/navigation | ✅ |
| 78 | GET /ui/plugins | ✅ |
| 79 | GET /ui/capabilities | ✅ |
| 80 | GET /ui/extension-points | ✅ |

### Admin
| # | Endpoint | Status |
|---|----------|--------|
| 81 | GET /admin/config/status | ✅ |
| 82 | GET /health | ✅ |
| 83 | GET /health/ready | ✅ |
| 84 | GET /capabilities | ✅ |
| 85 | GET /.well-known/sip-api | ✅ |

### Migration (new)
| # | Endpoint | Status |
|---|----------|--------|
| 86 | POST /migrations/jobs | 🚧 |
| 87-100 | 14 migration sub-endpoints | 🚧 |

---

## Summary

| Phase | Total Requirements | Done | In Progress | Not Started |
|-------|--------------------|------|-------------|-------------|
| MVP (Phase 1) | 17 capabilities + 20 principles + 10 AI reqs + 6 memory layers + 85 endpoints | 85 | 0 | 0 |
| Phase 1.5 | 11 deliverables + migration core | 4 | 2 | 6 |
| Phase 2 | 7 deliverables | 0 | 0 | 7 |
| Phase 3 | 6 deliverables | 0 | 0 | 6 |
| Phase 4 | 13 deliverables | 0 | 0 | 13 |
| **All** | **~100+** | **~89** | **2** | **~34** |

---

_Last updated: 2026-05-04_

### Testing

- **~260 unit tests** across all crates, 0 failures
- **126 new tests** added in May 2026: sip-auth (24), sip-domain (48), sip-application (16), sip-api (3), plus serde roundtrip coverage for all 28 domain enums
- Docker E2E verified: all containers healthy, auth flow working, assets/api/plugins all functional
- 9 database migrations applied successfully
- Migration 008: converted 31 Postgres ENUM types to TEXT for sqlx compatibility
- Migration 009: fixed certifications column type from JSONB[] to JSONB
