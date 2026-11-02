# DIST-005: Meta Muse and Grok catalog eligibility

**Status:** Eligibility and implementation proposal. No application, account connection, or catalog submission is performed.
**Checked:** 2026-11-02. **SIP baseline:** `c3ad101ab5b60ca6cce3d23277bf8ce7a37c35ca`.

## Why these channels need explicit gates

The shared MCP gateway makes a connection technically possible only after transport/auth compatibility is proven. It does not grant a public catalog listing. Neither a secondary directory count nor a successful custom connection establishes seller eligibility.

## Meta Muse

Meta's developer announcement links its Muse Connector Platform at platform.muse.ai. Readable public material in this review establishes the platform's existence, but not an unrestricted self-service SIP publication path or complete submission requirements.

Proposed first slice is a bounded eligibility spike:

- Confirm the official publisher application/contact route, program access, eligible use cases, supported markets, and whether a maintenance/service-data product is accepted.
- Obtain the current MCP/API protocol and auth requirements from an accessible official source or provider response. Keep Meta AI glasses/WebMCP preview interfaces distinct from the Muse catalog.
- Record distribution/branding/support requirements, data handling, and review/publishing steps with dates and source links.
- Produce an exact package/deployment plan only after access and requirements are confirmed. If access is unavailable, record blocked_eligibility and the evidence rather than inventing a manifest or claiming support.

A later implementation reuses #5 and runs two-tenant permission/revocation tests, followed by a clean-account listing/install test. Proposed future directory: `integrations/distribution/meta-muse/`; no operative package is added here.

## Grok conversational connectors

Official Grok documentation describes connecting a custom public MCP endpoint through grok.com/connectors, with separate team provisioning controls. That is a useful pilot route, but it does not establish public catalog inclusion.

Proposed slices:

1. Test the shared authenticated gateway as a custom connector using only a synthetic SIP tenant.
2. Confirm the current official publisher/catalog intake route, owner verification, required assets, review criteria, and update/removal workflow.
3. Record connection success and public-distribution eligibility separately.
4. If a listing route is available, prepare an operator-reviewable packet and later record its submission, approval, publication, and search evidence.

PR #7 is for Grok Bot's Cursor-compatible integration. Grok conversational connector publication is a separate surface; a Cursor Marketplace listing or working Grok Bot plugin does not automatically create a grok.com catalog entry.

Proposed future directory: `integrations/distribution/grok-catalog/`. Do not substitute a model inference API integration for this connection/distribution work.

## Acceptance criteria

- [ ] Both channels have dated official eligibility/intake evidence or a clearly documented blocker.
- [ ] No guessed console URL, manifest schema, quota, directory count, or review SLA becomes a requirement.
- [ ] Access restrictions distinguish an unsupported endpoint from a not-yet-admitted publisher.
- [ ] A custom pilot passes actual fixture calls and tenant/revocation denial tests.
- [ ] Public listings are claimed only with an exact app ID/URL and clean-account verification.
- [ ] The operator reviews terms, publisher identity, and exact payload before any public submission.
- [ ] A platform eligibility blocker does not block the shared gateway or other channel packages.

## Dependencies and sources

Follows #5, distribution readiness, and existing deployment priorities. Neither channel is marked runtime-supported or published by this proposal.

- [Meta's Muse Connector Platform announcement](https://developers.meta.com/blog/meta-connect-recap/)
- [Official Muse Platform destination linked by Meta](https://platform.muse.ai/)
- [Grok custom connector documentation](https://docs.x.ai/grok/connectors)
- [Grok team connector management](https://docs.x.ai/grok/connector-management)
- [Grok Bot's distinct Cursor-compatible surface](https://x.ai/bot/guides/grok-bot-101)
