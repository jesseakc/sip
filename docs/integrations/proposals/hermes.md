# Hermes Agent integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

Let a self-hosted Hermes operator retrieve service evidence, create a shift briefing, and prepare a maintenance change from CLI or a messaging conversation. Messaging identity alone must not determine SIP authorization.

## Documented integration surface

Hermes documents local stdio and remote HTTP MCP clients and a portable Agent Plugins v1 adapter. Native Hermes plugins use plugin.yaml plus register(ctx), which is a different extension surface. Begin with a portable MCP/workflow package; introduce Python hooks only for a demonstrated gap.

## Implementation plan

1. Propose integrations/agents/hermes/ with portable plugin.json, mcp.json, maintenance workflows, and a Hermes setup guide.
2. Use Hermes' configured MCP connection and discovery; do not create a parallel credential store or a second raw HTTP client inside a skill.
3. Document the mcp_servers config mapping for a manual setup fallback, with secret references supplied through the operator's secret facility.
4. Add an optional follow-up proposal for a thin native plugin only if gateway interactions need Hermes-specific lifecycle integration. Native hook installation is not part of the first package.
5. Keep memory minimal: persist useful workflow preferences, not unrestricted service histories or credentials. Recheck live permissions when a conversation resumes.
6. Validate CLI and one operator-selected messaging surface separately. A shared gateway process must not lend one SIP user's grant to another sender.

## Acceptance checks before claiming support

- [ ] Portable package validation and linked/fresh installation pass on a pinned Hermes release.
- [ ] MCP discovery and read-only fixture tools work after restart.
- [ ] Gateway reconnect, cancellation, timeout, and denied tools are visible to the operator.
- [ ] Different sender/tenant grants remain separated; unbound messaging senders fail closed.
- [ ] A saved workflow never expands its grant through a prompt or memory.
- [ ] Install, upgrade, disable, and uninstall preserve unrelated Hermes config and redact secrets.

## Scope and dependencies

Design only. No Hermes configuration is changed by this PR, and SIP has no published Hermes package yet. A native plugin is optional, not a requirement for using the shared MCP contract. Depends on the shared gateway and its authorization tests.

## Sources

- [Hermes MCP client](https://hermes-agent.nousresearch.com/docs/user-guide/features/mcp)
- [Hermes native and portable plugin formats](https://hermes-agent.nousresearch.com/docs/developer-guide/plugins)
