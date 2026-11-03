# Proposal 03: SIPmem Runtime Integration and Retrieval Baseline

**Status:** Planning specification only; implementation is not included in this PR.
**Priority:** P0
**Dependencies:** proposal 01
**Repository scope:** SIP engine, infrastructure and plugins only. No separate website work.

## Purpose
Establish a reviewed, implementation-ready contract for sipmem runtime integration and retrieval baseline within SIP's modular Rust architecture.

## Implementation requirements
- Audit sipmem-core, sipmem-adapters, sip-ai, migrations and runtime call paths before implementation.
- Replace placeholder RetrievalService with functional orchestrator connected to existing interfaces.
- Support authorized SQL exact lookup, lexical search, vector retrieval, relationship/temporal lookup where available.
- Enforce tenant isolation and access control before model context construction.
- Return structured provenance, source citations and verification outcomes.
- Create benchmark fixtures for exact, semantic, relationship, temporal and missing-evidence queries.

## Acceptance criteria
- [ ] Real authorized data is retrievable end to end.
- [ ] No cross-tenant retrieval leakage.
- [ ] SQL-only queries work without AI.
- [ ] Recall, latency and citation correctness baseline recorded.

## Engineering constraints
- No implicit AI model/provider defaults; all enabled AI capabilities require explicit customer configuration.
- Preserve tenant isolation, source provenance, auditability and backward compatibility.
- Add migrations, automated tests, operational metrics and documentation during implementation.
- Never commit secrets, personal information or sensitive diagnostic samples.
- Review current code before implementation; this document is a proposal, not proof of existing features.

## Delivery process
Implementation should be a separate follow-up change or a reviewed expansion of this proposal branch. Do not merge this planning PR as if it implements the feature.
