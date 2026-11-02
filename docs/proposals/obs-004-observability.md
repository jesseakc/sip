# OBS-004: Telemetry collector and retention adapters

> **PR WORKER PRIORITY: P0 / OBSERVABILITY CRITICAL PATH.** Implement this collection before unrelated queued SIP PR work, subject to safety gates and dependency order. This is a proposal specification, not implementation.

## Objective
Collector pipeline with validation, redaction, bounded buffering, backpressure, offline operation, OTLP export, pluggable local and remote backends, tenant isolation, retention and sampling.

## Acceptance criteria
Graceful failure under unavailable backend; queue limits and dropped-event metrics; retention and isolation tests.

## Shared engineering constraints
- OpenTelemetry-compatible; versioned structured JSON; UTC timestamps and distributed trace context.
- Self-hosted/offline-first and vendor-neutral; optional hosted backends.
- Redact sensitive data at source and collector; no secrets in logs.
- Enforce tenant isolation, least privilege, and security audit separation.
- Add automated tests, documentation and migration notes; avoid breaking existing plugins.
- Do not merge to main automatically. Repogan independent review and human approval required.

## Execution
PR Worker: replace proposal-only change with implementation on this branch, run tests and CI, then request Repogan review. Prioritize OBS-001, OBS-002, OBS-003, OBS-004, OBS-005 before remaining OBS PRs. Parallelize independent tasks when safe.
