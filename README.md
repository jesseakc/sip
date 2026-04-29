# SIP (Service Intelligence Platform)

An AI-first service intelligence platform for physical assets. Open source (AGPLv3).

## Project Structure

```
sip/
├── Cargo.toml                 # Rust workspace root
├── crates/                    # Rust modular monolith crates
│   ├── sip-api/               # Axum HTTP API server (binary)
│   ├── sip-domain/            # Core domain entities and state machines
│   ├── sip-application/       # Application services and use cases
│   ├── sip-infrastructure/    # DB, Redis, MinIO, external clients
│   ├── sip-auth/              # JWT, password hashing, RBAC
│   ├── sip-tenancy/           # Tenant context and RLS session handling
│   ├── sip-assets/            # Asset, AssetType, Manufacturer, AssetModel
│   ├── sip-work-orders/       # WorkOrder, Assignment, Dispatch, Inspection
│   ├── sip-documents/         # Document metadata and processing pipeline
│   ├── sip-activity/          # Activity audit log
│   ├── sip-outbox/            # Transactional outbox and poller
│   ├── sip-ai/                # AI orchestration and retrieval
│   ├── sip-plugins/           # Plugin registry and manifests
│   ├── sip-scheduler/         # Cron scheduling and PM generation
│   ├── sip-parts/             # Parts and part usage
│   ├── sip-search/            # Structured and keyword search
│   ├── sip-observability/     # Tracing, metrics, health checks
│   ├── sip-config/            # Configuration loading
│   └── sip-cli/               # Admin/dev CLI
├── frontend/                  # Next.js 15 frontend
├── migrations/                # PostgreSQL schema + seed data
├── docs/                      # Documentation and ADRs
├── docker-compose.yml         # Local deployment stack
└── .env.example               # Environment variable template
```

## Quick Start (Docker Compose)

```bash
# 1. Copy environment template
cp .env.example .env

# 2. Start the full stack
docker compose up --build

# 3. Access services
# - Frontend: http://localhost:3000
# - API:      http://localhost:8000
# - MinIO Console: http://localhost:9001
# - Ollama:   http://localhost:11434
```

## Demo Login

Use the seeded demo account:
- Email: `admin@acme.local`
- Password: `password`

## Development

### Rust Backend

Requires: Rust stable (1.80+), PostgreSQL 16+ with pgvector, Redis, MinIO.

```bash
# Install SQLx CLI for migrations
cargo install sqlx-cli --no-default-features --features native-tls,postgres

# Run migrations
psql postgres://sip:sip@localhost:5432/sip -f migrations/001_initial_schema.sql
psql postgres://sip:sip@localhost:5432/sip -f migrations/002_indexes.sql
psql postgres://sip:sip@localhost:5432/sip -f migrations/003_rls_policies.sql
psql postgres://sip:sip@localhost:5432/sip -f migrations/004_seed_data.sql

# Run checks
cargo check --workspace

# Run API server
cargo run -p sip-api
```

### Next.js Frontend

Requires: Node.js 22+, npm.

```bash
cd frontend
npm install
npm run dev
```

## MVP Features

- **Auth**: JWT login with Argon2id password hashing
- **Organizations & Tenancy**: Row-level security (RLS) with `organization_id`
- **Asset Registry**: CRUD for assets with status transitions (Operational → Degraded → Maintenance → Retired)
- **Work Orders**: Full lifecycle management (Draft → Open → Assigned → Accepted → In Progress → Completed → Reviewed → Closed)
- **AI Chat**: Basic Ollama integration for natural language queries
- **Document Upload**: MinIO-backed object storage (stubbed for MVP)
- **Health Checks**: `/api/v1/health` and `/api/v1/health/ready`

## Architecture

- **Backend**: Rust (Axum, Tokio, SQLx) with modular crate workspace
- **Frontend**: Next.js 15 (App Router, TypeScript, Tailwind CSS)
- **Database**: PostgreSQL 16 + pgvector + ltree
- **Vector Search**: pgvector HNSW indexes for AI retrieval
- **AI**: Ollama (local LLM) with structured JSON response format

## License

AGPLv3 — see LICENSE file.
