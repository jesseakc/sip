# Claude Code integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

An operator working in Claude Code can inspect asset service context, explain an overdue work order, and prepare a reviewable maintenance change with source links. The package should work from an ordinary operator project, not require editing SIP source or cloning the whole server repository.

## Documented integration surface

Claude Code has native plugins. Its documented package layout includes .claude-plugin/plugin.json and plugin-root components. Local development can load a plugin with --plugin-dir, and upstream provides plugin validation. Use those mechanisms rather than a generic prompt advertised as a native plugin.

## Implementation plan

1. Propose integrations/agents/claude-code/ with .claude-plugin/plugin.json, host-appropriate MCP configuration, focused workflows, and setup/uninstall guidance.
2. Point the package at the shared authenticated SIP gateway. Offer a versioned stdio bridge only if needed for a supported deployment.
3. Keep reusable workflows portable while retaining a Claude-specific manifest adapter. Package components stay at the plugin root, not inside the metadata directory.
4. Add asset briefing, work-order explanation, and change-preparation workflows. Tool discovery supplies the real available tool names; no instruction should fabricate missing tools.
5. Start with local development loading. A repository marketplace and public marketplace submission are later distribution steps with separate review.

## Acceptance checks before claiming support

- [ ] Upstream manifest/component validation passes for the pinned Claude Code version.
- [ ] Fresh load, tool discovery, an authenticated fixture read, and uninstall are exercised.
- [ ] Reconnect/expired credentials behave correctly in interactive and supported noninteractive sessions.
- [ ] A second tenant's record is denied even when its ID appears in a prompt.
- [ ] An unavailable gateway produces an error instead of a claimed successful maintenance action.
- [ ] Evaluation examples distinguish evidence-based summaries from hallucinated records and unauthorized writes.

## Scope and dependencies

This PR is an implementation proposal, not a working plugin. Planned package files, commands, and credentials must be validated against the chosen upstream release before shipping. Depends on the shared gateway and security prerequisites; no public package is published by merging this document.

## Sources

- [Claude Code plugin overview](https://code.claude.com/docs/en/plugins)
- [Create and test a Claude Code plugin](https://code.claude.com/docs/en/plugins/create)
