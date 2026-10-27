# Cline and Kilo Code integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

Cline and Kilo users can query authorized SIP service records and prepare a maintenance handoff from their existing editor or CLI.

## Documented integration surface

Cline documents local and remote MCP setup through its MCP settings. Kilo documents its own configuration locations and client-specific options. Related ancestry is not evidence that configuration files, authentication, or plugin APIs are interchangeable.

## Implementation plan

1. Propose integrations/agents/cline/ and integrations/agents/kilo/ as separate future package/recipe directories sharing the SIP tool contract.
2. Prefer native MCP configuration first. A custom editor extension or hook layer must solve a demonstrated problem rather than duplicate all tools.
3. Provide host-specific setup, secret references, enable/disable, and uninstall instructions after validating the selected IDE/CLI release.
4. Add maintenance briefing and record lookup workflows with explicit sources; keep business writes out of automatic approvals.
5. Test CLI and IDE settings separately. Read existing configuration and preserve unrelated server definitions.
6. Track Roo Code and Continue as compatibility/discovery candidates for existing users. Confirm their current upstream maintenance/distribution status and supported releases before offering a new native package; this proposal makes no active-support claim for either.

## Acceptance checks before claiming support

- [ ] Cline and Kilo each load their own configuration on pinned versions.
- [ ] Each calls a real SIP fixture tool and handles an unavailable gateway visibly.
- [ ] Auth reconnect/revoke, cross-tenant denial, and missing permissions pass per host.
- [ ] Installing one package never modifies the other client's settings.
- [ ] Disable/uninstall leaves unrelated MCP servers intact.
- [ ] Any legacy recipe is labeled with the exact tested client/version and maintenance caveats.

## Scope and dependencies

Design only. No Cline/Kilo configuration is installed. Both depend on the shared gateway; neither is marked runtime-tested merely because upstream documents MCP.

## Sources

- [Cline MCP setup](https://docs.cline.bot/mcp/mcp-overview)
- [Kilo MCP setup](https://kilo.ai/docs/automate/mcp/using-in-kilo-code)
