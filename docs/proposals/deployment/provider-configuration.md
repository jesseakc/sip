# Provider selection and configuration resolution

**Priority:** P0
**Category:** Core
**Status:** Planning proposal, not implementation

## Scope and acceptance criteria
Extend sip-config and Sip.toml with per-capability local/remote/disabled provider choices, environment overrides, secret references, validation and redacted diagnostics. Support mixed deployments and fail closed on missing required dependencies. Acceptance: local-only, cloud-only and hybrid fixtures; no secrets in logs; restart requirements documented.

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
