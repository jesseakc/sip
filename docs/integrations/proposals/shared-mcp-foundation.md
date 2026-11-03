# Shared SIP MCP foundation integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Problem and repository evidence

SIP has a service API, an AgentIdentity entity, and McpToolDef manifest declarations. It does not yet serve MCP. SIP-native UI plugins are a separate extension system; a UI plugin manifest is not an installable external-agent integration.

Baseline evidence: crates/sip-plugins/src/manifest.rs, crates/sip-domain/src/entity/agent_identity.rs, crates/sip-api/src/main.rs, and docs/architecture.md's planned MCP tool-serving entry. Existing authentication uses user JWT login/refresh, not an MCP OAuth authorization server. There is no live MCP URL or published SIP agent package to install.

## Proposed architecture

Add a small gateway around SIP application services, with one tool contract and thin client packages. Propose crates/sip-mcp for transport/dispatch and typed adapters; wire it into sip-api behind an off-by-default capability. Use the same domain authorization and database transaction boundaries as the HTTP API. No direct SQL interface, arbitrary HTTP proxy, shell execution, or model-provider credential access belongs in these tools.

Hosted clients need an authenticated HTTPS MCP endpoint. Local clients may use a separately packaged stdio bridge to that endpoint; keep protocol frames on stdout and diagnostics on stderr. The endpoint path, executable/package names, SDK, and supported protocol versions are implementation decisions still to be reviewed, not current products. Select a maintained SDK after a proof of interoperability. Record negotiated protocol and transport per client; do not assume that every host supports the newest MCP revision or event methods.

## Initial tool contract

All names below are proposed. Inputs reject unknown fields, validate UUIDs, bound page size, and omit tenant identity from user-controlled arguments.

| Proposed tool | Result | SIP permission / implementation |
| --- | --- | --- |
| sip_assets_list | Bounded asset page and explicit continuation/completeness information | asset:read; extend current API pagination where needed |
| sip_assets_get | One asset, stable ID, source URL, observed version/time | asset:read; GET /api/v1/assets/{id} contract |
| sip_work_orders_list | Bounded work-order page; structured filters | work_order:read; define filters/pagination rather than silently clipping |
| sip_work_orders_get | One work order with canonical status and version | work_order:read; GET /api/v1/work-orders/{id} contract |
| sip_service_history_get | Ordered records with source IDs and timestamps | work_order:read plus relevant source permissions; add an audited bounded application/API endpoint if necessary |

Return schema-validated structured data and concise source links. Map current flat versus data-enveloped API responses explicitly. Unsupported feature flags and missing history must produce clear unavailable/incomplete results. No inference-only AI retrieval or indexing placeholder may masquerade as verified service evidence.

Phase two may add sip_work_orders_prepare_change and a separately authorized apply operation. A proposal is not a completed business write. Apply needs a server-validated approval record bound to tenant, actor, target, exact change hash, expected version, expiry, and idempotency key. A model-supplied approved=true is not authorization. Enforce lifecycle-specific permissions and audit success/failure transactionally. Keep review, cancellation, user administration, imports, and bulk actions out of the first release.

## Authorization and tenancy prerequisite

Design delegated, scoped authorization for the remote MCP resource. Use current MCP OAuth discovery/PKCE/resource-audience requirements, with registered clients or supported client metadata as appropriate. Store client secrets and refresh credentials in host secret storage; never in repository examples or skills. Local stdio credentials use the host's environment/secret facility. Do not pass a browser's unrestricted user JWT through a plugin or accept a model-selected organization header as authority.

Derive the tenant and permitted actions from the authenticated grant on every request. Enforce inactive/revoked identities, expiry, role changes, and explicit organization predicates. AgentIdentity is attribution metadata until a real authentication/authorization mapping exists. Client labels, model names, and supplied actor IDs are untrusted diagnostics.

Production access depends on resolving the migration/auth issues tracked in PRs #1 and #3 plus the separately identified pool/RLS tenant-isolation and audit-atomicity findings. Read-only operation still requires isolation. A docs-only RFC may merge before those fixes; an operational gateway must not be advertised as safe before its negative tests pass.

## Work packages and completion criteria

- [ ] Decide typed transport/SDK and supported protocol profile with a short interoperability spike.
- [ ] Add the gateway, feature flag, authenticated discovery, schemas, timeouts, cancellation, and bounded responses.
- [ ] Add tenant-bound OAuth grants and revocation; document self-hosted and cloud network requirements.
- [ ] Verify one local client and one hosted client against the same synthetic two-tenant fixture.
- [ ] Reject cross-tenant IDs, permission escalation, expired/revoked tokens, mismatched audiences, and instructions embedded in record text.
- [ ] Verify pagination/completeness, canonical statuses, disabled capabilities, and source links.
- [ ] For later writes, prove denied/stale/duplicate requests have the correct effects and no false success; record the human and integration actor.
- [ ] Publish a versioned client matrix with exact host/server versions, transport, auth method, test date, and evidence. Documentation review alone remains planned.
- [ ] Release only after package provenance, install/uninstall, credential redaction, and upgrade checks pass.

## Rollout and wider coverage

First prove local Claude Code or Hermes plus hosted Dots or Grok Bot, then expand adapters without duplicating business rules. Each companion proposal can be reviewed independently; runtime packages depend on this contract.

Additional candidates beyond the individual proposals include Claude Desktop/Cowork, OpenHands, Aider, Augment, Google Antigravity, JetBrains Junie, and existing Roo Code/Continue installations. These are a discovery queue, not a claim that native plugin APIs or maintained support exist. Confirm current upstream status, transport, auth, licensing, and users before allocating implementation work. Offer an explicit CLI/extension bridge only where an upstream client lacks native MCP; label that dependency.

## Sources

Reviewed 2026-10-27. External protocol facts come from the sources; the SIP design above is a proposal.

- [MCP authorization specification](https://modelcontextprotocol.io/specification/2026-07-28/basic/authorization)
- [OpenAI portable plugin packaging](https://developers.openai.com/plugins/build/plugins)
- [Hermes MCP support](https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp)
- [Cursor MCP support](https://cursor.com/docs/mcp)
