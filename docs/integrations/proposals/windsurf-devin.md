# Windsurf / Devin Desktop Cascade integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

Cascade users can retrieve SIP maintenance evidence and prepare service investigations without embedding production credentials in editor rules.

## Documented integration surface

The current Windsurf MCP documentation redirects to Devin Desktop Cascade documentation. That page documents stdio, Streamable HTTP, SSE, and OAuth support. Use the current supported setup flow while retaining a versioned recipe for existing Windsurf installations. Do not infer hosted Devin agent support from Desktop support.

## Implementation plan

1. Propose integrations/agents/windsurf-devin/ with current Cascade setup instructions and a clearly labeled legacy-client recipe.
2. Connect the shared authenticated SIP gateway. Prefer the transport supported by the selected host release; do not choose an SSE endpoint just because an old example uses it.
3. Add asset/work-order investigation guidance with stable record links and incomplete-data reporting.
4. Inspect the actual config schema and secret storage in the release being tested before generating installation examples.
5. Validate a cloud Devin environment separately if requested. Network reachability, identity, and repository-secret access are independent of Desktop configuration.

## Acceptance checks before claiming support

- [ ] Documentation names the exact product/build and configuration file used.
- [ ] Cascade discovers the gateway and reads a known synthetic record.
- [ ] OAuth reconnect and revoked-token behavior are tested.
- [ ] Cross-tenant IDs and missing permissions are denied.
- [ ] Disable/uninstall preserves unrelated MCP integrations.
- [ ] Hosted Devin remains unverified until an independent authenticated smoke test exists.

## Scope and dependencies

Design only. No editor is configured, and no hosted Devin support is claimed. Depends on the shared gateway. This proposal avoids creating another backend simply to support a renamed or versioned client.

## Sources

- [Current Cascade MCP documentation](https://docs.devin.ai/desktop/cascade/mcp)
- [Windsurf documentation entry and redirect](https://docs.windsurf.com/windsurf/cascade/mcp)
