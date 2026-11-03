# Goose, Amp, Kiro, and Pi integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

Extend SIP service-data access beyond the first client wave without maintaining a separate business API per harness.

## Proposed packages and documented boundaries

| Client | Proposed integration | Boundary to verify |
| --- | --- | --- |
| Goose | MCP extension plus maintenance workflow/recipe | Current official extension docs describe MCP extensions; validate the exact Desktop/CLI build and auth flow |
| Amp | Focused skill/plugin with MCP wiring or a remote MCP definition | Upstream distinguishes local configuration from saved remote definitions; choose explicitly and verify its credential/requester identity |
| Kiro | MCP configuration with an optional maintenance Power/workflow | IDE, CLI, and Web have separate deployment and identity constraints; test each supported surface |
| Pi | Native Pi extension/package or an explicitly pinned MCP bridge | Upstream documents extensions, skills, and packages; verify MCP capability in the selected build rather than assuming a universal mcp.json loader |

The current Pi repository resolves to earendil-works/pi. Record the chosen upstream commit/package, not only an obsolete repository or package name.

## Implementation plan

1. Propose integrations/agents/goose/, amp/, kiro/, and pi/ with small host adapters and shared workflow source.
2. For MCP-capable hosts, connect the shared gateway using their documented setup and credentials facilities.
3. For Pi, inspect its current extension interfaces and choose a maintained bridge or a small typed SIP tool extension. Do not publish a pretend native MCP recipe. A direct adapter must preserve the gateway's authorization and schema contract.
4. On Amp, distinguish a thread owner's credential identity from the person initiating a shared-thread request. Do not add workload-identity trust or grant mapping solely because a token has a valid external signature.
5. Keep orchestration features optional. SIP does not need to invoke every harness or become an agent-to-agent scheduler to expose service tools.
6. Add install/uninstall and versioned conformance evidence for each adapter before public release.

## Acceptance checks before claiming support

- [ ] Each client has an exact version, transport/bridge, auth path, and successful staging read recorded.
- [ ] The shared two-tenant denial and revoked-grant fixtures pass per client.
- [ ] A shared/team session never lends an owner's grant to an unauthorized participant.
- [ ] Pi's selected extension/bridge is pinned, inspected, and tested; stdout/logs do not leak credentials or protocol frames.
- [ ] Repeated jobs cannot duplicate later authorized writes.
- [ ] Documentation distinguishes documented upstream capabilities, implemented SIP adapters, and untested targets.

## Additional discovery queue

After this wave, evaluate OpenHands, Aider, Augment, Google Antigravity, JetBrains Junie, Claude Desktop/Cowork, and existing Roo/Continue deployments. Confirm current maintenance status, actual plugin/MCP interfaces, licensing, and user demand. Unsupported clients need an explicitly reviewed CLI/extension bridge; a shared schema does not automatically make every AI tool compatible.

## Scope and dependencies

Proposal only. None of these packages or bridges is installed or published by this PR. Each may become its own implementation PR after the shared gateway is operational.

## Sources

- [Goose extension documentation](https://goose.vcorp.ai/docs/getting-started/using-extensions/)
- [Amp MCP and identity documentation](https://ampcode.com/docs/customize/mcp)
- [Kiro MCP documentation](https://kiro.dev/docs/mcp/)
- [Pi upstream README](https://github.com/earendil-works/pi/blob/main/packages/coding-agent/README.md)
