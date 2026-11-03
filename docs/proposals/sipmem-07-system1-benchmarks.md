# Proposal 07: System 1 Router and Model Benchmark Harness

**Status:** Planning specification only; implementation is not included in this PR.
**Priority:** P1
**Dependencies:** proposal 01, proposal 03
**Repository scope:** SIP engine, infrastructure and plugins only. No separate website work.

## Purpose
Establish a reviewed, implementation-ready contract for system 1 router and model benchmark harness within SIP's modular Rust architecture.

## Implementation requirements
- Implement rules-first, SQL-first task routing without invoking LLMs for deterministic requests.
- Offer optional customer-configured small-model classification, extraction and escalation; never select a default model.
- Support explicit confidence thresholds, abstention, fallback policies and per-task budgets.
- Benchmark configured models on retrieval, technical support, inventory and temporal datasets for quality, p50/p95 latency, cost, memory and throughput.
- Track model/provider/version, decisions, errors and tenant-safe traces; support rollback and shadow evaluation.

## Acceptance criteria
- [ ] No implicit model selection.
- [ ] Deterministic queries use zero inference calls.
- [ ] Low-confidence cases abstain or follow configured escalation.
- [ ] Reproducible benchmarks and tenant-safe audit traces exist.

## Engineering constraints
- No implicit AI model/provider defaults; all enabled AI capabilities require explicit customer configuration.
- Preserve tenant isolation, source provenance, auditability and backward compatibility.
- Add migrations, automated tests, operational metrics and documentation during implementation.
- Never commit secrets, personal information or sensitive diagnostic samples.
- Review current code before implementation; this document is a proposal, not proof of existing features.

## Delivery process
Implementation should be a separate follow-up change or a reviewed expansion of this proposal branch. Do not merge this planning PR as if it implements the feature.
