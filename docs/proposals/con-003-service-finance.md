# CON-003: AI-assisted contract extraction and exception review

Priority: P2
Status: implementation proposal for PR Worker

## Scope
Use configured model providers to propose structured clauses and citations from versioned SIPmem contract documents. No default AI provider; installer requires explicit configuration when enabled. Human validates clauses before activation; unresolved ambiguities require review. Acceptance: provenance, prompt injection resistance, permissions and evaluation fixtures.

## Architecture
SIPcore owns assets, work orders, tenant identity and plugin capability contracts. SIP Contracts owns agreements and entitlements. SIP Billing owns financial ledger and invoices. Plugins exchange typed, versioned APIs/events; they remain independently installable. Shared coverage evaluation is reusable, deterministic and auditable.

## Dependencies and safeguards
Coordinate with PR #27 (plugin capability contracts), PR #59 (expenses), PR #37 (SIP subscription billing is separate), and observability/cloud deployment priorities. Tenant isolation, authorization, structured telemetry, migrations, backwards compatibility and automated tests required. Human approval required before merging.

## Delivery
PR Worker should decompose into bounded implementation tasks and verify integrated acceptance criteria before marking a slice complete.
