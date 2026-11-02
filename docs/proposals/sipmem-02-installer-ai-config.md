# Proposal 02: Installer Wizard: Mandatory AI Configuration

**Status:** Planning specification only; implementation is not included in this PR.
**Priority:** P0
**Dependencies:** proposal 01
**Repository scope:** SIP engine, infrastructure and plugins only. No separate website work.

## Purpose
Establish a reviewed, implementation-ready contract for installer wizard: mandatory ai configuration within SIP's modular Rust architecture.

## Implementation requirements
- Build installer-plugin AI step with no preselected provider, model, or deployment mode.
- When AI is enabled, require applicable provider, model identifier, endpoint, and authentication configuration; local no-auth runtimes must not require fake keys.
- Allow local/cloud/hybrid setups, multiple profiles and explicit task assignments, with capability-aware validation.
- Test connectivity, authentication, model availability, supported modalities and embedding dimension before enabling features.
- Allow skip-AI installation, later configuration, resumable setup, safe secret handling, and CLI/container parity via shared core validation.

## Acceptance criteria
- [ ] No defaults or automatic fallback models.
- [ ] Enabled AI cannot pass installation with missing mandatory fields.
- [ ] AI-disabled install succeeds; configured model tests show actionable errors.
- [ ] No secret is logged, committed, or exported.

## Engineering constraints
- No implicit AI model/provider defaults; all enabled AI capabilities require explicit customer configuration.
- Preserve tenant isolation, source provenance, auditability and backward compatibility.
- Add migrations, automated tests, operational metrics and documentation during implementation.
- Never commit secrets, personal information or sensitive diagnostic samples.
- Review current code before implementation; this document is a proposal, not proof of existing features.

## Delivery process
Implementation should be a separate follow-up change or a reviewed expansion of this proposal branch. Do not merge this planning PR as if it implements the feature.
