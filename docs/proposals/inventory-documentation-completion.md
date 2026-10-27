# Inventory and engineering documentation completion plan

## Purpose
Ensure SIP documentation accurately reflects implemented behavior after inventory intelligence PRs #19, #20, #21, #22 and #23 are delivered. Documentation must never present proposed functionality as shipped.

## Dependency gate
- Track implementation status of each of #19–#23 independently.
- Update the relevant feature docs as each feature actually lands; perform a final cross-module documentation audit once all five are merged and verified.
- Do not mark this documentation PR complete while any required feature is unimplemented, undocumented, or contradicted by the current code.
- If upstream APIs, schemas or UI change, document the actual merged behavior, not the original proposals.

## Required documentation
1. **Architecture:** Rust crates/modules, API boundaries, database entities, tenant isolation, background jobs, storage, AI provider integration and data-flow diagrams.
2. **User guides:** add one item, add multiple serialized assets, import spreadsheets, attach evidence, scan labels, review AI proposals, locate manuals and product images, and resolve conflicts.
3. **Engineering service guides:** versioned EBOM versus as-maintained BOM, component positions, replacing parts with new serial numbers, approvals, RMA/disposition, configuration genealogy, compatibility and history lookup.
4. **Lifecycle guides:** consumable lots, batteries, calendar/usage/condition/hybrid policies, approved engineering sources, lifecycle resets, warnings, stale meter data, work orders and service scheduling.
5. **Fleet intelligence:** forecasting assumptions, stock demand, due/overdue queries, aggregation boundaries and uncertainty.
6. **API and agent reference:** actual REST/MCP contracts, request/response examples, authorization scopes, idempotency, pagination, failure modes and rate limits.
7. **Operations:** installation, configuration, migrations, backups, retention, object storage, indexing, job retries, monitoring and rollback.
8. **Security:** uploaded file handling, prompt-injection boundaries, permissions, provenance, sensitive customer data, safety-critical approval gates and audit logging.
9. **Troubleshooting and examples:** realistic non-sensitive sample inventory, serialized replacements, five-year battery policy, bulk imports and common error conditions.
10. **Release notes and navigation:** changelog, version compatibility, README feature summary and links to detailed documentation; avoid bloating the README.

## Acceptance criteria
- Every shipped capability in #19–#23 is covered with accurate, reproducible instructions.
- All examples match live schemas and API behavior and contain no secrets, private data or fabricated manufacturer requirements.
- Documentation links resolve; code snippets and API examples are validated where practical in CI.
- Screenshots/diagrams represent the current UI and architecture.
- Features not yet released are explicitly labeled planned or experimental.
- A maintainer can follow docs to import inventory, replace a serialized part, configure a battery lifecycle policy and inspect fleet maintenance forecasts.
- Final review cross-checks merged implementation PRs against documentation and flags gaps.

## Automation
Add or extend documentation checks in CI where appropriate: link validation, OpenAPI/MCP contract drift, example tests and documentation-change prompts for inventory-related PRs. Avoid requiring documentation updates for unrelated code changes.

## Related implementation PRs
- #19 Inventory foundation
- #20 AI-assisted intake
- #21 Engineering BOM / replacements
- #22 Lifecycle maintenance
- #23 Fleet intelligence

**This file is a planning specification only.** Documentation implementation and validation remain outstanding.
