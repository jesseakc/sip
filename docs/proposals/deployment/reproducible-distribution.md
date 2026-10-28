# Reproducible commercial SIP operator distribution

**Priority:** P0
**Category:** Deployment
**Status:** Planning proposal, not implementation

## Scope and acceptance criteria
Produce source-build and prebuilt-container distributions plus documented independent operator bootstrap. No mandatory official SIP account, endpoint, license key or hosted service. Include deployment templates, admin setup, secret generation, upgrades and AGPL source/notice obligations. Acceptance: clean-room deployment on independent domain with local-only services and operator-controlled tenant onboarding.

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
