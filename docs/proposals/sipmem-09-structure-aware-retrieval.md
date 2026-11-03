# Proposal 09: SIPmem Structure-Aware Retrieval (STAIR-Inspired)

**Status:** Planning specification only; implementation is not included in this PR.
**Priority:** P1
**Dependencies:** proposal 03, proposal 04, proposal 05
**Repository scope:** SIP engine, infrastructure and plugins only. No separate website work.

## Purpose
Establish a reviewed, implementation-ready contract for sipmem structure-aware retrieval (stair-inspired) within SIP's modular Rust architecture.

## Implementation requirements
- Preserve chapter, section, subsection, procedure, table, figure, warning and page hierarchy with stable identifiers.
- Add parent-child retrieval, structural candidate generation and contextual expansion alongside existing SQL/lexical/vector retrieval.
- Fuse and rerank candidates while enforcing tenant, document revision and asset applicability constraints.
- Return exact manual/revision/section/page citations and include relevant surrounding safety warnings.
- Support optional explicitly configured learned section router; do not train or install a default model.
- Benchmark against same-corpus hybrid baseline using Recall@1/@5, citation correctness, latency, cost and safety-context coverage.
- Gate production activation on measured improvements and retain baseline fallback.

## Acceptance criteria
- [ ] Hierarchical citations are source-grounded.
- [ ] No cross-tenant or superseded-current confusion.
- [ ] Strategy can be toggled for A/B evaluation.
- [ ] No default AI model is introduced.

## Engineering constraints
- No implicit AI model/provider defaults; all enabled AI capabilities require explicit customer configuration.
- Preserve tenant isolation, source provenance, auditability and backward compatibility.
- Add migrations, automated tests, operational metrics and documentation during implementation.
- Never commit secrets, personal information or sensitive diagnostic samples.
- Review current code before implementation; this document is a proposal, not proof of existing features.

## Delivery process
Implementation should be a separate follow-up change or a reviewed expansion of this proposal branch. Do not merge this planning PR as if it implements the feature.
