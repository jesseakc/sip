# SIP Inventory Plugin: implementation specification

Status: proposed, planning-only PR. No runtime implementation in this PR.

## Goal
Build an opt-in, cross-industry inventory plugin for SIP. Support grocery stores, warehouses, repair depots, service vans, vessels/barges, retail shops, and other stock-holding organizations. Keep business-specific features in optional extensions rather than SIP Core.

## Architectural boundaries
- Plugin owns its migrations, domain models, APIs, permissions, events, jobs, UI contributions, and tests.
- SIP Core provides stable plugin lifecycle, tenant identity, RBAC, audit, event bus, storage abstractions, and integration contracts where available. Any missing core contracts must be proposed as separate narrowly scoped PRs.
- No inventory tables or domain rules in core. Plugins can run locally or under independently hosted SIP-compatible services, with explicit configuration and capability negotiation.
- Compatible with AGPLv3 and independently deployable commercial SIP distributions under applicable license obligations.
- No website or marketing-site work in this repository.

## Domain model
- Product: immutable ID, tenant, SKU, GTIN/UPC/EAN, descriptions, unit of measure, category, optional variants, pack conversion.
- Inventory location: hierarchical site > zone > aisle > rack > shelf > bin, plus mobile and vessel locations; parent and facility type are metadata, not separate stock ledgers.
- Stock lot: product, lot/batch, expiration/best-by, supplier, condition, owner, cost layer when enabled.
- Serialized unit: individual identity, serial number, current custody/location/condition and linked asset where applicable.
- Stock balance: tenant/product/location/lot/condition/owner dimensions, on-hand, reserved, available, in-transit; authoritative balances derived from or reconciled to ledger.
- Immutable movement ledger: receive, transfer, reserve, release, pick, ship, sell, consume, adjust, return, quarantine, dispose; actor, timestamp, reason, source, correlation ID and idempotency key.
- Counts: cycle counts, physical counts, discrepancy review and approved adjustments.
- Thresholds: reorder policy, min/max, lead time, safety stock and alert preferences.
- External references: optional supplier, purchase order, work order, sales order, shipment and cargo manifest.

## Key behaviors
1. Add one product, multiple variants, or bulk import via CSV with validation, dry run, per-row errors, and deduplication.
2. Barcode/QR lookup and scan-first receiving, transfers, counts, picking and fulfillment; offline scan queue with conflict resolution.
3. Atomic stock movements and reservation rules; no negative stock unless explicit permission/policy.
4. Units, cases, pallets, weight and volume with exact decimal quantities and explicit conversions.
5. FEFO/FIFO picking policies; expiry alerts and lot recalls; cold-chain temperature observations through optional connectors.
6. Warehouse transfers, stock on vehicles/barges, custody changes, and transit reconciliation.
7. Integration with SIP asset/work-order/BOM/serialized replacement workflows via versioned contracts; avoid duplicate sources of truth.
8. Consumables with time-based and usage-based replacement cycles, compatible with prior service-parts planning.
9. Reorder suggestions and optional demand forecasts. Forecasts are advisory and source-grounded; no autonomous purchase orders by default.
10. Multi-tenant isolation, granular permissions for adjustments and stock movements, immutable audit, export and retention controls.
11. Accessible responsive UI, inventory dashboards, search/filter, low-stock and expiry views; API-first for agents and third-party systems.
12. Optional retail/POS, procurement, shipping, ERP, IoT temperature, and accounting adapters. Never make these mandatory.

## Public APIs and events
- Versioned REST endpoints for products, locations, lots, serialized units, balances, movements, reservations, counts, policies, and imports.
- Cursor pagination, filtering, batch operations, idempotency, optimistic concurrency and documented error schemas.
- Events: inventory.received, inventory.moved, inventory.reserved, inventory.released, inventory.consumed, inventory.adjusted, inventory.low_stock, inventory.expiring, inventory.count_reconciled.
- Contract tests for agent access, event consumers, and optional plugins.

## Security and correctness
- Enforce tenant scoping and authorization on every read/write/event and AI retrieval.
- Transactionally update ledger and balances; prevent double posting from retries.
- Guard concurrent reservations and transfers, prohibit invalid unit conversions, preserve movement provenance.
- Require approval for high-risk adjustments and configurable quarantine/recall handling.
- Never use AI inference as authoritative stock count.

## Proposed follow-on PR sequence
1. Core plugin extension-point gap analysis and minimal contracts (only if missing).
2. Inventory plugin scaffold, manifest, migrations and tenant/RBAC wiring.
3. Product catalog, units and barcode identifiers.
4. Hierarchical locations and custody including mobile/vessel locations.
5. Ledger, balance projection, idempotent atomic movement engine.
6. Lots, serials, conditions and expiry/FEFO.
7. Reservations, picks, transfers and fulfillment.
8. Bulk CSV import, export, scanning and offline reconciliation.
9. Counts, discrepancy approvals and audit reports.
10. Reorder policies, alerts and optional forecasting.
11. Optional SIP work-order/BOM/asset and consumables integration.
12. Optional POS, purchasing, ERP, shipping, temperature and accounting connectors.
13. UI dashboard and role-based operational workflows.
14. Documentation, OpenAPI, sample datasets and end-to-end tests.

## Acceptance criteria
- Plugin can be disabled or uninstalled without breaking SIP Core or unrelated plugins; documented data retention/export behavior.
- Two tenants cannot access or affect each other's inventory.
- Concurrent stock operations cannot silently oversell or lose units; retrying the same idempotency key does not duplicate a movement.
- A grocery scenario supports perishable lot receiving, FEFO picking and recall; warehouse scenario supports bins and counts; barge scenario supports cargo in transit and custody.
- A field-service scenario links a serialized part replacement to an asset/work order and updates stock.
- All endpoints, permissions, events, installation and migration paths are documented and tested.
- No website changes and no mandatory cloud dependency.

## Open design decisions for implementation PRs
- Confirm existing SIP plugin SDK, migration conventions, asset/BOM APIs and event bus before writing runtime code.
- Choose stock ledger balance projection and database transaction/locking strategy based on repository patterns.
- Define precise plugin-to-plugin ownership for serialized service parts and asset custody.
- Define licensing and marketplace packaging metadata in coordination with plugin infrastructure PRs.
