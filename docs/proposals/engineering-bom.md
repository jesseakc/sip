# Engineering BOM and serialized component replacement genealogy

## Goal
Track engineering BOM revisions and the actual as-maintained configuration of each installed system.

## Requirements
- Versioned EBOM with positions, quantities, approved parts, substitutions, revision compatibility and supporting engineering evidence.
- As-maintained hierarchical asset configuration with stable component positions and temporal installation records.
- Capture installed part number, manufacturer serial, firmware/hardware revision, technician, timestamp, work order and customer site.
- Replacement workflow: select existing component, scan new serial/part, validate compatibility, assign removed disposition (RMA, repair, quarantine, scrap, retain), attach evidence and required functional verification.
- Atomic removal/installation transaction; no serialized component installed in two active positions; idempotent submissions.
- Preserve all historical configurations and reconstruct installed components at any past timestamp.
- Search affected serial ranges, lots and revisions for service bulletin/recall impact.
- Mobile-first field service UI and agent API; AI can draft but cannot independently authorize safety-critical substitutions or return-to-service signoff.

## Acceptance criteria
- Replace one serialized component and verify historical/current configurations and immutable audit history.
- Duplicate serial placement and incompatible substitution are rejected.
- A failed transaction leaves the original configuration unchanged.
- Query affected serial range across current and historical installations.
- All mutations honor tenant scope and technician permissions.

## Dependencies
Inventory foundation; optional evidence enrichment integration.

## Delivery
Implement domain, migrations, UI/API, tests and operational docs before marking complete.