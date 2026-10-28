# Pluggable identity, OIDC and SSO

**Priority:** P1
**Category:** Connector
**Status:** Planning proposal, not implementation

## Scope and acceptance criteria
Add external OIDC issuer validation/JWKS rotation and local identity provider option, organization membership mapping, MFA/passkey integration points, account linking, session revocation and cross-brand SSO. Keep independent operator control and existing auth migration path. Acceptance: Keycloak-compatible local OIDC and hosted OIDC integration tests, tenant boundary and logout tests.

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
