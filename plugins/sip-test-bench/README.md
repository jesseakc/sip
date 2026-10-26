# SIP Test Bench (`sip-test-bench`)

A standalone UI plugin for SIP: **Service Intelligence Platform**. It is a test
harness, not a product surface — it loads fixture data through the real
migration pipeline, queries the AI and shows its traces, exercises every API
endpoint as any role, and runs read-only integrity checks.

It is the first SIP plugin to ship its own built UI bundle, so it is also the
reference implementation of the generic plugin UI host contract.

---

## What it does

Four hash-routed views:

| Route | View | Purpose |
| --- | --- | --- |
| `#/datasets` | **Datasets** | Shows the shipped fixture load plan and can push all five object types through the migration pipeline (create → upload → validate → dry-run → import → report) in dependency order, then compares live SIP entity counts with the fixture. |
| `#/query` | **Query Console** | Streams `POST /api/v1/ai/chat` over SSE, rendering the answer, sources, verification result and any unknown event verbatim; fetches retrieval/verification traces; lists conversations. |
| `#/api` | **API Explorer** | Curated endpoint list with a method picker, editable path, JSON body editor and a raw response pane (status, latency, exact bytes). Includes *login as* for the six seeded SIP roles. |
| `#/checks` | **Checks** | A read-only battery: health, capabilities, UI plugin registry (asserts this plugin exposes `ui.routes` with a mount), navigation, assets, work orders, parts, plus migration-job and fixture-count checks. |

Every number shown comes from a live HTTP response. Failures — 401s, transport
errors, empty payloads — are rendered as results, not hidden. There is no mock
data anywhere in this app.

---

## How it is hosted

The SIP API serves a plugin's built bundle from `plugins/<id>/dist/`:

```
GET /api/v1/ui/plugins/sip-test-bench/app/index.html
GET /api/v1/ui/plugins/sip-test-bench/app/assets/<file>
GET /api/v1/ui/plugins/sip-test-bench/app/fixtures/<file>
```

The route is declared in `plugin.toml`:

```toml
[[ui.routes]]
id = "test-bench"
path = "/test-bench"
mount = "/api/v1/ui/plugins/sip-test-bench/app/index.html"
layout = "standalone"
```

Because the document is loaded from a deep path, **every asset URL is
relative** (`base: './'` in `vite.config.ts`). An absolute `/assets/...` URL
would resolve against the API origin root and 404.

`dist/` is **committed** (`.gitignore` ignores only `node_modules/`) because the
API serves it directly.

### postMessage handshake

The shell embeds this app in an iframe with
`sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-modals"`
and owns the auth token. On startup the app (`src/sip-bridge.ts`):

1. posts `{type:'sip-plugin:hello', pluginId:'sip-test-bench'}` to `window.parent`;
2. retries that every 500 ms, up to 20 attempts (~10 s);
3. accepts `{type:'sip-plugin:context'}` only when `pluginId === 'sip-test-bench'`,
   storing `token`, `apiBase`, `user` and `theme`;
4. replies `{type:'sip-plugin:ready'}`;
5. uses `apiBase` + `Authorization: Bearer <token>` for every request;
6. posts `{type:'sip-plugin:resize', height}` whenever content height changes
   (ResizeObserver plus a 1 s poll) so the host can size the frame.

If no context arrives within ~10 s the app still renders, in **standalone mode**:
a banner explains it is not hosted by the shell and offers manual token + API
base entry, persisted in `sessionStorage` for that tab.

---

## Layout

```
plugins/sip-test-bench/
  plugin.toml                  manifest (type = "ui", layout = "standalone")
  package.json                 dev / build / preview / sync-fixtures
  vite.config.ts               base './', outDir 'dist', target es2020
  tsconfig.json                strict, noUnusedLocals/Parameters
  index.html
  .gitignore                   node_modules only — dist/ is committed
  scripts/sync-fixtures.mjs    copies the SMRT canonical fixture into public/
  public/fixtures/             populated by sync-fixtures (build input)
  dist/                        vite build output (served by the API)
  src/
    main.tsx                   entry; creates the bridge before React mounts
    App.tsx                    shell chrome, hash routing, status strip,
                               standalone credentials, identity probing
    sip-bridge.ts              postMessage handshake + resize reporting
    api.ts                     typed fetch wrapper; never discards a response
    ui.tsx                     shared presentational primitives
    styles.css                 standalone palette, light + dark
    views/Datasets.tsx
    views/QueryConsole.tsx
    views/ApiExplorer.tsx
    views/Checks.tsx
```

## Fixtures

`scripts/sync-fixtures.mjs` runs automatically in `prebuild` (and `predev`). It
reads the canonical SMRT fixture from
`data/smrt-maintenance-logs/fixture/canonical/`, copies the five files into
`public/fixtures/`, and writes `public/fixtures/index.json` describing the load
plan and record counts:

```
location (01) -> asset_type (02) -> asset (03) -> part (04) -> work_order (05)
```

That order is a hard dependency: assets resolve to their asset type and
location by `external_id`, and work orders resolve to assets and parts.

If the data directory is missing (plugin checked out alone, CI), the script
prints a warning and exits 0 so the build still succeeds; the UI then reports
the missing load plan at runtime instead of inventing one.

The app fetches the plan relative to its own document:

```js
new URL('fixtures/index.json', document.baseURI)
```

## Develop / build / verify

```bash
cd plugins/sip-test-bench
npm install
npm run dev            # http://localhost:5173 (standalone mode)
npm run sync-fixtures  # refresh public/fixtures/ by hand
npm run typecheck      # tsc --noEmit
npm run build          # sync-fixtures + tsc --noEmit + vite build -> dist/
npm run preview        # serve dist/ (vite preview)
```

To exercise the real pipeline you need the SIP API running (Postgres up, see the
repo root `QUICKSTART.md`). Without it, every API-driven pane honestly reports
connection failures; the fixture-driven parts (load plan, counts) still work.

### Identity and permissions

The API Explorer's *login as* uses the shared seed password `password` for
`admin@acme.local`, `manager@acme.local`, `tech1@acme.local`, `viewer@acme.local`,
`vendor@acme.local` and `auditor@acme.local`. Adopting a token updates the status
strip identity so it is always explicit which identity the current token belongs
to.

### Rollback expectations

*Rollback all* posts `/rollback` for every job created in the session. Locations,
asset types and parts have no archive operation in SIP, so those rollbacks
report zero processed records — that is expected, not a bug. Assets and work
orders are archived.

## Notes on the build output

`vite build` emits `dist/index.html`, `dist/assets/*` and a verbatim copy of
`public/fixtures/`. The bundled fixtures include the ~10 MB work-order file, so
`dist/` is roughly 11 MB. That is deliberate: the Test Bench loads the fixture
over HTTP at runtime rather than bundling it into the JS.

## Licence

AGPL-3.0-or-later, matching SIP.
