# RFC: SIP Standalone Installer and Plugin Marketplace

Status: proposal only. This PR does not implement the installer.

## Goals
Deliver a standalone, independently released SIP installer that helps a nontechnical customer deploy, configure, verify, maintain and upgrade SIP without manually editing configuration files. Keep the installer separate from SIP Core, sharing only stable versioned APIs and plugin contracts.

## Three installation paths
1. **Prebuilt binaries:** detect OS, CPU and dependencies; download signed versioned artifacts; verify signatures and checksums; stage atomically; validate before activation.
2. **Build from source:** select a tagged release; verify provenance; pin toolchain and Cargo.lock; run reproducible locked Rust workspace builds for required crates and selected compile-time components; display toolchain, resource and native dependency checks.
3. **Containers:** generate pinned Docker Compose deployments with health checks, persistent volumes, networks, secrets, database options and reverse proxy/TLS. Consider Podman only after compatibility testing.

Do not assume that a dynamically installed plugin is interchangeable with a statically compiled Rust crate. Define explicit runtime kinds and compatibility rules.

## Proposed components
- `sip-installer`: independently downloadable Rust executable.
- `sip-install-engine`: deterministic plan/apply/resume/rollback orchestration and state journal.
- `sip-install-cli`: interactive terminal wizard and headless commands.
- `sip-install-web`: local browser wizard with loopback binding and short-lived authenticated setup session.
- `sip-plugin-contract`: versioned manifest and lifecycle interface.
- `sip-plugin-registry`: signed curated catalog and dependency resolver.
- Deployment adapters for binary, source and container modes.

SIP Core remains independent. Installation must work before SIP Core starts.

## Wizard flow
1. Welcome, deployment target, environment and prerequisite checks.
2. Choose binary, source or container deployment, with tradeoffs and estimated requirements.
3. Choose an installation profile or customize individual plugins.
4. Search/select plugins; show dependencies, permissions, conflicts, versions and support status.
5. Configure database, TLS, backups and credentials; run connection tests.
6. Connect the existing SIP database migrator through a versioned interface; preview source data, map fields, dry-run, import with checkpoints and reconcile.
7. Configure AI model providers separately from AI agent harnesses; select multiple connectors, enter keys securely or use OAuth, generate client-specific configurations and test connections.
8. Configure UI hostname, HTTPS, branding, ports, administrator bootstrap, email, MFA and SSO where supported.
9. Review a sanitized deterministic plan and explicitly approve changes.
10. Execute with progress, redacted logs, retry, resume and safe cancellation.
11. Verify application, schema, authentication, plugins, connectors, TLS and backup readiness.
12. Open the UI and provide a sanitized diagnostic report.

Use progressive disclosure, contextual help, keyboard accessibility, resumable sessions and a shared engine across browser, terminal and headless modes. The wizard should also manage installed SIP instances after initial setup.

## Plugin marketplace design
Define a machine-validated signed manifest containing: id, publisher, version, license, core API compatibility, supported OS/deployment methods, runtime kind, artifact digest/signature, dependencies, conflicts, capabilities, permissions, configuration and secret schemas, migrations, health checks, install/upgrade/uninstall lifecycle, and support status.

Resolve dependencies deterministically, detect cycles, pin versions, enforce trust and compatibility, and display transitive permissions before approval. Support install, enable, disable, update, rollback and uninstall, with data retention choices. Initial marketplace is curated and signed; later phases can add third-party publishing, review, revocation, advisories and offline bundles. Never execute unverified plugin code or assume an ABI/sandbox that has not been implemented.

## Connector roadmap already proposed in GitHub
Catalog entries must be **planned**, not available, until the underlying connector is merged and verified:
- #5: shared MCP gateway and client contract, prerequisite.
- #6 OpenAI Dots; #7 Grok Bot.
- #8 Claude Code; #9 Hermes; #10 OpenClaw; #11 Codex and ChatGPT; #12 OpenCode.
- #13 Cursor; #14 Gemini CLI; #15 GitHub Copilot CLI, VS Code and cloud agents.
- #16 Windsurf/Devin Desktop; #17 Cline and Kilo Code, with separately validated legacy compatibility.
- #18 Goose, Amp, Kiro and Pi.

Every connector must declare auth method, local/cloud transport, supported versions, configuration targets, required scopes, tenant boundary, tool permissions, health test and rollback. Distinguish model provider APIs from agent harnesses and ChatGPT application integrations. Default agent tools to read-only until explicitly authorized.

## Security requirements
Signed release artifacts and catalog; pinned trust roots and revocation; secure secret references rather than plaintext config; OAuth PKCE/device flow where available; API key masking, validation and rotation; TLS validation; least-privilege installer operations; explicit network exposure; first-admin secure bootstrap; optional MFA/SSO; tenant isolation and audit logs. The browser wizard binds to loopback by default, uses a short-lived setup token, origin checks and CSRF protections. Never leak secrets in logs, diagnostics, command arguments or Git.

## Migration and reliability
Never reset an existing database by default. Check compatibility, privileges, storage and TLS. Classify migrations by reversibility, require backup for risky operations, support dry runs, duplicate resolution, field mapping, idempotency, per-batch checkpoints and reconciliation. Track install states: discovered, preflighted, planned, approved, staging, configuring, migrating, activating, verifying, complete or recoverable failure. Do not automatically roll back imported customer data.

## Agent automation
Expose schema-validated commands such as:
```sh
sip-installer plan --config install.yaml --output plan.json
sip-installer apply --plan plan.json --approve-plan-hash HASH
sip-installer resume --run-id RUN_ID
sip-installer doctor --json
```
Use secret references, stable machine-readable errors, explicit approval policy and bounded unattended permissions. A blanket approval must not authorize destructive migrations.

## Implementation phases
- M0: architecture decisions, threat model, compatibility matrix and manifest schema.
- M1: standalone engine, binary installation, preflight, journal, CLI and diagnostics.
- M2: source and container adapters with release parity.
- M3: accessible browser wizard and recovery.
- M4: signed catalog, resolver and plugin lifecycle.
- M5: database migrator onboarding and validation.
- M6: AI harness onboarding progressively as PRs #5-#18 become functional.
- M7: upgrades, offline installation, marketplace governance and advanced operations.

## Acceptance criteria
- Fresh supported hosts can install via all three paths and reach a verified, authenticated SIP UI.
- No-AI, single-agent and multi-agent installation profiles work without misleading support claims.
- Unsupported, unmerged or unverified plugins are clearly marked planned.
- Failed downloads, invalid signatures, dependency conflicts, interrupted migrations and restarts fail safely and can recover.
- Secret leakage, CSRF, insecure exposure, tenant crossing and privilege escalation tests pass.
- Plugin install/upgrade/removal and database backup/restore scenarios are exercised.
- Automated tests cover manifest/schema validation, dependency resolution, OS/deployment matrix, dry-run determinism, recovery and accessibility.
- Documentation includes novice install guides, headless setup, security, upgrades and troubleshooting.

**Definition of done:** this RFC can be accepted as design documentation. Actual implementation requires separate code, tests, release artifacts and verified deployment evidence. Do not mark the feature implemented merely because this proposal is merged.
