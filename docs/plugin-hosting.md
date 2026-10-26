# Hosting a Plugin's UI

How a plugin's own interface reaches the browser, and why it is built this way.

## The gap this closes

SIP's manifest reference documented `[[ui.routes]]` from the start, and
`docs/plugins/ui-plugins.md` described route layouts and compatibility levels. But
nothing consumed a route declaration:

- `PluginManifest::public_view()` mapped `ui` to `PublicUiInfo`, which carried
  `kind`, `framework`, `entrypoint`, `dev_url`, `production_mount`,
  `compatibility` and `theme` — and dropped `routes` and `actions` entirely.
- `GET /api/v1/ui/navigation` returned only sidebar links, which is all the shell
  ever fetched (`frontend/app/shell.tsx`).
- Every page under `frontend/app/` was a hand-written Next.js file, including the
  pages belonging to `sip-migration-studio`.

The result: a plugin could appear in the sidebar and had nowhere to point. A
declared route such as `/test-bench` returned 404. Plugins were manifests plus
first-party code, not independent components.

## The contract now

```
plugin.toml                      SIP API                          SIP shell (Next.js)
───────────                      ───────                          ───────────────────
[[ui.routes]]                    GET /api/v1/ui/plugins           GET /api/v1/ui/plugins
  path  = "/test-bench"    ──▶     └─ ui.routes[] with mount  ──▶   └─ build path → route map
  mount = "/api/v1/ui/plugins/                                       app/[...pluginSlug]/page.tsx
           <id>/app/index.html"                                      └─ <PluginFrame src={mount}>
                                                                        └─ iframe + postMessage

                                 GET /api/v1/ui/plugins/<id>/app/<path>
                                   └─ serves plugins/<id>/dist/<path>
```

Three pieces:

### 1. A route can declare where its application lives

`UiRouteDef` gained one optional field:

```toml
[[ui.routes]]
id = "test-bench"
path = "/test-bench"
mount = "/api/v1/ui/plugins/sip-test-bench/app/index.html"
layout = "standalone"
```

`mount` may be:

- **same-origin** — a path, normally the SIP API's plugin-app route (see below);
- **absolute** — `https://plugins.example.com/bench/`, for an externally hosted app.

A route with neither `mount` nor a `ui.production_mount` fallback is *not* hosted
by this mechanism. It is a legacy declarative entry whose `component` is expected
to exist in the first-party tree, and it is skipped rather than half-rendered.

### 2. The API hosts plugin build output

```
GET /api/v1/ui/plugins/{plugin_id}/app/{*path}
  → <SIP_PLUGIN_DIR>/<plugin_id>/dist/<path>       (default SIP_PLUGIN_DIR=plugins)
```

- `plugin_id` is re-validated as `[a-z0-9_-]` before it is used as a path segment,
  and any path containing `.`, `..` or an empty segment is rejected, so a manifest
  cannot reach outside its own `dist/`.
- The resolved path is re-checked to still be under the plugin's `dist/`.
- Content type comes from a fixed extension table; anything unrecognised is served
  as `application/octet-stream` rather than guessed.
- Served from the unauthenticated router, because it is build output only. Secrets
  never appear in it — the plugin receives the user's token over `postMessage`,
  scoped to its own origin (below).

This is why a plugin needs no container of its own: `plugins/<id>/dist/` is the
deployment unit, and `SIP_PLUGIN_DIR` already controls where plugins are loaded
from.

### 3. The shell hosts the plugin in an iframe

`frontend/app/[...pluginSlug]/page.tsx` is the only first-party file involved. It
is a catch-all: every static route in `app/` wins over it, so existing pages are
untouched, and it exists solely to resolve paths that only a plugin knows about.

`frontend/components/plugin-frame.tsx` renders the plugin with:

```html
<iframe sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-modals"
        src="…/app/index.html" data-sip-plugin="sip-test-bench">
```

The sandbox deliberately withholds `allow-top-navigation`, so a plugin cannot
navigate the host away.

## The token handshake

An iframe is a separate document on possibly a separate origin. It cannot read the
shell's `localStorage`, which is the point — and also the problem, since it needs
the user's JWT to call the API.

The shell therefore hands it over once, over `postMessage`:

| # | Direction | Message |
|---|---|---|
| 1 | plugin → shell | `{ type: 'sip-plugin:hello', pluginId }` |
| 2 | shell → plugin | `{ type: 'sip-plugin:context', token, apiBase, user, theme }` |
| 3 | plugin → shell | `{ type: 'sip-plugin:ready' }` |
| 4 | plugin → shell | `{ type: 'sip-plugin:resize', height }` (optional) |
| 5 | plugin → shell | `{ type: 'sip-plugin:error', message }` (optional) |

Two rules the shell enforces:

- It replies to a `hello` **only** when `event.source === iframe.contentWindow`,
  so no other window on the page can ask for the token.
- It sends the context to the frame's **own origin**, never `'*'`.

A plugin that is opened directly (its own dev server, `npm run dev`) never receives
a context. Good plugins detect this and offer manual token + API-base entry so
they remain usable standalone — `sip-test-bench` does exactly that.

## What a plugin must do

1. Ship `plugin.toml` with `[[ui.routes]]` entries that carry `mount`.
2. Build its app to `plugins/<id>/dist/` with **relative asset paths**
   (`base: './'` in Vite). Assets are fetched from
   `/api/v1/ui/plugins/<id>/app/…`, a deep path, so absolute `/assets/...` URLs
   break.
3. Use in-document routing (hash routing) rather than path routing, so the app
   stays a single static document and needs no server rewrites.
4. Implement the handshake and declare `[ui.compatibility] level = "standalone"`.

## Compatibility levels, revisited

The levels in `docs/plugins/ui-plugins.md` still describe intent, and the iframe
host is the implementation of `standalone`. Note the honest gap: `native` and
`compatible` plugins are specified to render *inside* the shell using SIP's React
components, which requires the plugin's source to be compiled into the first-party
frontend build. That build hook does not exist yet. Today:

| Level | Status |
|---|---|
| `standalone` | Implemented — iframe host described above. |
| `compatible` | Not implemented. Hosted like `standalone` if it declares a mount, but it will not share the shell's DOM or components. |
| `native` | Not implemented. Needs a generated plugin registry plus a Next.js/webpack build hook. |

The `native` compatibility flags in a manifest are advisory until that build hook
exists. Do not assume a `native` plugin renders in-shell.

## No remote code execution in the shell

`docs/plugins/manifest-reference.md` states that SIP supports "no remote module
loading, eval, or component injection". That remains true and is why this design
uses an iframe: the plugin's JavaScript runs in its own document and origin, and
the shell never imports or evaluates plugin code. The plugin's capabilities come
from the API's own auth and RBAC — the handshake only transports a token the plugin
would otherwise have to be given anyway.
