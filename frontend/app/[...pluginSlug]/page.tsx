'use client';

import { useEffect, useState } from 'react';
import { usePathname } from 'next/navigation';
import { useAuth } from '@/lib/auth-context';
import {
  buildMountUrl,
  loadPluginRoutes,
  resolvePluginRoute,
  type PluginRoute,
} from '@/lib/plugin-host';
import PluginFrame from '@/components/plugin-frame';

/**
 * Catch-all route for plugin-hosted pages.
 *
 * Static routes in `app/` always win over this dynamic segment, so every
 * first-party page keeps its own file. This handles the rest: paths that only a
 * plugin's manifest knows about. Previously such a path was a 404, which is why
 * plugins could appear in the sidebar but had nowhere to point.
 *
 * Resolution happens client-side because the route table lives in the plugin
 * registry behind the API, not in the build.
 */
export default function PluginRoutePage() {
  const pathname = usePathname();
  const { user } = useAuth();
  const [route, setRoute] = useState<PluginRoute | null>(null);
  const [state, setState] = useState<'loading' | 'resolved' | 'unknown' | 'error'>('loading');
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setState('loading');

    loadPluginRoutes()
      .then((routes) => {
        if (cancelled) return;
        const match = resolvePluginRoute(routes, pathname);
        if (!match) {
          setRoute(null);
          setState('unknown');
          return;
        }
        setRoute(match);
        setState('resolved');
      })
      .catch((e: unknown) => {
        if (cancelled) return;
        setError(e instanceof Error ? e.message : String(e));
        setState('error');
      });

    return () => {
      cancelled = true;
    };
  }, [pathname]);

  if (state === 'loading') {
    return <p className="text-sm text-gray-500">Resolving plugin route {pathname}…</p>;
  }

  if (state === 'error') {
    return (
      <div className="rounded-md border border-red-200 bg-red-50 p-4 text-sm text-red-700">
        <p className="font-medium">Could not load the plugin route table.</p>
        <p className="mt-1">{error}</p>
        <p className="mt-2 text-xs">
          The plugin API is at <code>/api/v1/ui/plugins</code>. If the API is unreachable this page
          cannot resolve plugin routes.
        </p>
      </div>
    );
  }

  if (state === 'unknown' || !route) {
    return (
      <div className="rounded-md border border-gray-200 bg-white p-6">
        <h1 className="text-lg font-semibold text-gray-900">No plugin owns this path</h1>
        <p className="mt-1 text-sm text-gray-600">
          <code>{pathname}</code> is not a page in this frontend and no enabled plugin declares it in
          a <code>[[ui.routes]]</code> block with a mount point.
        </p>
      </div>
    );
  }

  return <PluginFrame route={route} src={buildMountUrl(route, pathname)} user={user} />;
}
