# OpenAI Codex integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

Connect an operator's Codex workflow to SIP service records with consistent tools and evidence links. Reuse the remote connector for supported ChatGPT surfaces while testing the behavior of each surface independently.

## Documented integration surface

Official OpenAI documentation recommends a portable root plugin.json with mcp.json and skills/. A .codex-plugin/plugin.json may provide compatibility metadata. Codex hosts also document direct MCP configuration; native plugin support and direct MCP support are distinct capabilities. Account, host, and directory availability must be checked rather than assumed identical everywhere.

## Implementation plan

1. Propose integrations/agents/codex/ with a portable manifest, remote MCP mapping, and maintenance workflows; add a compatibility overlay only when needed by supported releases.
2. Prefer a hosted HTTPS gateway for cloud clients. Supply a documented stdio bridge configuration for local hosts that need it.
3. Add setup that chooses a SIP deployment, establishes a tenant-bound connection, and demonstrates one real read-only call.
4. Keep workflows independent of SIP's repository coding instructions. An operator should not need to load maintenance data into AGENTS.md or share a browser token.
5. Reuse core workflow text with the Dots proposal; track Codex CLI, desktop, direct-MCP/IDE, and hosted-plugin smoke tests as separate rows.
6. Keep native lifecycle hooks and public directory submission outside the initial package.

## Acceptance checks before claiming support

- [ ] Portable/compatibility manifests validate on the pinned supported host versions.
- [ ] At least one local Codex host and one intended hosted surface complete an authenticated read.
- [ ] Tool results cite the actual SIP fixture IDs and source URLs.
- [ ] Cloud inability to reach a private endpoint is explained; localhost is never silently substituted.
- [ ] Credential revocation, reconnect, and tenant isolation pass.
- [ ] Unsupported surfaces are explicitly listed; package installation alone is not recorded as a successful tool test.

## Scope and dependencies

Design proposal only. No plugin is installed, submitted, or published, and no operational SIP MCP endpoint exists yet. Depends on the shared foundation and its security gates.

## Sources

- [OpenAI plugin packaging](https://developers.openai.com/plugins/build/plugins)
- [OpenAI MCP client capabilities](https://learn.chatgpt.com/docs/extend/mcp)
- [Plugin availability across surfaces](https://learn.chatgpt.com/docs/plugins)
