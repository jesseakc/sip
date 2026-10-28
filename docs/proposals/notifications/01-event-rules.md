# SIP Notifications: Event and Rules Foundation

Status: Proposal, not implementation.

## Goal
Provide an optional standalone `sip-notifications` plugin for tenant-safe service event communications. SIP Core publishes versioned events and exposes authorized read APIs; all notification logic remains in the plugin.

## Event contract
Required: event_id, tenant_id, event_type, asset_id, occurred_at, actor_id, schema_version. Optional: service_case_id, old_status, new_status, severity, source_revision, correlation_id. Publish via transactional outbox or equivalent durable mechanism. Preserve ordering per asset where possible, handle replay and idempotency explicitly.

## Rule engine
Tenant-scoped versioned rules with triggers, asset/location filters, severity, role-based audiences, business-hours windows, delays, cooldowns, escalation thresholds, and priority. Deterministic evaluation without AI. Dry-run simulator, rule conflict checks, safe defaults, enable/disable and rollback. Rules must not allow bypassing tenant authorization.

## Stakeholder resolution
Resolve users, supervisors, equipment owners, support groups and authorized external contacts using SIP relationships and explicit recipient permissions. Verify current recipient eligibility at dispatch, honor communication preferences and suppression policies, deduplicate contacts. Support missing/ambiguous recipients via exception queue, never guess addresses.

## Deliverables
Plugin scaffold and manifest; event subscriber; event/rule/recipient interfaces; storage migrations owned by plugin; CLI/API configuration; installer wizard integration; audit events; tests and documentation. No hardcoded AI model, provider or email credentials.

## Acceptance tests
Down event creates correct jobs; replay does not duplicate; tenant boundaries hold; rule version recorded; invalid/missing recipients cannot send; disabling plugin leaves SIP Core functional; event publication survives crash/restart.
