# Development

[Documentation home](README.md)

Use the [Quick Start](../QUICKSTART.md) for a local installation and your first
administrator. This page covers working on the source.

## Source layout

- `crates/`: the Rust API, application services, domain model, and supporting libraries.
- `frontend/`: the Next.js and TypeScript web interface.
- `plugins/`: plugin manifests, source, examples, and deployable UI bundles.
- `migrations/`: SQLx database migrations.
- `scripts/`: local diagnostics and demo tooling.
- `docs/`: guides, references, requirements, and design notes.

See the [platform reference](platform-reference.md#project-structure) for the
detailed tree and the [architecture guide](architecture.md) for runtime boundaries.

## Backend

The workspace declares Rust 1.80 as its minimum version. Use a toolchain compatible
with the committed lockfile. Docker builds with the `rust:1-slim-bookworm` image.
PostgreSQL needs `uuid-ossp`, `vector`, `ltree`, and `pg_trgm`; PostGIS is not required.

```bash
cargo install sqlx-cli --no-default-features --features postgres,rustls
sqlx migrate run --database-url postgres://sip:sip@localhost:5432/sip
export SIP_DATABASE__URL=postgres://sip:sip@localhost:5432/sip
cargo run -p sip-api
```

Use an existing local PostgreSQL service, or start Compose's data services with
`docker compose up -d postgres redis minio`. Avoid binding a second API to port
8000 if the Compose API is still running. Run all migrations, including the one
that removes the old demo tenant, and create your own administrator.

The `sip-cli` binary exposes command names, but its command bodies are placeholders.
Use SQLx for migrations rather than relying on `sip-cli migrate` or `sip-cli seed`.

## Frontend

The Docker build uses Node.js 22. From `frontend/`:

```bash
npm ci
npm run dev
```

The frontend defaults to `http://localhost:8000/api/v1`. To change it, set
`NEXT_PUBLIC_API_URL` in `frontend/.env.local`. Include `/api/v1`. Production
builds inline this setting, so a changed API address requires a new build.

## Configuration

The API loads `Sip.toml` and then `SIP_` environment variables. Nested variables
use **double underscores after the section name**:

| Setting | API environment variable |
|---------|--------------------------|
| Database | `SIP_DATABASE__URL` |
| API port | `SIP_SERVER__PORT` |
| JWT signing secret | `SIP_AUTH__JWT_SECRET` |
| Setup token | `SIP_SECURITY__BOOTSTRAP_TOKEN` |
| Allowed web origins | `SIP_SECURITY__CORS_ALLOWED_ORIGINS` |
| Redis | `SIP_REDIS__URL` |
| Object storage | `SIP_OBJECT_STORAGE__ENDPOINT`, `SIP_OBJECT_STORAGE__ACCESS_KEY`, `SIP_OBJECT_STORAGE__SECRET_KEY`, `SIP_OBJECT_STORAGE__BUCKET` |
| Enable AI | `SIP_AI__ENABLED` |
| AI provider | `SIP_AI__PROVIDER` |
| OpenAI-compatible endpoint | `SIP_AI__OPENAI__BASE_URL`, `SIP_AI__OPENAI__API_KEY`, `SIP_AI__OPENAI__MODEL` |
| Ollama | `SIP_AI__OLLAMA__URL`, `SIP_AI__OLLAMA__MODEL`, `SIP_AI__OLLAMA__EMBEDDING_MODEL` |
| Chat feature flag | `SIP_FEATURES__AI_CHAT_ENABLED` |
| Plugin feature flag | `SIP_FEATURES__PLUGIN_SYSTEM_ENABLED` |

`SIP_ENV` selects the environment. `local`, `development`, `test`, `staging`,
and `production` are recognized; production enables stricter validation.

**Compose variables and API variables are different layers.** The base Compose
file translates `SIP_AI_ENABLED`, `SIP_AI_PROVIDER`, `SIP_OLLAMA_MODEL`, and
`JWT_SECRET` into the nested names above. Other values in a root `.env` file are
not automatically passed into the API container. Add them through a Compose
override, or set them directly in your API host's environment. Native
`cargo run` also does not load the root `.env` file automatically.

For example, an API process using an OpenAI-compatible provider needs:

```dotenv
SIP_AI__ENABLED=true
SIP_AI__PROVIDER=openai
SIP_AI__OPENAI__BASE_URL=https://your-provider.example/v1
SIP_AI__OPENAI__API_KEY=your-provider-key
SIP_AI__OPENAI__MODEL=your-model
SIP_FEATURES__AI_CHAT_ENABLED=true
```

Set these in the process environment or a Compose override. Keep real keys in
untracked local configuration or the hosting platform's secret store.

For the existing local Ollama profile, the Compose settings are:

```bash
SIP_AI_ENABLED=true SIP_AI_PROVIDER=ollama \
  docker compose --profile ollama up -d
```

The profile requests `llama3.1:8b`. Confirm the container starts and the model is
available in its logs before testing chat. The profile alone does not change
the API's AI-disabled default. Embedding-based retrieval also needs the configured
embedding model, which defaults to `nomic-embed-text`.

The legacy `SIP_OLLAMA_URL` and `SIP_OLLAMA_MODEL` aliases are explicitly supported
by the native API. The [configuration source](../crates/sip-config/src/lib.rs)
defines all fields and validation rules. In local development, an authenticated
`GET /admin/config/status` request shows redacted configuration details.

## Build options

The API defaults to the `full` feature set. Smaller builds can select features:

```bash
cargo build -p sip-api --no-default-features
cargo build -p sip-api --no-default-features --features "ai,export"
```

The compile-time features are `ai`, `documents`, `export`, and `plugins`. Runtime
feature flags and provider settings still determine availability. Compiling a
feature does not complete an unfinished implementation.

## Checks

From the repository root:

```bash
cargo test --workspace --all-features
cargo check --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --all -- --check
```

From `frontend/`:

```bash
npm ci
npx tsc --noEmit
npm run build
```

`scripts/preflight.sh` checks local prerequisites. The existing `scripts/smoke.sh`
and parts of the [manual testing guide](testing-weekend.md) still assume the
removed Acme demo account. Adapt those credentials and data assumptions before
using them with an empty installation. The [Test Bench](../plugins/sip-test-bench/README.md)
provides additional workflow and trace inspection tools.
