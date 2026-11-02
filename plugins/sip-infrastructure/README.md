# SIP Infrastructure (INF-006)

Optional first-party hybrid plugin for physical AI infrastructure and energy facilities.

## Scope of this foundation
- Plugin identity, declarative navigation, capability and permission declarations.
- Two profile definitions under `profiles/` for later asset template registration.
- Architecture and implementation contract for subsequent INF PRs.
- No new database schema, live connector, telemetry ingestion, hardware control or AI runtime.
- Plugin disabled by default. Enable explicitly using SIP's supported plugin configuration.

## Important integration boundary
The existing SIP Rust crate `crates/sip-infrastructure` is the persistence layer, not this plugin. Avoid naming a new Rust crate `sip-infrastructure`. Future backend code should use a distinct crate name such as `sip-infrastructure-plugin` and register through the existing authenticated API.

The plugin manifest is discoverable using the current registry. A manifest alone does not implement its UI or backend. The `/infrastructure` route must be wired to a real page or hosted bundle before this plugin can be considered functional. Until then, keep it disabled.

## Follow-on dependencies
INF-001 typed asset relationships; INF-002 component identity; INF-003 documents; INF-004 SIPmem runtime; INF-005 plugin runtime hardening; INF-007 data center profiles and UI.

## Safety
All future hardware connectors are read-only by default. Any operational action needs separate permissions, human approval, and immutable audit records. All queries must use existing tenant authorization.

## Acceptance criteria for INF-006
1. Manifest parses and validates under the existing SIP registry.
2. Disabled-by-default plugin does not add navigation or expose an inaccessible route.
3. Enabling the plugin is documented and produces a usable, authenticated landing page (not a 404).
4. Data Center and Energy profile descriptors validate.
5. Tests cover disabled/enabled discovery and route access.
6. No modifications to core asset schema, no fabricated live operational data.
