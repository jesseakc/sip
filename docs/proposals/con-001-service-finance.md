# CON-001: Service agreements and asset entitlement schema

Priority: P1
Status: implementation proposal for PR Worker

## Scope
Model agreements, parties, covered assets/sites/fleets, effective intervals, amendments, renewals, exclusions and rate schedules. Tenant-scoped records, immutable revisions, references to SIPmem documents. Acceptance: migrations, CRUD/API, permission checks, date-bound coverage lookup and tests.

## Architecture
SIPcore owns assets, work orders, tenant identity and plugin capability contracts. SIP Contracts owns agreements and entitlements. SIP Billing owns financial ledger and invoices. Plugins exchange typed, versioned APIs/events; they remain independently installable. Shared coverage evaluation is reusable, deterministic and auditable.

## Dependencies and safeguards
Coordinate with PR #27 (plugin capability contracts), PR #59 (expenses), PR #37 (SIP subscription billing is separate), and observability/cloud deployment priorities. Tenant isolation, authorization, structured telemetry, migrations, backwards compatibility and automated tests required. Human approval required before merging.

## Delivery
PR Worker should decompose into bounded implementation tasks and verify integrated acceptance criteria before marking a slice complete.
