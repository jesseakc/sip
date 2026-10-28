# Production deployment security hardening

**Priority:** P0
**Category:** Security
**Status:** Planning proposal, not implementation

## Scope and acceptance criteria
Make production CORS allowlists mandatory, reject development secrets and demo accounts, enforce HTTPS/proxy trust boundaries, add security headers, rate limiting, signup abuse controls, MFA-ready flows and security regression tests. Existing docker-compose defaults must be clearly local-only. Acceptance: unsafe production config fails startup; no public database/cache/storage admin ports in production templates.

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
