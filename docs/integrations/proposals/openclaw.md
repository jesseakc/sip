# OpenClaw integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

An OpenClaw agent can inspect maintenance context and prepare a service handoff from a configured operator identity. Gateway/channel availability is separate from permission to modify SIP records.

## Documented integration surface

OpenClaw documents both native runtime plugins and compatible Agent Plugins/Codex/Claude/Cursor bundles. Native plugins use openclaw.plugin.json plus a runtime module. Use the smallest compatible MCP/skills bundle first and test its supported subset on a pinned release.

## Implementation plan

1. Propose integrations/agents/openclaw/ containing the compatible package, SIP maintenance guidance, and setup/disable/uninstall instructions.
2. Connect the shared authenticated SIP gateway and filter the exposed tool set. Explicitly verify that bundle MCP tools load on the target OpenClaw version.
3. Add a native TypeScript plugin only if the bundle cannot expose required tools. In that case use the upstream manifest/runtime interfaces, a narrow configuration schema, and the same gateway contract.
4. Document plugin activation and tool permission policy separately. An enabled package must not imply that all registered tools are callable.
5. Keep credentials and tenant bindings outside chat-visible configuration. Bind channel senders through an explicit operator/grant mapping; never use a sender-provided organization string as authority.
6. Exclude automatic hook-triggered business writes from the first package.

## Acceptance checks before claiming support

- [ ] The pinned bundle format validates and the running Gateway can call a real SIP fixture tool.
- [ ] Cold plugin inspection and actual runtime capability tests are both recorded.
- [ ] Tool allow/deny settings behave as documented; disabled tools fail closed.
- [ ] Two channel users cannot reuse each other's service grants.
- [ ] Disabling or uninstalling the package removes its capabilities without altering unrelated plugins.
- [ ] A gateway restart or retried event does not duplicate a later authorized write.

## Scope and dependencies

Design only; no OpenClaw Gateway is configured by this PR. Compatibility support is an upstream capability, not proof this SIP package works. Depends on the shared gateway and client smoke tests.

## Sources

- [OpenClaw plugin formats and runtime verification](https://docs.openclaw.ai/tools/plugin)
