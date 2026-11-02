# SIP dual-deployment architecture (proposal v0.1)

Status: proposed; architecture contract, not evidence of deployed capability.

## Goals
One Rust SIPcore, Next.js frontend, SIPmem, plugin interfaces and API across SIP-hosted SaaS, dedicated enterprise cloud and customer-controlled installations. Stage 1 is a Vercel proof of concept, Stage 2 is production SaaS and private enterprise. Avoid mandatory Kubernetes.

## Deployment profiles
| Profile | Web | SIPcore | Data | Operations |
| --- | --- | --- | --- | --- |
| POC | Vercel | Single persistent container | Managed PostgreSQL and object storage | SIP team |
| SaaS | Vercel or replicated web | Replicated managed containers | Shared tenant-isolated PostgreSQL initially, durable object storage and jobs | SIP team |
| Private cloud | Customer-selected frontend hosting | Dedicated containers | Dedicated database, storage and keys | SIP or customer contract |
| Self-hosted | Customer-controlled | Signed containers | Customer-managed compatible services | Customer, supported by SIP |

Profiles are packaging and infrastructure choices, not product forks. Keep SIPcore modular; split out jobs only when justified.

## Trust boundaries
- Tenant is primary security/data ownership boundary. Facilities and assets are subordinate scopes; cross-tenant service providers require explicit grants.
- Resolve principal and tenant from authenticated context, never an untrusted request header or queue payload.
- Enforce database RLS with a least-privileged runtime role, deny-by-default and tests for missing/wrong tenant context, connection-pool reuse and bypass attempts.
- Scope documents, object access, embeddings, vector search, graph edges, AI context, cache keys, audit records and worker jobs to authorized tenants and resources.
- Third-party plugins use capability-scoped identities. Untrusted plugins execute out of process. Agent tools follow the same authorization model and require approval for privileged actions.
- Separate diagnostic telemetry from append-only security audit evidence. Redact credentials, sensitive customer data and tokens.

## Deployment contract
- PostgreSQL connection, schema migration/version and health checks.
- Object storage adapter: S3-compatible first, alternative providers through adapters. No persistent application filesystem.
- Durable job interface with explicit tenant context, retries, idempotency and dead-letter handling. In-process cron must not duplicate work across replicas.
- OIDC identity with optional enterprise federation; external secrets injection.
- OpenTelemetry traces, metrics and structured logs with correlation IDs.
- AI provider and embedding configuration are explicitly selected by administrators; no implicit default model.
- Capability discovery and versioned plugin compatibility manifest.
- Reproducible container images and deployment templates; no provider-specific assumptions in domain code.

## Data lifecycle and portability
PostgreSQL owns transactional state; object storage owns files; SIPmem stores knowledge metadata and rebuildable indexes; telemetry is retained separately. Exports and restores must include tenant mapping, object references, schema/plugin versions, encryption-key strategy and verification. Migration between SaaS and private deployments requires an authorized, audited cutover.

## Operations
CI builds signed artifacts and runs integration, tenant isolation and upgrade tests. Stage 1 requires staging environment, health/readiness endpoints, secrets, backup/restore smoke tests and end-to-end asset -> work order -> document -> retrieval -> diagnostics workflow. Production profiles additionally require HA, disaster recovery targets, rolling deployments, rate limits, tenant usage metering and supportable rollback procedures.

## Design constraints
- Do not introduce Kubernetes until deployment scale and operational requirements justify it.
- Do not assume Redis, MinIO, outbox, document ingestion or SIPmem are wired simply because crates or Compose services exist.
- Reconcile claims against runtime behavior and tests, not only docs/PROGRESS.md.
- Preserve human approval for merging to main and production promotion.

See docs/deployment-gap-analysis.md in companion PR for execution sequencing and existing proposal branch mapping.
