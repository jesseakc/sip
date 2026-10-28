# Proposal 01: Model-Agnostic AI Provider Registry

**Status:** Planning specification only; implementation is not included in this PR.
**Priority:** P0
**Dependencies:** None
**Repository scope:** SIP engine, infrastructure and plugins only. No separate website work.

## Purpose
Establish a reviewed, implementation-ready contract for model-agnostic ai provider registry within SIP's modular Rust architecture.

## Implementation requirements
- Separate generation, embeddings, reranking, vision/document understanding, and routing into capability-specific Rust traits, with compatibility adapters for existing LlmProvider.
- Implement provider/model profile registry with explicit provider type, endpoint, model ID, capabilities, credential secret references, tenant scope, and audit metadata.
- Support local runtimes, native providers and user-defined OpenAI-compatible endpoints without assuming all endpoints support the same API.
- Enforce explicit model choice per enabled capability; no bundled, implicit, or hardcoded default model or fallback.
- Implement endpoint security including SSRF protection, encrypted secrets, redaction, credential rotation, timeout and health checks.

## Acceptance criteria
- [ ] Fresh installation selects no AI provider or model.
- [ ] Independent capabilities may use different customer-selected providers.
- [ ] Incomplete or unsupported profiles fail validation; AI-disabled SIP works.
- [ ] Existing provider implementations remain compatible and tests pass.

## Engineering constraints
- No implicit AI model/provider defaults; all enabled AI capabilities require explicit customer configuration.
- Preserve tenant isolation, source provenance, auditability and backward compatibility.
- Add migrations, automated tests, operational metrics and documentation during implementation.
- Never commit secrets, personal information or sensitive diagnostic samples.
- Review current code before implementation; this document is a proposal, not proof of existing features.

## Delivery process
Implementation should be a separate follow-up change or a reviewed expansion of this proposal branch. Do not merge this planning PR as if it implements the feature.
