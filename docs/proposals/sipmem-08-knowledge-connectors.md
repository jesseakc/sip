# Proposal 08: Knowledge Connectors and Portability

**Status:** Planning specification only; implementation is not included in this PR.
**Priority:** P2
**Dependencies:** proposal 04, proposal 05
**Repository scope:** SIP engine, infrastructure and plugins only. No separate website work.

## Purpose
Establish a reviewed, implementation-ready contract for knowledge connectors and portability within SIP's modular Rust architecture.

## Implementation requirements
- Create pluggable sync contracts for file shares, SharePoint, Google Drive, Confluence and other document repositories.
- Support incremental sync, checkpoints, revision detection, tombstones, retries and deletion propagation.
- Preserve and revalidate source permissions; do not expose revoked source content.
- Export original files, extracted text, metadata, provenance and graph relationships in documented open formats.
- Track embedding model/version/dimensions per index generation and support background re-embedding with atomic index cutover.
- Protect connector credentials and prevent sensitive content in logs or exports.

## Acceptance criteria
- [ ] Incremental sync works and source revocation removes retrieval access.
- [ ] Export does not require provider-specific vector data.
- [ ] Model changes do not destroy knowledge.
- [ ] Partial failures are recoverable.

## Engineering constraints
- No implicit AI model/provider defaults; all enabled AI capabilities require explicit customer configuration.
- Preserve tenant isolation, source provenance, auditability and backward compatibility.
- Add migrations, automated tests, operational metrics and documentation during implementation.
- Never commit secrets, personal information or sensitive diagnostic samples.
- Review current code before implementation; this document is a proposal, not proof of existing features.

## Delivery process
Implementation should be a separate follow-up change or a reviewed expansion of this proposal branch. Do not merge this planning PR as if it implements the feature.
