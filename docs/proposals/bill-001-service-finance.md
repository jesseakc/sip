# BILL-001: Work-order financial ledger and charge calculation

Priority: P1
Status: implementation proposal for PR Worker

## Scope
Introduce tenant-scoped labor/parts/expense/travel charge records, currency, rate snapshots, adjustments and coverage allocations. Integrate EXP-001–006 PR #59 without duplicating expense ownership. Preserve internal costs separately from customer billable amounts. Acceptance: reconciled totals, idempotent imports, RBAC and tests.

## Architecture
SIPcore owns assets, work orders, tenant identity and plugin capability contracts. SIP Contracts owns agreements and entitlements. SIP Billing owns financial ledger and invoices. Plugins exchange typed, versioned APIs/events; they remain independently installable. Shared coverage evaluation is reusable, deterministic and auditable.

## Dependencies and safeguards
Coordinate with PR #27 (plugin capability contracts), PR #59 (expenses), PR #37 (SIP subscription billing is separate), and observability/cloud deployment priorities. Tenant isolation, authorization, structured telemetry, migrations, backwards compatibility and automated tests required. Human approval required before merging.

## Delivery
PR Worker should decompose into bounded implementation tasks and verify integrated acceptance criteria before marking a slice complete.
