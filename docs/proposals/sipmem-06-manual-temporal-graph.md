# Proposal 06: Technical Manual Intelligence and Temporal Graph

**Status:** Planning specification only; implementation is not included in this PR.
**Priority:** P1
**Dependencies:** proposal 03, proposal 04, proposal 05
**Repository scope:** SIP engine, infrastructure and plugins only. No separate website work.

## Purpose
Establish a reviewed, implementation-ready contract for technical manual intelligence and temporal graph within SIP's modular Rust architecture.

## Implementation requirements
- Model manual applicability by manufacturer, family, serial range, component, firmware and effective date.
- Represent typed relationships applies-to, supersedes, installed-in, replaced-by and documented-by with evidence and provenance.
- Extract sections, tables, error codes and part references through explicitly configured parser/model adapters.
- Distinguish source assertions from approved operational facts and preserve contradictory evidence.
- Enable as-of temporal queries and historical revision comparisons; require human approval for safety-critical promotion.

## Acceptance criteria
- [ ] Specific serial/revision filters exclude inapplicable manuals.
- [ ] Historical guidance remains accessible and clearly marked.
- [ ] Relationships cite source revisions.
- [ ] Unverified AI outputs never silently become authoritative facts.

## Engineering constraints
- No implicit AI model/provider defaults; all enabled AI capabilities require explicit customer configuration.
- Preserve tenant isolation, source provenance, auditability and backward compatibility.
- Add migrations, automated tests, operational metrics and documentation during implementation.
- Never commit secrets, personal information or sensitive diagnostic samples.
- Review current code before implementation; this document is a proposal, not proof of existing features.

## Delivery process
Implementation should be a separate follow-up change or a reviewed expansion of this proposal branch. Do not merge this planning PR as if it implements the feature.
