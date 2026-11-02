# SIP connector distribution proposals

Checked 2026-11-02 against main at `c3ad101ab5b60ca6cce3d23277bf8ce7a37c35ca`.
These are implementation proposals, not evidence of a deployed integration, submitted application, approved listing, or public availability.

## Review queue

| Proposal | Scope | Draft PR |
| --- | --- | --- |
| DIST-001 | Shared evidence model, Official MCP Registry, and ten-channel inventory | [#83](https://github.com/jesseakc/sip/pull/83) |
| DIST-002 | Google Connected Apps for Gemini and Search | [#84](https://github.com/jesseakc/sip/pull/84) |
| DIST-003 | Microsoft 365 Copilot Agent Store | [#85](https://github.com/jesseakc/sip/pull/85) |
| DIST-004 | AWS Marketplace and Salesforce AgentExchange | [#86](https://github.com/jesseakc/sip/pull/86) |
| DIST-005 | Meta Muse and Grok catalog eligibility | [#87](https://github.com/jesseakc/sip/pull/87) |

Each PR is independently based on main. Read its proposal from that PR's Files changed view until it is merged. [Readiness](readiness.md) and [targets.json](targets.json) define the common inventory and evidence requirements.

## Relationship to existing integrations

| Channel or surface | Existing work | Distribution work |
| --- | --- | --- |
| Official MCP Registry | Shared gateway [#5](https://github.com/jesseakc/sip/pull/5) | #83 |
| ChatGPT and OpenAI Dots | [#11](https://github.com/jesseakc/sip/pull/11), [#6](https://github.com/jesseakc/sip/pull/6) | Submission guidance on existing PRs; #83 readiness |
| Claude | Claude Code [#8](https://github.com/jesseakc/sip/pull/8) | Directory submission guidance on #8; #83 readiness |
| Google Connected Apps | Gemini CLI [#14](https://github.com/jesseakc/sip/pull/14) covers a different surface | #84 |
| Meta Muse | No confirmed SIP publisher access | #87 eligibility research |
| Grok catalog | Grok Bot [#7](https://github.com/jesseakc/sip/pull/7) covers a different surface | #87 custom-connector pilot and catalog intake research |
| Microsoft 365 | GitHub Copilot [#15](https://github.com/jesseakc/sip/pull/15) covers a different surface | #85 |
| Salesforce AgentExchange | Shared gateway #5 | #86; confirm the applicable partner intake |
| AWS Marketplace | Shared gateway #5 | #86 |
| Cursor Marketplace | [#13](https://github.com/jesseakc/sip/pull/13) | Submission guidance on #13; #83 readiness |

The internal SIP plugin marketplace [#38](https://github.com/jesseakc/sip/pull/38) remains a separate product feature. It does not publish SIP to third-party stores.

## Execution order and completion

Preserve the existing observability and cloud-deployment priorities. Metadata and eligibility research can proceed independently; runtime pilots require a deployed, authenticated gateway with tenant isolation. Do not treat fake-server tests or a locally loaded plugin as public distribution evidence.

Track eligibility, runtime validation, submission, approval, publication, and clean-account discovery separately. Record the exact release, tool/auth snapshot, platform ID, date, and evidence for each transition. The inventory deliberately starts with runtime and publication status `not_verified` and no invented endpoint, publishing identity, or listing URL.

The Registry is an early metadata destination once a release is ready. It does not automatically submit to other stores. Aggregator ingestion policies need independent confirmation.

No marketplace application, commercial choice, term acceptance, or publication was performed by these PRs. Prepare the exact release packet for operator review before those actions. Keep uncertain access explicit, especially for preview or curated programs.
