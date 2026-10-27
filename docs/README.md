# SIP documentation

SIP helps service teams manage equipment, maintenance work, and operational
knowledge. Start with the guide for what you want to do.

## Get started

- [Quick Start](../QUICKSTART.md): run locally, create your administrator, and import data.
- [Vercel deployment](deployment/vercel.md): host the frontend on Vercel and the API separately.
- [Architecture](architecture.md): current runtime behavior, diagrams, and implementation gaps.

## Use and extend SIP

- [Migration overview](migrations/overview.md): supported imports, dependency order, and rollback limits.
- [Migration Studio](../plugins/sip-migration-studio/README.md): the import wizard and examples.
- [Migration API](migrations/api-reference.md) and [external ID mapping](migrations/external-id-mapping.md).
- [API specification](openapi.yaml): endpoint definitions.
- [Platform reference](platform-reference.md): capabilities, endpoint examples, entities, roles, and database structure.
- [Development](development.md): local development, configuration, build options, and checks.

## Build plugins

- [Plugin overview](plugins/overview.md): architecture, plugin types, and lifecycle.
- [Manifest reference](plugins/manifest-reference.md): the `plugin.toml` contract.
- [Plugin API](plugins/api-reference.md): discovery and registration endpoints.
- [Host a plugin UI](plugin-hosting.md): mounts, iframe hosting, and authentication.
- [UI plugins](plugins/ui-plugins.md) and [navigation](plugins/navigation.md).
- [Extension points](plugins/extension-points.md): where plugins contribute to SIP.
- [Theme contract](plugins/theme-contract.md) and [native UI checklist](plugins/sip-native-ui-checklist.md).
- [Examples](plugins/examples.md): plugin walkthroughs and sample manifests.

## Understand the AI design

SIPmem is the memory and evidence engine being developed for SIP. These pages
describe its design; the [architecture guide](architecture.md) explains which
parts are connected to the running application.

- [SIPmem overview](sipmem/overview.md): memory types, facts, and retrieval recipes.
- [Architecture](sipmem/architecture.md): pipeline and component design.
- [Verification loop](sipmem/verification-loop.md): checking claims against evidence.
- [Temporal memory](sipmem/temporal-memory.md): facts and changes over time.
- [Retrieval recipes](sipmem/retrieval-recipes.md): how a query selects evidence.
- [Plugin contracts](sipmem/plugin-contracts.md): custom retrievers and verifiers.

## Test and contribute

- [Development checks](development.md#checks): backend and frontend commands.
- [Manual testing guide](testing-weekend.md): workflow scenarios. Its demo-login instructions predate empty-database setup; use your own accounts.
- [Test Bench plugin](../plugins/sip-test-bench/README.md): test harness and trace inspection.
- [Maintenance-log dataset tools](../data/smrt-maintenance-logs/README.md): optional local test data preparation.
- [Demo tooling](../scripts/demo/README.md): historical demo setup. It assumes the removed Acme seed account and needs adaptation for a fresh installation.
- [Database migrations](../migrations/README.md): migration workflow and conventions.

## Product and planning

- [Product vision](product-vision.md): purpose, intended users, and future direction.
- [Progress tracker](PROGRESS.md): implementation tracking.
- [Product requirements](PRD-v2.md), [earlier requirements](PRD.md), and [website content strategy](PRD-website.md).

These Markdown pages can also supply a future documentation website. The root
[README](../README.md) stays focused on the introduction and first steps.
