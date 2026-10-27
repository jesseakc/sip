# OpenAI Dots integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

A dot can answer which assets need attention, assemble a service briefing with linked work orders, and prepare a change for an operator to review. SIP remains the record of work; a conversation summary is not evidence that a work order was changed.

## Documented integration surface

Official OpenAI documentation says dots can use supported installed plugins and their existing account permissions. Plan a remote plugin connection to the shared SIP MCP gateway. This is not an assumed public Dots control API. Local skills need the appropriate connected-computer setup; the primary hosted proposal should not depend on a laptop being online.

## Implementation plan

1. Package a remote SIP plugin with portable plugin.json, mcp.json, and focused maintenance workflows. Proposed future package directory: integrations/agents/openai-dots/. Treat all paths as planned.
2. Add onboarding that selects the SIP deployment, authenticates, confirms the authorized tenant, and shows the initial read-only tool set.
3. Add asset briefing and overdue-work-order workflows using source IDs, observed timestamps, canonical statuses, and explicit incomplete-data reporting.
4. Add setup/reconnect/revoke guidance. Sharing a plugin or a dot must never distribute one operator's credentials.
5. Add event support only in a later change after SIP's outbox actually delivers events. The current outbox poller is not a working event delivery system.

OpenAI currently documents MCP Events for dots with the 2026-07-28 protocol profile and webhook delivery. Isolate this from the baseline client tool contract. Proposed events such as work_order.created or work_order.status_changed are not implemented SIP endpoints. A later event adapter needs persisted subscriptions, tenant filters, stable event IDs, bounded retries, callback verification, signing, duplicate handling, and unsubscribe/revocation tests. A generic webhook is not automatically MCP Events compliant.

## Acceptance checks before claiming support

- [ ] A fresh eligible account can connect the test plugin to an HTTPS staging gateway and identify its authorized tenant.
- [ ] A briefing cites actual synthetic fixture records and reports missing history without filling gaps.
- [ ] Local-only deployment failure gives actionable network guidance.
- [ ] Disconnect and token revocation prevent subsequent reads and event delivery.
- [ ] Events, when added, deliver only matching authorized records; a retry does not duplicate business actions.
- [ ] Any write uses the shared server-side authorization and reviewed-change flow, never just an autonomous instruction in dot memory.

## Scope and dependencies

This PR proposes the design only. The MCP gateway, plugin package, event methods, and a live Dots smoke test remain future work. Availability and organization policy can vary; verify the target account at implementation time. Depends on the shared foundation and its security gates.

## Sources

- [Meet dots](https://learn.chatgpt.com/docs/dots)
- [Plugin packaging](https://developers.openai.com/plugins/build/plugins)
- [MCP Events](https://developers.openai.com/plugins/build/mcp-events)
