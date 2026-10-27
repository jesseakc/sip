# GitHub Copilot and VS Code integration proposal

**Status:** Proposed design for review. No working integration is delivered by this document.

**Research date:** 2026-10-27. **SIP baseline:** `dcad211f1f7742e7810551dd428c3137e76c015d`.

## Product outcome

Developers and support staff using Copilot can retrieve authorized SIP service context while investigating an integration or preparing a maintenance handoff.

## Documented integration surface

GitHub documents MCP across Copilot CLI, IDE, app, and cloud-agent surfaces with different setup paths. VS Code's local MCP format uses servers and can live in .vscode/mcp.json; Copilot harness configuration is separate. Do not copy one file to every surface and label it supported.

## Implementation plan

1. Propose integrations/agents/github-copilot/ with distinct VS Code-local, Copilot CLI, and cloud-agent setup guides and templates.
2. Use the shared gateway's read-only tools first, with host-appropriate OAuth or injected secret storage.
3. Keep repository automation credentials separate from individual operator grants. A GitHub repository collaborator does not automatically have permission to read a SIP tenant.
4. For cloud/code-review use, verify actual supported tools, auth, network policy, and read-only requirements before enabling the endpoint.
5. Limit the tool catalog to maintenance context. Do not expose user administration, imports, or automatic work-order mutations to repository events.
6. Keep the app/other IDEs as separate matrix rows until each is exercised; one VS Code success is not a pass for all Copilot hosts.

## Acceptance checks before claiming support

- [ ] Each supported surface validates its own configuration.
- [ ] CLI and VS Code each call a real staging fixture tool; cloud support has an independent run or remains explicitly unverified.
- [ ] A fork/untrusted repository event cannot obtain SIP credentials.
- [ ] Read-only tooling is enforced on the server and reflected in truthful tool metadata.
- [ ] Credential revoke and organization-policy denial fail closed.
- [ ] An unrelated repository with no SIP grant cannot retrieve service data.

## Scope and dependencies

Design only. No repository secrets or Copilot configuration are changed. Depends on the shared gateway, deployment policy, and per-runtime validation.

## Sources

- [GitHub Copilot MCP surfaces](https://docs.github.com/en/copilot/concepts/customizations/mcp)
- [VS Code MCP configuration reference](https://code.visualstudio.com/docs/agents/reference/mcp-configuration)
- [Repository/cloud MCP configuration](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/configure-mcp-servers)
