# Enterprise deployment gap analysis and PR Worker sequencing

Status: proposal from repository docs and branch inventory, 2026-11-02. This is not a completed source-level audit or CI verification.

## Existing work: avoid duplicate proposals
- OBS branches: proposal/obs-001-observability through proposal/obs-010-observability. Reuse and verify these before adding new observability implementation.
- Deployment branches: proposal/deployment-observability, -database-isolation, -object-storage, -jobs-events-notifications, -backup-migration, -production-security, -identity-providers, -provider-configuration, -ai-providers, -reproducible-distribution, -deployment-test-matrix, -plugin-capability-contracts, -operator-tenant-brand, -operator-billing, -plugin-marketplace and others. Treat these as existing candidate slices, not proven merged functionality.
- Installer branch: proposal/sip-installer-marketplace-20261027; AI installer: proposal/sipmem-installer-ai-config.
- Existing priority coordination: issue #82, observability P0 and first cloud deployment P1.

## Verified documentation discrepancies requiring runtime tests
- docs/PROGRESS.md claims document upload and RAG complete; docs/architecture.md says document upload route absent and SIPmem core not wired to API.
- docs/architecture.md describes Redis and MinIO as provisioned but unused and outbox as present but unused.
- In-process scheduler ticks in sip-api; replicas risk duplicate scheduling without leader election or idempotent distributed execution.
- Existing PostgreSQL tenant role and RLS are documented; correctness under pooling, worker jobs and retrieval needs adversarial tests.
These are audit targets, not conclusions about current branch HEAD behavior.

## Ordered slices and acceptance criteria
| Order | Priority | Slice | Existing proposal mapping | Minimum acceptance |
| --- | --- | --- | --- | --- |
| 1 | P0 | OBS structured logging, tracing and diagnostics | obs-001..010; deployment-observability | Trace request across API, job and plugin; redaction tests; actionable viewer |
| 2 | P0 | Tenant isolation hardening | deployment-database-isolation; deployment-production-security | Negative cross-tenant tests for SQL, storage, retrieval, queue and pooled connections |
| 3 | P0 | Persistent SIPcore container and configuration | deployment-provider-configuration; deployment-reproducible-distribution | Health checks, secret validation, reproducible image, no local persistent files |
| 4 | P0 | Object storage runtime | deployment-object-storage | Authorized upload/download and versioning; no public cross-tenant object access |
| 5 | P0 | Managed DB migrations and backups | deployment-backup-migration | Fresh provision, upgrade and restore in isolated staging |
| 6 | P1 | Vercel frontend + hosted Rust API | deployment-deployment-test-matrix; reproducible-distribution | HTTPS auth, assets, WOs, document flow, diagnostics and rollback |
| 7 | P1 | Durable jobs/outbox | deployment-jobs-events-notifications | Retry, idempotency, DLQ, no duplicate scheduled jobs across replicas |
| 8 | P1 | SIPmem runtime and configured AI | sipmem-sipmem-runtime; sipmem-installer-ai-config; deployment-ai-providers | Tenant-authorized cited retrieval; provider failure handling; no default model |
| 9 | P1 | SaaS tenant lifecycle and operations | deployment-operator-tenant-brand; deployment-operator-billing | Tenant provisioning, quota/usage, admin audit, safe suspension |
| 10 | P1 | Dedicated private-cloud templates | deployment-identity-providers; deployment-production-security | Reproducible isolated deployment with enterprise identity and secrets |
| 11 | P2 | Installer, upgrades and self-hosting | sip-installer-marketplace; deployment-plugin-capability-contracts | Idempotent install, compatibility validation, backup-before-upgrade, signed artifacts |
| 12 | P2 | Cluster HA and disaster recovery | deployment-deployment-test-matrix; deployment-backup-migration | Multi-replica load, failover and measured RTO/RPO |

## Worker rules
- Check whether each existing proposal branch has an open PR and inspect its diff before starting an implementation slice.
- Prefer advancing existing proposal PRs rather than creating competing branches. Do not mark a slice complete for documentation alone.
- Implement bounded execution tasks with integration tests and require Repogan independent review before requesting human merge approval.
- Never merge to main or promote production without explicit human approval.
- Prioritize OBS first, then security and deployability, preserving issue #82's intent.
- Record exact PR/branch dependencies and blockers in GitHub rather than treating this matrix as an automatically executable queue.

## Suggested next verification
Inspect docker-compose.yml, Dockerfiles, frontend configuration, migrations, API startup, tenant middleware, storage adapters, document routes, scheduler and CI workflows at current main. Update the matrix with commit SHAs and concrete PR numbers once verified.
