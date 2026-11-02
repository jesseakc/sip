# Gemini CLI integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

A Gemini CLI user can request an evidence-backed asset summary, inspect work-order status, and prepare an operator-reviewed change.

## Documented integration surface

Gemini CLI extensions package context/workflows and MCP server definitions. Use gemini-extension.json and the release's supported extension mechanism rather than copying another harness's plugin manifest.

## Implementation plan

1. Propose integrations/agents/gemini-cli/ with gemini-extension.json, scoped maintenance context/workflows, and install/link/uninstall instructions.
2. Point to the shared gateway with a supported remote auth flow, or a versioned stdio bridge if required by the target release.
3. Keep sensitive deployment values in supported settings/secret storage. Do not ship a live credential in extension context or a public configuration file.
4. Document settings-versus-extension precedence so an operator's existing server definition is not silently replaced.
5. Add a read-only asset briefing and work-order lookup; stage write workflows behind the shared server authorization design.
6. Record CLI and extension versions separately and test from a clean profile before distributing an extension source URL.

## Acceptance checks before claiming support

- [ ] The extension manifest and all referenced components validate on the pinned release.
- [ ] Local development linking and a fresh installation discover the same SIP tool contract.
- [ ] One fixture lookup, a denied tenant lookup, and an unavailable gateway are tested.
- [ ] An existing local override remains effective and does not cause duplicate server/tool registration.
- [ ] Upgrade/uninstall do not expose credentials or alter unrelated extensions.
- [ ] Output reports source IDs, observed timestamps, and incomplete data correctly.

## Scope and dependencies

Design only. No Gemini extension is installed or published. Depends on the shared gateway and an implementation-time check of extension/authentication schemas.

## Sources

- [Gemini CLI extensions](https://geminicli.com/docs/extensions/)
- [Extension manifest reference](https://geminicli.com/docs/extensions/reference/)
- [MCP configuration](https://geminicli.com/docs/tools/mcp-server/)
