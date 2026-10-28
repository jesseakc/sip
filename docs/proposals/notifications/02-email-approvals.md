# SIP Notifications: Email Delivery, Lifecycle and Approvals

Status: Proposal, not implementation. Depends on event/rules foundation.

## Scope
Standalone email adapter and durable notification workflow for equipment down/degraded/restored, service progress, and SLA escalation. Use SMTP initially with provider interface for future channels. Never embed secrets in events or logs.

## State machine
pending -> drafted -> awaiting_approval -> approved -> queued -> sending -> sent or failed. Include rejected, canceled, expired and dead_letter states. Persist all transitions with actor, timestamps, policy and message version. Delivery provider acceptance is not proof of inbox receipt; report accurately.

## Approval
Rule-level modes: always, critical-only, external-only, and policy-authorized automatic. Reviewers can edit, approve, reject, request revision. Any material message change after approval invalidates prior approval. Require role-based approval rights and enforce separation where configured. Critical internal alerts may use preapproved deterministic templates while external communications await review. Approval timeouts escalate rather than silently sending.

## Reliability and privacy
Transactional job creation, stable idempotency keys, bounded retries with backoff and jitter, dead-letter queue, rate limits, suppression windows, unsubscribe/preferences where applicable, delivery diagnostics, tenant isolation, encryption in transit, audit logs and retention configuration. No PHI or sensitive diagnostics in customer messages without authorization and configured controls.

## Acceptance tests
Simulate SMTP outage, repeated event, revoked recipient, approval expiration, edits after approval, reopened incident, restored notification, tenant-crossing attempt, and plugin shutdown/restart. Demonstrate no silent loss and accurate delivery states.
