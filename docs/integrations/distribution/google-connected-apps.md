# DIST-002: Google Connected Apps

**Status:** Implementation proposal; no preview application, account connection, or submission is performed.
**Checked:** 2026-11-02. **SIP baseline:** `c3ad101ab5b60ca6cce3d23277bf8ce7a37c35ca`.

## Outcome and verified surface

Distribute SIP's service-record tools through Google's Connected Apps platform for the Gemini app and applicable Search experiences. Google's October 28 announcement describes a phased preview. Access, supported regions, product availability, and SIP's category eligibility must be confirmed for the actual publisher.

This is separate from the Gemini CLI extension in #14. CLI manifest validation or a fake MCP server test cannot establish a Connected Apps listing.

## Proposed implementation slices

1. Confirm preview access and publishing identity. Record the supported category, surface, region, owner, and any partner conditions. Do not treat access application submission as approval.
2. Reuse #5's remote gateway with a stable HTTPS deployment. Prefer the remote-server route for the first release; Android AppFunctions requires a separate Android product/implementation and is not implied by this work.
3. Build a Connected Console configuration adapter and metadata packet. Supply descriptions, real capabilities, example prompts, brand assets, support/privacy/terms URLs, and synthetic reviewer instructions.
4. Implement the current domain challenge on an operator-owned domain. Keep that proof distinct from the Official MCP Registry's namespace proof.
5. Implement delegated user authorization and exact app-specific redirect registration. Use the chosen supported registration/discovery flow; never replace end-user tenant grants with one broad shared service credential.
6. Expose only the allowed tool snapshot and truthful read/write annotations. Sync and record the exact version before testing.
7. Run a small tester pilot and submit the reviewed release packet when the operator approves it. Record Search and Gemini availability separately.
8. Add update/rollback instructions. A changed gateway schema must not silently invalidate the tool snapshot associated with a published version.

Proposed future files: `integrations/distribution/google/` for metadata adapters, `tests/integration/google_connected_apps/` for profile fixtures, and a public setup/support guide. Do not commit generated account IDs, domain challenge values, credentials, or guessed production URLs as fixtures.

## Acceptance criteria

- [ ] Publisher has confirmed preview/category access and selected the intended surfaces.
- [ ] Domain and HTTPS checks pass against the actual review deployment.
- [ ] Two users with different SIP grants authenticate without cross-tenant data access.
- [ ] Revocation, expired authorization, insufficient scope, and a blocked callback fail safely.
- [ ] A test user can invoke an actual fixture tool in Gemini Developer Mode, with source IDs and observed timestamps.
- [ ] Console metadata and exposed tool snapshot match the release hash.
- [ ] Reviewer access uses only an isolated synthetic tenant and platform-approved secure channels.
- [ ] Approval, publication, and a clean-account discovery test are recorded separately for each available surface.
- [ ] Update/removal and credential-revocation procedures are documented.

## Dependencies and sequencing

Depends on #5, the distribution-readiness contract, and a deployable gateway with enforced tenant isolation. Metadata preparation can proceed earlier; no runtime support is claimed until live tests pass. Keep the existing observability/cloud priorities.

## Primary references

- [Google's launch and phased preview](https://blog.google/innovation-and-ai/technology/developers-tools/connected-apps-one-place/)
- [Remote MCP development and test path](https://developers.google.com/connected-apps/mcp)
- [Connected Console overview](https://developers.google.com/connected-apps/gpc)
- [Registration, review, and version publication](https://developers.google.com/connected-apps/gpc/perform-the-steps)
