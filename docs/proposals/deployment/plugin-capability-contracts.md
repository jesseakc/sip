# Plugin capability contracts and lifecycle

**Priority:** P0
**Category:** Core
**Status:** Planning proposal, not implementation

## Scope and acceptance criteria
Extend sip-plugins with versioned provider capability traits, typed errors, capability discovery, init/health/shutdown lifecycle, compatibility negotiation and optional-service failure behavior. Preserve existing manifests and iframe hosting. Acceptance: sample local and remote providers conform to identical contracts; disabled provider never blocks unrelated features; unit and contract tests.

## Architectural principles
- SIP Core remains open-source and independently commercially deployable.
- Cloud capabilities are optional and provider-replaceable; local, hosted and hybrid configurations are first-class.
- Existing Rust crates and plugin contracts should be extended, not duplicated.
- Preserve tenant isolation, auditable permissions and migration compatibility.
- Document configuration, dependencies, tests and operator-facing setup.

## Implementation checklist
- [ ] Inspect affected crates and identify integration points
- [ ] Define backwards-compatible interfaces and migrations
- [ ] Implement with tests and secure defaults
- [ ] Document self-hosted and hosted configurations
- [ ] Validate local-only and relevant hybrid deployment modes

## Dependencies
Coordinate with P0 capability contracts, provider configuration, plugin dependency management, operator isolation and security work before implementation where applicable.
