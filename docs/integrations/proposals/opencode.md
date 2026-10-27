# OpenCode integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

An OpenCode user can inspect asset/work-order data and prepare a maintenance action while retaining normal local project configuration and credential boundaries.

## Documented integration surface

OpenCode documents local and remote MCP servers. Current V2 documentation puts named servers under mcp.servers; V1 documentation uses a different configuration shape. V2 remote connections document OAuth discovery/PKCE. A native OpenCode plugin is an optional convenience layer, not a substitute for the SIP gateway.

## Implementation plan

1. Propose integrations/agents/opencode/ with separately versioned V1 and V2 setup recipes, not one ambiguous opencode.json example.
2. Start with the native MCP connection flow and scoped OAuth. Use explicit secret-reference syntax supported by the selected release if a staging bearer-token fallback is needed.
3. Add a small native plugin only after a concrete need is identified, such as presenting setup status. Evaluate its supported configuration-transform API against the pinned version.
4. Reuse the common tools and maintenance workflow text; do not intercept arbitrary shell commands or all model traffic.
5. Detect server-name collisions and document project-versus-user precedence. Never overwrite an existing SIP connection for a different tenant.

## Acceptance checks before claiming support

- [ ] Each supported major version accepts its own config; a V1 file is never labeled V2-compatible without validation.
- [ ] Fresh OAuth sign-in, tool discovery, refresh/reconnect, and revoke work on staging.
- [ ] The client can retrieve a known fixture and reports denied or incomplete data.
- [ ] Optional plugin enable/disable leaves unrelated settings and tools intact.
- [ ] Package upgrade detects unsupported upstream API/configuration changes.

## Scope and dependencies

Design only. No OpenCode configuration is installed by this PR. Exact supported versions and optional plugin APIs remain to be selected during the compatibility spike. Depends on the shared gateway and authorization tests.

## Sources

- [OpenCode V2 MCP configuration](https://opencode.ai/v2/docs/mcp-servers)
- [OpenCode V1 MCP documentation](https://docs.opencode.ai/docs/mcp-servers/)
- [OpenCode V2 plugin API](https://opencode.ai/v2/docs/build/plugins)
