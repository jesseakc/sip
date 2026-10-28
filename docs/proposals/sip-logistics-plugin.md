# SIP Logistics plugin proposal

## Objective
Create an optional, independently deployable logistics domain plugin for SIP, supporting self-hosted and SIP Cloud deployments, without adding logistics-specific coupling to core. This PR is a planning specification, not an implementation claim.

## Market scenarios
1. Road freight and refrigerated cargo: breakdown response, temperature excursions, reefer repair.
2. Ocean containers and barges: container events, custody, manifest, intermodal transfers.
3. Construction and rental fleets: machine telemetry, hour-based service, availability.
4. Warehouse and distribution: pallets, lots, serialized units, inventory reconciliation.
5. Field-service logistics: technician/parts dispatch, mobile work orders, proof of repair.
6. High-value equipment: shock, vibration, tamper, insurance and audit evidence.
7. Medical/pharma cold chain: validated temperature monitoring, chain of custody, exception handling.
8. Rail and intermodal: railcar, container, trailer interchange and asset service.
9. Reverse logistics: returns, recalls, quarantine, repair and redeployment.

## Domain model
Tenant-scoped Asset, Vehicle, Equipment, Shipment, ShipmentLeg, Stop, Route, CargoUnit, HandlingUnit, Container, Manifest, CommodityLine, Lot, Serial, CustodyTransfer, LocationObservation, ConditionObservation, Sensor, Geofence, Alert, Exception, ServiceIncident, Dispatch, Technician, ServicePart, WorkOrder, MaintenanceSchedule, ProofOfDelivery, Document and IntegrationSource.
Distinguish movable assets from cargo, containers, vehicles, shipments and ownership. Model nested containment and time-bounded assignments, with historical as-of queries. No assumption that GPS represents cargo position after detachment. Store source, observed_at, received_at, accuracy, freshness and confidence for each location/condition reading. Use append-only events and correction/supersession.

## Functional requirements
- Inventory/asset integration with existing SIP IDs, BOM, serialized parts and disposable replacement schedules.
- Manifest hierarchy: shipment -> containers/pallets -> lots/serials; load/unload scans, quantity reconciliation, overage/shortage/damage and proof of custody.
- Multimodal journeys: truck, rail, vessel, barge, air, warehouse, intermodal handoffs; ETA and exception states.
- Map-based fleet and cargo view, historical playback, geofences, stale-data flags, offline mobile capture.
- Telemetry ingestion (GPS, odometer, engine hours, DTC, temperature, humidity, shock, door, battery and reefer alarms), deduplication and configurable thresholds.
- Asset-in-motion maintenance: safe stop and escalation, qualified dispatch, parts availability, work order, remote triage, service documentation, return-to-service checks.
- Dispatch scoring using technician certification, hours, location, availability, inventory, SLA and travel estimates; manual override; no autonomous safety-critical dispatch without authorization.
- Rules and optional AI for anomaly detection, predictive maintenance, ETA risk and incident summaries. Provenance, explainability, human approval, false-positive feedback.
- Customer-facing tracking with tenant-safe, role-limited access; selective disclosure of location and cargo value.
- Reporting: downtime, MTTR, on-time delivery, temperature excursions, claims, cargo discrepancies, asset utilization, cost per mile/hour, service response and recovery.

## Architecture
- Separate plugin crates/services, migrations, API routes, permission scopes, background workers and optional UI.
- SIP core changes only for generic extension points: typed asset references, event ingestion, plugin registration, geospatial primitives, authorization and job hooks, where missing. Avoid duplicating existing facilities.
- Connector adapters behind versioned contracts; no hard dependency on a commercial provider.
- Standards roadmap: DCSA Track & Trace 2.2 first, versioned 3.0 mapping when accessible (3.0 public release planned H2 2027); ISO 15143-3/AEMP mixed fleet; NMFTA DSDC LTL visibility; GS1 identifiers/EPCIS; EDI X12 204/214/210/856; optional AIS and telematics APIs. Validate licensing and access constraints.
- Security: tenant isolation, fine-grained location/cargo permissions, encryption, retention policies, webhook signing, replay defense, idempotency, immutable audit trail, export/deletion and offline conflict resolution.
- Privacy: do not expose live driver location to unauthorized parties; configurable precision/delay.
- No website implementation in this repo; infrastructure, APIs, plugin UI components and docs only.

## Implementation phases and suggested follow-up PRs
1. Core extension-point audit and minimal generic contracts.
2. Logistics domain schema, migrations and event model.
3. Cargo manifest, nesting, barcode/QR/RFID and custody workflow.
4. Location tracking, geofences, event replay and stale-data handling.
5. Telemetry ingestion and condition alerts.
6. Mobile breakdown and service dispatch integration.
7. Multimodal journey/ETA and exception engine.
8. Standards connectors: DCSA, ISO/AEMP, LTL, GS1/EDI.
9. Optional AI risk scoring and recommendation engine.
10. Dashboard/API and role-based customer portal components.
11. Security, scalability, synthetic demos, load testing and documentation.

## Acceptance criteria
- Can track moving asset independently of its cargo and carrier, including transfer between vehicles.
- Can answer where a serialized item was last observed, what contains it, and when that observation occurred.
- Can trigger a refrigerated trailer incident, locate a qualified responder and matching serialized part, and close a linked work order.
- Can ingest duplicated/out-of-order events without corrupting custody or location history.
- Can operate with zero commercial telematics integrations using synthetic/manual events.
- Tenant isolation, permissions, audit history, integration failure handling and migration rollback are tested.
- Existing SIP installations work with logistics disabled.
- Document standards version compatibility, data freshness, API contracts and runbooks.

## References
- https://dcsa.org/standards/track-and-trace
- https://dcsa.org/newsroom/track-and-trace-3-0-0-iot-reefer-visibility
- https://www.aem.org/standards/iso15143/3
- https://dsdc.nmfta.org/apis/in-transit-visibility-api-standard
