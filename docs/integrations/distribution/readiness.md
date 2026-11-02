# DIST-001: MCP publishing and discovery readiness

**Status:** Implementation proposal. No registry or store submission is made by this PR.
**Checked:** 2026-11-02. **Repository baseline:** `c3ad101ab5b60ca6cce3d23277bf8ce7a37c35ca`.

## Why this is separate from the harness PRs

PRs #5-18 cover a gateway and client integrations. A local plugin loading successfully does not establish public store eligibility, approval, or search visibility. This proposal adds a distribution workstream without changing those branches or repeating their implementation.

The user's list is a useful channel inventory. Directory counts and week-over-week growth are third-party snapshots, not acceptance criteria. The original social post was not provided as a source URL. Use the platform-owned links below as the implementation references and recheck them before submitting.

## Ten-channel inventory

| Channel | Verified route or remaining uncertainty | Related work |
| --- | --- | --- |
| Official MCP Registry | Publisher namespace verification and versioned server metadata; remote-only servers can use remotes without a package | #5 |
| ChatGPT / Codex | Verified publisher and reviewed plugin publication through OpenAI's portal | #6, #11 |
| Claude | Directory portal accepts a remote connector or a Git-hosted plugin bundle | #8 |
| Google Connected Apps | Preview platform for Gemini and Search; Connected Console manages registration/review | Separate from Gemini CLI #14 |
| Meta Muse | Meta announces its Connector Platform; public intake/eligibility was not established from readable public instructions | Eligibility spike |
| Grok | Custom remote MCP connection is documented; public catalog inclusion needs a separately confirmed intake | Separate from Grok Bot #7 |
| Microsoft 365 Copilot | Supported Microsoft 365 package and tenant pilot; public offer eligibility and Partner Center certification | Separate from GitHub Copilot #15 |
| Salesforce AgentExchange | MCP cards/tools are documented in Agentforce; obtain the exact applicable partner listing/review route | Enterprise marketplace work |
| AWS Marketplace | Seller product flow includes MCP endpoints and fulfillment/authentication configuration | Enterprise marketplace work |
| Cursor Marketplace | Tested public plugin repository, supported manifest, and submission review | #13 |

The Registry is a preferred early metadata destination once SIP has a working release. It is not a prerequisite imposed by every store and does not automatically publish SIP to them. It hosts metadata, not the server's runtime or package artifacts. A successful namespace check must not be described as remote-domain ownership verification.

Aggregators such as Glama, PulseMCP, and mcp.directory are secondary verification targets. Confirm each current ingestion/submission policy; do not promise automatic propagation, indexing, ranking, or endorsement.

## Proposed repository deliverables

- `targets.json` in this documentation directory: a planning inventory, not a live store manifest. Every SIP runtime/publication status starts as not_verified and has explicit evidence slots.
- Future `integrations/distribution/`: reusable publisher metadata, version/tool snapshots, source links, and host-specific adapters with no embedded credentials.
- Future release tooling: validate metadata and package paths, archive exact release hashes, compare advertised tools against a live authenticated gateway, and produce a review packet.
- A dedicated synthetic reviewer tenant, reset/revoke procedure, stable public review endpoint, support/legal URLs, brand assets, and an operator-selected verified publishing identity.
- The eventual registry server.json must be generated against the current schema and chosen namespace. Do not register a guessed SIP domain, publish a placeholder endpoint, or invent a distributable npm package.

Use the official mcp-publisher validate flow before any registry publication. Choose registry namespace ownership proof deliberately, record the pinned publisher/schema versions, and query the resulting record after publication. Draft artifacts should validate offline without publishing; credential-bound actions are a later release step.

## Domain discovery and MCP Server Cards

Add domain-level discovery to this workstream, with HTTP serving owned by the gateway in #5. The server-card suggestion was not explicit in the original #5 or #83 proposals, and no such artifact was found on the checked main baseline. This section is a proposed implementation, not a published SIP endpoint.

Upstream status checked 2026-11-02: [SEP-2127](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/d0e246ea3c69eafaa2806bc4cd28025703ba2c2b/seps/2127-mcp-server-cards.md) is Final on the Extensions Track; its PR merged on October 6. The extension repository's README still carries an experimental warning. Use the accepted charter and the pinned versioned wire-format specification, and recheck both before implementation rather than inferring current status from that README alone.

The accepted extension is optional. Configured MCP connections continue to work without a card, and publishing one does not guarantee crawler indexing or marketplace admission. Its current design uses an AI Catalog at `/.well-known/ai-catalog.json` to link to a Server Card. The recommended card location is `<streamable-http-url>/server-card`, with `application/mcp-server-card+json`; the catalog uses `application/ai-catalog+json`. For an endpoint configured as `/mcp`, the default card route would be `/mcp/server-card`.

The social post's `/.well-known/mcp/server-card.json` route and instruction to list every tool reflect an older draft. The current card covers remote identity/connectivity and intentionally omits tools, resources, prompts, capabilities, and negotiated extension claims. Enumerate primitives through authenticated runtime operations such as `tools/list`. Registry `server.json`, Server Cards, and OAuth discovery metadata are separate artifacts with separate schemas and purposes.

Proposed SIP implementation and checks:

- [ ] Select and pin the current extension/schema and AI Catalog versions, with source hashes and a review date. The reviewed extension snapshot is `526201bbc80231daa40ffcdecfc9da4e54e5dc93`; do not fetch floating schemas during request handling or silently accept future schema changes.
- [ ] Generate registry metadata, card identity/endpoints, and release evidence from shared deployment configuration. Validate each artifact against its own schema; do not reuse the Registry payload as a card. Require an operator-selected public origin and actual deployed protocol support, without trusting arbitrary Host/forwarded headers or inventing a production URL.
- [ ] Make public discovery an explicit deployment setting, disabled for private installations. Publish only an allowlisted public identity and endpoint description. Exclude credentials, internal URLs, tenant/user identifiers, records, plugin inventory, and permission-dependent primitive listings. Disabling discovery must not disable authorized configured connections.
- [ ] Serve the catalog and card over HTTPS with the pinned media types, read-only CORS, cache validation/ETag, and bounded rate/response limits. Keep card routing separate from MCP transport/SSE handling and existing OAuth metadata. Only public metadata may use shared caches.
- [ ] Test a fresh client following catalog to card to the real MCP endpoint, then authenticating and listing permitted tools. Compare identity, endpoint, versions, and available live discovery metadata against the release; test cross-tenant, anonymous, expired, and revoked access denials. Card contents must never grant access or enable a capability.
- [ ] Rehearse updates and discovery disablement, including cache invalidation and stale-card handling. Test both configured connections without a card and clients that support the card. Record the observed host/version rather than claim universal agent support.
- [ ] Evaluate the older `/.well-known/mcp/server-card.json` route only as a separately versioned compatibility adapter if a target client/scanner needs it. Verify the consumer's expected schema and media type; a redirect or a differently shaped document is not assumed compatible. Any legacy tool summary would need a separate reviewed public allowlist, never a dump of user-specific tools.
- [ ] After a synthetic deployment exists, optionally run [Cloudflare's Agent Readiness scan](https://isitagentready.com/) and attach the date, URL, report, and observed limitations. Do not change authentication, disclose data, or add unsupported capabilities solely to improve the score. A scan pass is neither live MCP conformance evidence nor proof of public discovery/listing.

This is a bounded discovery slice after the working HTTPS/auth gateway and existing OBS/cloud priorities. It adds no new store target and does not block private deployments or clients that use an explicitly configured endpoint.

## Evidence and completion model

Track these independent dimensions per channel: eligibility; package/runtime validation; submission; approval; publication; and discovery. For each, record the exact channel app/offer ID, immutable source SHA, package version/hash, tool/auth snapshot, tested host/version, date, result, and evidence URL. An account connection, screenshot of a local plugin, or submitted form cannot mark all dimensions complete.

A completed channel requires a clean account or tenant to find the intended listing, install/connect it, authenticate to the correct SIP tenant, and complete an evidence-linked fixture lookup. Negative tests must reject another tenant's ID, stale/revoked grants, and unavailable capabilities. Later write tools also need server-enforced permissions, approval binding, idempotency, version conflicts, and atomic audit evidence.

A public review endpoint must expose only the declared integration surface. A publicly reachable server does not make service records public. Provide synthetic reviewer access through the platform's secure review channel, never a repository file. Apply current platform-specific reviewer-access requirements only to an isolated synthetic environment; do not weaken production authentication.

## Rollout and ownership

Keep existing OBS and cloud-deployment priorities. Do not enable a production integration until the runtime/auth/tenant-isolation prerequisites in #5 and the deployment work are satisfied. Prepare metadata and account-eligibility research in parallel; platform previews must not block the common gateway.

Suggested stages: reproducible gateway and reviewer environment; registry metadata plus one hosted-client pilot; ChatGPT/Claude/Cursor distribution; eligible Google preview; enterprise and curated channels when access is confirmed. Platform-specific drafts may proceed independently.

Submitting under an identity, accepting store terms, choosing a paid/free commercial offer, and publishing a release belong to the operator's release decision. This PR prepares reviewable artifacts; it does not authorize or execute those external steps.

## Acceptance criteria

- [ ] Planning inventory covers exactly the ten requested channels and distinguishes verified routes from unknown intake.
- [ ] No external count, local test, or worker checkpoint can mark a channel published.
- [ ] Registry metadata validates against a pinned current schema and references a real release/endpoint before submission.
- [ ] Optional domain discovery passes the catalog/card/live-connection checks above without exposing private metadata; legacy compatibility and scanner results are reported separately.
- [ ] Required publisher, support/legal, demo, authentication, package, and reviewer fields are checked per platform.
- [ ] Live multi-tenant/negative tests and clean-account discovery receipts are attached.
- [ ] Updates and removal are rehearsed; credential rotation and a denied or suspended listing have an owner.
- [ ] Documentation names the actual supported surfaces and limitations.
- [ ] Owner approves the exact release/submission payload before any public publication.

## Primary references

- [Official Registry quickstart and remote-only metadata](https://github.com/modelcontextprotocol/registry/blob/main/docs/modelcontextprotocol-io/quickstart.mdx)
- [Accepted Server Cards charter, SEP-2127](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/d0e246ea3c69eafaa2806bc4cd28025703ba2c2b/seps/2127-mcp-server-cards.md)
- [Server Card discovery specification, reviewed snapshot](https://github.com/modelcontextprotocol/ext-server-card/blob/526201bbc80231daa40ffcdecfc9da4e54e5dc93/docs/discovery.md)
- [Server Card schema, reviewed snapshot](https://github.com/modelcontextprotocol/ext-server-card/blob/526201bbc80231daa40ffcdecfc9da4e54e5dc93/schema.json)
- [Cloudflare scanner announcement and older card example](https://blog.cloudflare.com/agent-readiness/)
- [OpenAI submission](https://developers.openai.com/plugins/deploy/submission)
- [Claude directory portal](https://claude.com/resources/articles/build-plugins-for-claude)
- [Google Connected Apps](https://developers.google.com/connected-apps)
- [Meta's Connector Platform announcement](https://developers.meta.com/blog/meta-connect-recap/)
- [Grok custom connectors](https://docs.x.ai/grok/connectors)
- [Microsoft Marketplace publication](https://learn.microsoft.com/en-us/microsoft-365/copilot/extensibility/publish-plugin-publicly)
- [Agentforce MCP discovery](https://developer.salesforce.com/blogs/2026/10/developers-guide-to-the-winter-27-release)
- [AWS listing flow](https://docs.aws.amazon.com/marketplace/latest/userguide/listing-saas-ai-agents.html)
- [Cursor submission checklist](https://cursor.com/docs/reference/plugins)

[Secondary tracker supplied by the user](https://agentdiscoverability.com/track): useful for ecosystem observations, not platform eligibility or SIP release evidence.
