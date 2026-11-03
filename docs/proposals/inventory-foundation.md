# Inventory foundation and serialized bulk intake

## Goal
Build a tenant-isolated Rust inventory domain integrated with SIP's existing architecture. Inspect current crates, schema, API conventions and authorization before implementation; reuse existing asset entities where possible.

## Requirements
- Product definitions distinct from inventory lots and physical asset instances.
- Single item, quantity-based, lot-based and individually serialized intake; immutable UUIDv7 IDs and separate human-readable asset tags. Never fabricate manufacturer serials.
- Transactional stock movements, locations, custody, statuses, supplier and purchase associations.
- CSV/XLSX bulk import with mapping preview, validation, partial-error report, idempotency keys and resumable jobs.
- Responsive UI for add-one, add-many, import preview and asset details.
- Tenant isolation, RBAC, audit trail, concurrency control, pagination and safe retry behavior.
- Document public Rust API and migration strategy; add unit, integration and security tests.

## Acceptance criteria
- Create one item, 20 unique asset instances, and import 10,000 rows with deterministic duplicate handling.
- Replaying the same import request does not duplicate stock.
- Manufacturer serials remain optional, separate from SIP-generated asset IDs.
- Cross-tenant reads and writes are denied; every mutation is audited.
- UI supports mobile and desktop and follows existing SIP patterns.

## Dependencies
Foundational PR; implement before AI enrichment, engineering configuration or lifecycle features.

## Delivery
Replace this planning specification with implementation, migrations, tests and documentation before marking complete.