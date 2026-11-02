# OBS-008: SIPmem evidence-grounded AI investigations

> **PR WORKER PRIORITY: P0 / OBSERVABILITY CRITICAL PATH.** Implement this collection before unrelated queued SIP PR work, subject to safety gates and dependency order. This is a proposal specification, not implementation.

## Objective
Read-only diagnostic investigation tools, retrieval over event IDs and traces, incident similarity, root-cause hypotheses with citations, prompt-injection defenses, model agnostic configuration.

## Acceptance criteria
Every conclusion links to evidence; untrusted logs cannot issue commands; tenant isolation and no default model.

## Shared engineering constraints
- OpenTelemetry-compatible; versioned structured JSON; UTC timestamps and distributed trace context.
- Self-hosted/offline-first and vendor-neutral; optional hosted backends.
- Redact sensitive data at source and collector; no secrets in logs.
- Enforce tenant isolation, least privilege, and security audit separation.
- Add automated tests, documentation and migration notes; avoid breaking existing plugins.
- Do not merge to main automatically. Repogan independent review and human approval required.

## Execution
PR Worker: replace proposal-only change with implementation on this branch, run tests and CI, then request Repogan review. Prioritize OBS-001, OBS-002, OBS-003, OBS-004, OBS-005 before remaining OBS PRs. Parallelize independent tasks when safe.
