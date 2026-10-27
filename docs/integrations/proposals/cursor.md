# Cursor integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

A Cursor user can assemble a maintenance investigation from SIP records and prepare a proposed change while developing an integration or supporting an asset.

## Documented integration surface

Cursor documents custom MCP configuration in .cursor/mcp.json or a user-level mcp.json, plus marketplace/plugin setup. Its MCP page documents remote OAuth and environment interpolation. The Grok Bot package may reuse this connector shape, but compatibility must be proven in both runtimes.

## Implementation plan

1. Propose integrations/agents/cursor/ with plugin metadata, MCP configuration recipes, focused maintenance guidance, and an operator setup checklist.
2. Use an authenticated remote gateway for hosted environments; support a versioned stdio bridge when local deployment requires it.
3. Make project-scoped versus user-scoped installation explicit. Detect a preexisting server definition and preserve it rather than replacing it.
4. Bundle only the shared SIP read tools initially. Add a small workflow that traces an asset question to linked work orders and separates observations from suggestions.
5. Keep actual service credentials out of .cursor rules, plugin source, and committed configuration.
6. Evaluate marketplace packaging and an installation link only after the package is operational. No claim of an official marketplace listing is made here.

## Acceptance checks before claiming support

- [ ] The chosen package and configuration validate on a pinned Cursor version.
- [ ] Local and intended cloud runtimes can reach the gateway with the documented auth flow.
- [ ] A real fixture read proves tool loading; the answer links to the fixture record.
- [ ] Project overrides and two deployments do not cross-wire credentials or tenant context.
- [ ] Disconnect/revoke and uninstall remove access without touching unrelated MCP entries.
- [ ] Grok Bot runs its own smoke suite rather than inheriting Cursor's pass status.

## Scope and dependencies

Proposal only, with no Cursor installation or public listing. Depends on the shared gateway and tenant isolation fixes.

## Sources

- [Cursor MCP setup, OAuth, and config locations](https://cursor.com/docs/mcp)
- [xAI's Cursor compatibility statement](https://x.ai/bot/guides/grok-bot-101)
