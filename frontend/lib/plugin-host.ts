import { API_BASE, getToken } from './api';

/**
 * Plugin route hosting.
 *
 * SIP plugin manifests declare `[[ui.routes]]`, but until now nothing consumed
 * them: `public_view()` dropped the route table from `/api/v1/ui/plugins`, and
 * every page in this frontend was a hardcoded Next.js file. This module closes
 * that loop. It reads the route table from the plugin discovery API and resolves
 * an incoming pathname to the plugin that owns it, so a plugin's own application
 * can be mounted without a first-party page existing for it.
 */

export interface PluginRoute {
  pluginId: string;
  pluginName: string;
  /** URL path this route owns, e.g. `/test-bench`. */
  path: string;
  id: string;
  layout?: string | null;
  title?: string | null;
  breadcrumb?: string | null;
  requiredPermissions: string[];
  /** Where the plugin serves this route itself. */
  mount?: string | null;
  compatibility?: string | null;
  /** Whether the plugin needs to render inside the SIP shell. */
  requiresShell: boolean;
}

interface RawRouteDef {
  id: string;
  path: string;
  component?: string | null;
  layout?: string | null;
  title?: string | null;
  breadcrumb?: string | null;
  required_permissions?: string[] | null;
  mount?: string | null;
}

interface RawUiInfo {
  kind?: string;
  framework?: string | null;
  entrypoint?: string | null;
  dev_url?: string | null;
  production_mount?: string | null;
  compatibility?: { level?: string | null; requires_shell?: boolean | null } | null;
  routes?: RawRouteDef[] | null;
}

interface RawUiPlugin {
  id: string;
  name: string;
  ui?: RawUiInfo | null;
  enabled?: boolean;
}

/**
 * Resolve where a plugin's application actually lives.
 *
 * Precedence: an explicit per-route `mount`, then `ui.dev_url` when
 * NEXT_PUBLIC_PLUGIN_DEV is set (a plugin iterated on with its own dev server),
 * then `ui.production_mount`.
 */
function resolveMount(ui: RawUiInfo | null | undefined, route: RawRouteDef): string | null {
  if (route.mount) return route.mount;
  if (!ui) return null;
  const devMode = process.env.NEXT_PUBLIC_PLUGIN_DEV === '1';
  if (devMode && ui.dev_url) return ui.dev_url;
  return ui.production_mount ?? null;
}

let cache: PluginRoute[] | null = null;
let inFlight: Promise<PluginRoute[]> | null = null;

/** Drop the cached route table. Call after enabling or disabling a plugin. */
export function invalidatePluginRoutes() {
  cache = null;
  inFlight = null;
}

/**
 * Fetch the plugin route table. Public endpoint, so a token is optional, but
 * sending one keeps behaviour consistent if the endpoint is later gated.
 */
export async function loadPluginRoutes(force = false): Promise<PluginRoute[]> {
  if (cache && !force) return cache;
  if (inFlight && !force) return inFlight;

  inFlight = (async () => {
    const token = getToken();
    const res = await fetch(`${API_BASE}/ui/plugins`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
    if (!res.ok) throw new Error(`plugin discovery failed: HTTP ${res.status}`);
    const json = await res.json();
    const plugins: RawUiPlugin[] = Array.isArray(json?.data) ? json.data : [];

    const routes: PluginRoute[] = [];
    for (const plugin of plugins) {
      const ui = plugin.ui;
      if (!ui?.routes?.length) continue;
      for (const route of ui.routes) {
        // A route with no mount and no production_mount has no application to
        // host: it is a declarative entry whose component is expected to exist
        // in the first-party tree, which is the old behaviour. Skip it here so
        // this host never claims a route it cannot actually render.
        const mount = resolveMount(ui, route);
        if (!mount) continue;
        routes.push({
          pluginId: plugin.id,
          pluginName: plugin.name,
          id: route.id,
          path: route.path,
          layout: route.layout ?? null,
          title: route.title ?? null,
          breadcrumb: route.breadcrumb ?? null,
          requiredPermissions: route.required_permissions ?? [],
          mount,
          compatibility: ui.compatibility?.level ?? null,
          requiresShell: ui.compatibility?.requires_shell ?? true,
        });
      }
    }

    // Longest path first so /test-bench/runs wins over /test-bench.
    routes.sort((a, b) => b.path.length - a.path.length);
    cache = routes;
    inFlight = null;
    return routes;
  })();

  try {
    return await inFlight;
  } catch (e) {
    inFlight = null;
    throw e;
  }
}

/** Find the plugin route that owns `pathname`, using longest-prefix matching. */
export function resolvePluginRoute(
  routes: PluginRoute[],
  pathname: string,
): PluginRoute | undefined {
  const clean = pathname.replace(/\/+$/, '') || '/';
  return routes.find((r) => {
    const base = r.path.replace(/\/+$/, '') || '/';
    return clean === base || clean.startsWith(`${base}/`);
  });
}

/**
 * Build the iframe URL for a route, preserving any sub-path below the route
 * base so a plugin can implement its own nested views.
 *
 * Same-origin mounts served by the SIP API (the common case) target their
 * `index.html`, because the API's asset route serves files rather than
 * directory listings. Plugins are expected to use hash-based internal routing so
 * this stays a single static document.
 */
/**
 * Origin the SIP API is served from.
 *
 * Plugin bundles are served by the API, not by this frontend, and
 * `next.config.ts` defines no proxy — so a relative mount such as
 * `/api/v1/ui/plugins/x/app/index.html` has to be resolved against the API's
 * origin. Left relative it would resolve against the frontend's own origin and
 * land on the catch-all route instead of the plugin.
 */
export function apiOrigin(): string {
  try {
    return new URL(API_BASE, typeof window === 'undefined' ? undefined : window.location.origin).origin;
  } catch {
    return typeof window === 'undefined' ? '' : window.location.origin;
  }
}

export function buildMountUrl(route: PluginRoute, pathname: string): string {
  const base = route.path.replace(/\/+$/, '');
  const rest = pathname.slice(base.length).replace(/^\/+/, '');
  let mount = route.mount ?? '';
  const isAbsolute = /^https?:\/\//i.test(mount);
  if (!isAbsolute) {
    // Same-origin to the API: prefix with its origin, and target a real file
    // because the API's asset route serves files rather than directory indexes.
    mount = mount.replace(/\/+$/, '');
    if (!/\.html?$/i.test(mount) && !mount.endsWith('/index.html')) {
      mount = `${mount}/index.html`;
    }
    mount = `${apiOrigin()}${mount.startsWith('/') ? '' : '/'}${mount}`;
  }
  return rest ? `${mount}#/${rest}` : mount;
}

/** True when the plugin's application is served from a different origin. */
export function isCrossOrigin(mount: string): boolean {
  if (!/^https?:\/\//i.test(mount)) return false;
  if (typeof window === 'undefined') return false;
  try {
    return new URL(mount).origin !== window.location.origin;
  } catch {
    return false;
  }
}
