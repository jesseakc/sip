# SIP Expenses and Service Cost Intelligence

Status: proposal for implementation, not a shipped feature.
Owner: SIP plugin ecosystem. Target: optional standalone expense plugin with minimal, reusable SIPcore cost attribution primitives.

## Problem
Service, repair, installation and preventive maintenance costs are fragmented across work orders, receipts, travel booking tools, inventory and accounting. SIP must measure actual lifecycle cost without forcing customers to replace their finance platform.

## Principles
- Optional plugin. SIPcore and unrelated installations function without it.
- Separate **expense capture** from **cost attribution**. SIPcore provides generic object references and allocation contracts, not expense UI or finance-vendor logic.
- Multi-tenant, permission-safe and auditable. Financial details are sensitive and must not be exposed to unauthorized technicians or AI retrieval.
- No default AI model. Receipt extraction is opt-in and uses configured tenant model credentials or deterministic/manual capture.
- Source amounts are exact decimal minor units or decimal NUMERIC, never floating point; ISO currency codes and explicit exchange-rate provenance.
- A receipt is evidence, not proof of reimbursement or approval.

## Capabilities
1. Technician creates expense lines for airfare, lodging, meals, mileage, rental car, fuel, tolls, parking, per diem, parts and miscellaneous charges.
2. Attach receipt or invoice through secure document storage; manual correction of extracted merchant, date, amount, tax, currency and category.
3. Link expense lines to multiple SIP objects: work order, service visit, asset, customer, location, installation, project, contract, or future plugin-defined entity.
4. Split a cost by fixed amount or percentage. Enforce allocation sum <= expense amount and prevent double-counting in rollups; permit unapportioned balance.
5. Mileage: origin/destination or manual distance, unit, travel date, rate snapshot, jurisdiction and policy; do not require GPS tracking. Per diem policy snapshots.
6. Draft -> submitted -> approved/rejected -> exported/reconciled state machine, with return-for-correction and immutable audit events. Finance-only overrides with reason.
7. Distinguish employee-paid, company-card, vendor-paid; reimbursable vs nonreimbursable; customer-billable vs internal; estimated vs actual.
8. Tenant-configurable approval rules and retention; immutable receipt hashes, encryption and access-controlled download URLs.
9. Import/export and idempotent external references for SAP Concur and other providers; never silently overwrite locally approved records.
10. Dashboards: cost by asset, work order, customer, location, visit and period; travel versus parts versus labor; clearly label estimated downtime and avoid treating it as paid expense.

## Suggested schema (design, not committed migration)
- expense_reports: id, tenant_id, owner_id, status, submitted_at, approved_at, policy_version, timestamps.
- expense_lines: id, tenant_id, report_id nullable, occurred_at, category, merchant, amount NUMERIC, currency CHAR(3), payment_source, reimbursable, billable, source_system, external_id, timestamps.
- expense_allocations: id, tenant_id, expense_line_id, target_type, target_id, allocated_amount NUMERIC, currency, method, timestamps; index on tenant/target.
- expense_receipts: id, tenant_id, expense_line_id, document_id, sha256, extraction_status, provenance, timestamps.
- expense_audit_events: id, tenant_id, subject_type, subject_id, actor_id, action, before/after references, occurred_at.
- expense_connector_sync: tenant_id, provider, external_id, internal_id, cursor, status, last_synced_at, error_redacted.

Enforce tenant-bound object references through a registry/authorization resolver; a polymorphic target_type/target_id pair alone is not a safe foreign key. Ensure cross-tenant linking fails closed, deleted targets preserve historical allocation labels, and financial reporting uses authoritative source amounts with no double counting.

## API and permissions (proposed)
- POST/GET /api/v1/expenses; PATCH /api/v1/expenses/{id} (draft only).
- POST /api/v1/expense-reports; POST /api/v1/expense-reports/{id}/submit; POST /approve; POST /reject.
- POST /api/v1/expenses/{id}/allocations; GET /api/v1/costs?target_type=&target_id=.
- POST /api/v1/expenses/{id}/receipts (validated upload); POST /api/v1/expenses/{id}/extract (optional).
- Permissions: expense.self.read/write, expense.team.review, expense.finance.read/export, expense.connector.manage, cost.read. Require tenant scope on every operation and agent tool.

## Delivery slices
- EXP-001: Plugin manifest, domain, migrations, tenant isolation, CRUD, permissions and tests.
- EXP-002: Generic cost-target registry/authorization and allocation contract, split allocation and rollup tests.
- EXP-003: Receipt capture, secure document links, extraction adapters, human confirmation and duplicate warnings.
- EXP-004: Mileage, travel, per diem, reporting, approvals and audit events.
- EXP-005: Vendor-neutral connector contract and first SAP Concur adapter after confirming tenant API entitlements; idempotent import/export.
- EXP-006: Service Cost Intelligence reporting and source-grounded SIPmem read-only insights.

## Acceptance criteria
- Tenant A cannot read, upload, allocate or report on tenant B's expenses or targets.
- Expense allocated across three work orders totals exactly the expense amount and is counted once in aggregate.
- A single flight can be split across multiple visits without duplicating receipts.
- A technician can submit manually without AI or external integrations.
- Repeated connector sync produces no duplicate expense or audit event.
- Approval, rejection, correction and export transitions are authorized and audited.
- OCR/LLM suggestions never auto-approve, reimburse, or alter confirmed financial data.
- Receipt access is time-limited and audited; malformed files and oversized uploads are rejected.
- All currency conversions record source, rate and effective date.
- Plugin can be disabled without breaking SIPcore or losing existing data.

## Review boundaries
Do not merge this proposal as a finished implementation. Follow-up implementation PRs should be independently tested and reviewed; Repogan review and human approval are required before merging to main.
