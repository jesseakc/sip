# CON-002: Deterministic contract coverage decision engine

Priority: P1
Status: implementation proposal for PR Worker

## Scope
Evaluate work-order line items for labor, parts, travel, after-hours and deductibles against versioned agreement rules and warranties. Return covered/partial/excluded/unknown with clause references, effective-date snapshot and human override audit. Never let LLM alone determine charges. Acceptance: deterministic tests for overlaps, exclusions, expiry, limits, overrides and no-contract cases.

## Architecture
SIPcore owns assets, work orders, tenant identity and plugin capability contracts. SIP Contracts owns agreements and entitlements. SIP Billing owns financial ledger and invoices. Plugins exchange typed, versioned APIs/events; they remain independently installable. Shared coverage evaluation is reusable, deterministic and auditable.

## Dependencies and safeguards
Coordinate with PR #27 (plugin capability contracts), PR #59 (expenses), PR #37 (SIP subscription billing is separate), and observability/cloud deployment priorities. Tenant isolation, authorization, structured telemetry, migrations, backwards compatibility and automated tests required. Human approval required before merging.

## Delivery
PR Worker should decompose into bounded implementation tasks and verify integrated acceptance criteria before marking a slice complete.
