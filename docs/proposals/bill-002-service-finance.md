# BILL-002: Generate and attach approved work-order invoices

Priority: P1
Status: implementation proposal for PR Worker

## Scope
Invoice draft/approval/issue lifecycle, PDF rendering and secure attachments to work orders and customer accounts. Include covered line-item reductions, $0 invoices, numbering per tenant, tax configuration, credit/void flow, immutable issued snapshot, access controls and audit trail. Acceptance: PDF generation tests, tenant isolation, no double issue and signed attachment access.

## Architecture
SIPcore owns assets, work orders, tenant identity and plugin capability contracts. SIP Contracts owns agreements and entitlements. SIP Billing owns financial ledger and invoices. Plugins exchange typed, versioned APIs/events; they remain independently installable. Shared coverage evaluation is reusable, deterministic and auditable.

## Dependencies and safeguards
Coordinate with PR #27 (plugin capability contracts), PR #59 (expenses), PR #37 (SIP subscription billing is separate), and observability/cloud deployment priorities. Tenant isolation, authorization, structured telemetry, migrations, backwards compatibility and automated tests required. Human approval required before merging.

## Delivery
PR Worker should decompose into bounded implementation tasks and verify integrated acceptance criteria before marking a slice complete.
