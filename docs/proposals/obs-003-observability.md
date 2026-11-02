# OBS-003: Plugin telemetry SDK and conformance checks

> **PR WORKER PRIORITY: P0 / OBSERVABILITY CRITICAL PATH.** Implement this collection before unrelated queued SIP PR work, subject to safety gates and dependency order. This is a proposal specification, not implementation.

## Objective
Mandatory plugin telemetry interface for logs, metrics, traces, health, stable errors and audit hooks; SDK helpers and CI conformance suite.

## Acceptance criteria
Existing plugins migrate incrementally; third-party plugins can validate conformance; no direct storage dependency.

## Shared engineering constraints
- OpenTelemetry-compatible; versioned structured JSON; UTC timestamps and distributed trace context.
- Self-hosted/offline-first and vendor-neutral; optional hosted backends.
- Redact sensitive data at source and collector; no secrets in logs.
- Enforce tenant isolation, least privilege, and security audit separation.
- Add automated tests, documentation and migration notes; avoid breaking existing plugins.
- Do not merge to main automatically. Repogan independent review and human approval required.

## Execution
PR Worker: replace proposal-only change with implementation on this branch, run tests and CI, then request Repogan review. Prioritize OBS-001, OBS-002, OBS-003, OBS-004, OBS-005 before remaining OBS PRs. Parallelize independent tasks when safe.
