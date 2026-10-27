# SIP AI integration proposals

These are design proposals for review, researched on 2026-10-27 against SIP baseline `dcad211f1f7742e7810551dd428c3137e76c015d`. **No client package or working MCP gateway is delivered by these RFCs.** Documented upstream capability is not tested SIP compatibility.

| Proposal | Draft PR | Proposed implementation |
| --- | --- | --- |
| Shared SIP MCP foundation | [#5](https://github.com/jesseakc/sip/pull/5) | Gateway, authorization, shared tools, conformance tests |
| OpenAI Dots | [#6](https://github.com/jesseakc/sip/pull/6) | Hosted plugin; event subscriptions in a later phase |
| xAI Grok Bot | [#7](https://github.com/jesseakc/sip/pull/7) | Cursor-compatible connector and Bot workflows |
| Claude Code | [#8](https://github.com/jesseakc/sip/pull/8) | Native Claude Code plugin and MCP wiring |
| Hermes Agent | [#9](https://github.com/jesseakc/sip/pull/9) | Portable plugin; native hooks only if needed |
| OpenClaw | [#10](https://github.com/jesseakc/sip/pull/10) | Compatible bundle; optional native runtime adapter |
| OpenAI Codex | [#11](https://github.com/jesseakc/sip/pull/11) | Portable plugin and Codex compatibility recipes |
| OpenCode | [#12](https://github.com/jesseakc/sip/pull/12) | Separate V1/V2 MCP recipes; optional plugin |
| Cursor | [#13](https://github.com/jesseakc/sip/pull/13) | Plugin/MCP setup with separate local/cloud checks |
| Gemini CLI | [#14](https://github.com/jesseakc/sip/pull/14) | Native Gemini CLI extension |
| GitHub Copilot and VS Code | [#15](https://github.com/jesseakc/sip/pull/15) | Separate CLI, VS Code, and cloud-agent recipes |
| Windsurf / Devin Desktop Cascade | [#16](https://github.com/jesseakc/sip/pull/16) | Versioned Cascade setup; cloud Devin checked separately |
| Cline and Kilo Code | [#17](https://github.com/jesseakc/sip/pull/17) | Independent Cline and Kilo recipes |
| Goose, Amp, Kiro, and Pi | [#18](https://github.com/jesseakc/sip/pull/18) | Goose, Amp, Kiro, and Pi adapters |

## Review and implementation order

1. Review the [shared foundation design](proposals/shared-mcp-foundation.md). It defines the service tool contract, tenant-bound authorization, deployment boundary, and common test suite.
2. Resolve the existing migration, authentication, tenant-isolation, and audit prerequisites before enabling external agents against real data. Related fixes are tracked in [PR #1](https://github.com/jesseakc/sip/pull/1) and [PR #3](https://github.com/jesseakc/sip/pull/3); those PRs do not cover every tenant-isolation finding.
3. Implement one local client and one hosted client against the same synthetic two-tenant fixture before expanding the package set.
4. Add client-specific packaging and onboarding. Reuse service logic and workflow source, while validating each host's manifests, credentials, permissions, and transport separately.
5. Consider controlled writes and events only after the read-only connection and server-enforced authorization are proven.

The documentation PRs can merge independently in any order. Runtime packages depend on the foundation; merging a proposal does not approve a deployment, register credentials, or create recurring tasks.

## Definition of supported

Keep a compatibility record containing client name/version, SIP gateway version, transport and negotiated protocol profile, authorization method, test date, fixture/evidence link, and outcome. Until it contains an actual successful tool call and negative authorization tests, label a client **planned**, not supported.

Require fixture read/source-link checks, incomplete-data reporting, cross-tenant denial, revoked-grant denial, reconnect behavior, secret redaction, and clean uninstall per host. Native plugin loading alone does not prove an integration works. For later business writes, add permission, approval binding, idempotency, version-conflict, and audit-atomicity checks.

## Further candidates

The shared design also records a discovery queue for Claude Desktop/Cowork, OpenHands, Aider, Augment, Google Antigravity, JetBrains Junie, and existing Roo Code/Continue installations. Confirm current upstream maintenance and actual extension interfaces before promising a package. This list is a candidate inventory, not a popularity ranking or an exhaustive compatibility claim.

Each proposal links its primary upstream documentation and distinguishes design choices from documented product capabilities.
