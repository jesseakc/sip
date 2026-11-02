# DIST-003: Microsoft 365 Copilot distribution

**Status:** Implementation proposal. No Entra app, tenant deployment, Partner Center offer, or store submission is created.
**Checked:** 2026-11-02. **SIP baseline:** `c3ad101ab5b60ca6cce3d23277bf8ce7a37c35ca`.

## Separate product boundary

GitHub Copilot/VS Code/CLI (#15) and Microsoft 365 Copilot are different distribution and runtime surfaces. The existing coding-harness recipe is not a Microsoft 365 Agent Store package.

Microsoft documents a declarative-agent route using an MCP plugin and its Agents Toolkit. The manifest references a running remote MCP server; it does not host that server. Public distribution requires an eligible package/offer type, publisher program enrollment, Partner Center submission, and applicable certification. A successful organizational pilot is not proof of public Marketplace eligibility.

## Proposed implementation slices

1. Select a supported initial Microsoft 365 experience and current package type. Record version, licenses, tenant permissions, tool-discovery model, auth restrictions, and public-offer eligibility.
2. Scaffold a small declarative agent/plugin wrapper around #5, with bounded maintenance context and explicit sources. Keep all service authorization in SIP.
3. Add user-delegated sign-in/account linking where supported. Bind an external identity to an explicitly authorized SIP principal and tenant; possession of a Microsoft account is not permission to read SIP.
4. Run a sideloaded synthetic-tenant pilot. Document administrator consent/assignment separately from each user's SIP grant.
5. Prepare listing and certification artifacts, including exact tested package, support/legal/availability details, reviewer instructions, and any required policy attestations.
6. Submit an eligible offer through the current supported Partner Center flow only after the operator reviews the payload. Track approval, publication, installability, authorization, and discovery independently.
7. Test update/removal and the behavior of organizations that deny installation or consent.

Proposed future package: `integrations/distribution/microsoft-365/`, with manifests generated/validated by a pinned current toolkit and synthetic conformance fixtures. Do not hard-code a supposedly universal manifest version or publish a raw MCP URL as if it were already a certified agent.

The Agent 365 BYO MCP/Tooling Gateway route is a separate product surface with its own preview and support boundaries. Do not silently substitute it for the declarative-agent route when eligibility or tool support differs.

## Acceptance criteria

- [ ] The exact package/offer type is documented as eligible for the intended public channel.
- [ ] A fresh test organization can install/assign the pilot and complete a real SIP fixture lookup.
- [ ] Two tenant grants, revoked grants, and organization policy denials are tested.
- [ ] Tool discovery and authenticated results match the release package.
- [ ] Reviewer credentials never appear in the repository, screenshots, or published assets.
- [ ] Certification receipts and actual public listing URL are recorded only after those steps occur.
- [ ] Removal revokes access and leaves unrelated Microsoft/SIP grants intact.

## Dependencies

Requires #5, distribution readiness, deployed HTTPS/auth infrastructure, and operator-controlled publisher/tenant accounts. Preserve the current OBS/cloud rollout order. Do not merge or close #15 merely because this proposal exists.

## Primary references

- [Build a declarative agent with an MCP plugin](https://learn.microsoft.com/en-us/microsoft-365/copilot/extensibility/build-mcp-plugins)
- [Choose organizational versus public distribution](https://learn.microsoft.com/en-us/microsoft-365/copilot/extensibility/publish)
- [Public Marketplace eligibility and Partner Center flow](https://learn.microsoft.com/en-us/microsoft-365/copilot/extensibility/publish-plugin-publicly)
- [Separate Agent 365 BYO MCP boundaries](https://learn.microsoft.com/en-us/microsoft-agent-365/bring-your-own-mcp)
