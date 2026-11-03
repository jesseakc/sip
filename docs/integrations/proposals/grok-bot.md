# xAI Grok Bot integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

Give a Grok Bot a focused maintenance-coordinator job: investigate an asset, prepare a cited handoff, and draft a proposed work-order update. Keep model inference in SIP and connecting an external Grok Bot to SIP as separate integration directions.

## Documented integration surface

xAI's Grok Bot guide documents compatibility with Cursor MCP servers, plugins, and skills. Team connector policy is inherited from Cursor. Reuse the SIP gateway and a Cursor-compatible package; do not invent a Grok Bot SDK or equate this with calling a Grok model API.

## Implementation plan

1. Propose integrations/agents/grok-bot/ with connector metadata, deployment/setup instructions, maintenance workflows, and synthetic acceptance fixtures.
2. Connect an authenticated remote SIP MCP server through the supported custom-connector/plugin flow. Confirm the actual client schema and versions during the implementation spike.
3. Keep the core tools identical to the Cursor package. Supply Bot-specific onboarding and durable-job guidance without copying token values or case records into public templates.
4. Support a personal Bot first, then validate Team Bots with two users whose SIP grants differ. The requesting person's SIP authorization must determine access.
5. Treat scheduling/routines as a separate opt-in setup. Do not create routines or connect external accounts merely by installing a package.
6. Return the resulting SIP record URL, version, and action outcome; stop at a prepared change until a server-validated apply flow exists.

## Acceptance checks before claiming support

- [ ] A fresh Bot can authenticate to staging and call a real read-only SIP tool.
- [ ] Two Team Bot users see only records allowed by their own grants.
- [ ] A blocked team connector produces an actionable setup message.
- [ ] The cloud runtime can reach the configured HTTPS endpoint; a private/local deployment is not silently assumed reachable.
- [ ] Reconnection and revocation are exercised; shared templates contain no credentials.
- [ ] Imported asset notes cannot expand tools, tenant selection, or write authority.
- [ ] A routine restart or repeated instruction cannot duplicate a future authorized write.

## Scope and dependencies

Design proposal only. No connector is installed or published, no Bot is created, and no external task is scheduled by this PR. Gateway implementation and account-specific Grok Bot testing remain outstanding. Reuse transport/business logic with Cursor while tracking their runtime tests separately.

## Sources

- [Grok Bot 101](https://x.ai/bot/guides/grok-bot-101)
- [Grok Bot team connector policy](https://docs.x.ai/grok-bot/teams-and-enterprises)
- [Team Bots and requester access](https://docs.x.ai/grok-bot/team-bots)
- [Cursor MCP documentation](https://cursor.com/docs/mcp)
