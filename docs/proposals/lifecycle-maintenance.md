# Consumables and lifecycle-based preventive maintenance engine

## Goal
Track replacement schedules for batteries, filters, single-use disposables, shelf-life-limited materials, cycle-limited parts and condition-monitored components.

## Requirements
- Versioned, approved LifecyclePolicy linked to product definitions, applicable BOM revisions and optional per-instance overrides.
- Support calendar intervals, manufacture/installation/activation/first-use start events, fixed expiry, operating hours, cycles, uses, condition thresholds and earliest-of hybrid rules.
- Model lot/expiry and quantity-based consumption separately from serialized replaceable components.
- Deterministic Rust evaluation engine, timezone-aware due dates, stale/missing meter reading status, warning/escalation windows and periodic evaluation jobs.
- Engineering-source provenance, approval workflow and effective dates; AI-discovered intervals are proposed only until approved.
- On replacement, close old lifecycle, preserve history, start new lifecycle using correct policy/start event, and link work order and verification.
- Preventive maintenance proposals, reminders, notifications, stock reservation suggestions and due/overdue UI.
- Safety: overdue and missing data must not be silently classified as safe; never auto-certify return to service.

## Acceptance criteria
- Five-year battery interval triggers correctly from configured start event with 180/30-day warnings.
- Hours/cycles/expiry/hybrid rules evaluate correctly, including stale readings and boundary conditions.
- Replacement resets only the new component lifecycle and preserves previous historical requirements.
- Policy revisions do not rewrite past compliance state.
- Tenant isolation, audit logging and idempotent maintenance job generation verified by tests.

## Dependencies
Inventory foundation and engineering component genealogy.

## Delivery
Implement Rust engine, persistence, UI/API, jobs and automated tests before completion.