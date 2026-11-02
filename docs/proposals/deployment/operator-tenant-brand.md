# Operator, tenant and brand isolation

**Priority:** P0
**Category:** Core
**Status:** Planning proposal, not implementation

## Scope and acceptance criteria
Introduce explicit operator/organization/brand/product identity and authorization boundaries, tenant-safe brand routing, membership/entitlement models, and RLS tests. Maintain compatibility with existing organization_id policies. Acceptance: cross-tenant and cross-operator access denied across API, AI retrieval and plugins; multiple brands share permitted identity without sharing data.

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
