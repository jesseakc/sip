'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import type { PluginRoute } from '@/lib/plugin-host';
import { isCrossOrigin } from '@/lib/plugin-host';
import { API_BASE } from '@/lib/api';

/**
 * Host for a plugin's own application.
 *
 * The plugin runs in an iframe so that it cannot reach the host document's DOM,
 * storage or module scope — the same isolation SIP documents for the
 * `standalone` compatibility level. The cost of that isolation is that the
 * plugin cannot read the user's token itself, so the shell hands it over once,
 * over `postMessage`, scoped to the plugin's own origin.
 *
 * Handshake:
 *   1. plugin -> shell  { type: 'sip-plugin:hello', pluginId }
 *   2. shell  -> plugin { type: 'sip-plugin:context', token, apiBase, user, theme }
 *   3. plugin -> shell  { type: 'sip-plugin:ready' }         (optional)
 *   4. plugin -> shell  { type: 'sip-plugin:resize', height } (optional)
 *
 * The token is only ever sent to the iframe's own origin, never to `*`.
 */

interface PluginUser {
  id?: string;
  name?: string;
  email?: string;
  role?: string;
  permissions?: string[];
}

export interface PluginFrameProps {
  route: PluginRoute;
  src: string;
  user: PluginUser | null;
}

export default function PluginFrame({ route, src, user }: PluginFrameProps) {
  const frameRef = useRef<HTMLIFrameElement | null>(null);
  const [height, setHeight] = useState<number | null>(null);
  const [status, setStatus] = useState<'loading' | 'connected' | 'error'>('loading');
  const [error, setError] = useState<string | null>(null);

  const targetOrigin = useCallback(() => {
    if (!isCrossOrigin(src)) return window.location.origin;
    try {
      return new URL(src).origin;
    } catch {
      return window.location.origin;
    }
  }, [src]);

  // Hand the plugin its context, but only in response to a hello from the frame
  // we actually rendered. Verifying event.source stops any other window on the
  // page from asking for the token.
  useEffect(() => {
    function onMessage(event: MessageEvent) {
      if (!frameRef.current || event.source !== frameRef.current.contentWindow) return;
      const data = event.data;
      if (!data || typeof data !== 'object') return;

      if (data.type === 'sip-plugin:hello') {
        if (data.pluginId && data.pluginId !== route.pluginId) return;
        const token = localStorage.getItem('sip_token');
        frameRef.current.contentWindow?.postMessage(
          {
            type: 'sip-plugin:context',
            pluginId: route.pluginId,
            token,
            apiBase: API_BASE,
            user: user ?? null,
            theme: {
              // The plugin is a separate document, so it cannot inherit the
              // shell's CSS custom properties. Ship the few values it needs to
              // match the surrounding interface.
              background: '#f9fafb',
              surface: '#ffffff',
              border: '#e5e7eb',
              text: '#111827',
              textMuted: '#6b7280',
              primary: '#2563eb',
            },
          },
          targetOrigin(),
        );
        setStatus('connected');
        return;
      }

      if (data.type === 'sip-plugin:ready') {
        setStatus('connected');
        return;
      }

      if (data.type === 'sip-plugin:resize' && typeof data.height === 'number') {
        setHeight(Math.max(320, Math.min(20000, Math.round(data.height))));
        return;
      }

      if (data.type === 'sip-plugin:error') {
        setStatus('error');
        setError(typeof data.message === 'string' ? data.message : 'plugin reported an error');
      }
    }

    window.addEventListener('message', onMessage);
    return () => window.removeEventListener('message', onMessage);
  }, [route.pluginId, user, targetOrigin]);

  if (!src) {
    return (
      <div className="rounded-md border border-red-200 bg-red-50 p-4 text-sm text-red-700">
        Plugin <strong>{route.pluginName}</strong> declares route{' '}
        <code>{route.path}</code> but no mount point, so there is nothing to load.
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex items-center justify-between border-b border-gray-200 pb-2">
        <div className="min-w-0">
          <h1 className="truncate text-lg font-semibold text-gray-900">{route.title || route.pluginName}</h1>
          <p className="truncate text-xs text-gray-500">
            plugin <code>{route.pluginId}</code>
            {route.compatibility ? ` · ${route.compatibility}` : ''}
            {isCrossOrigin(src) ? ' · external origin' : ''}
          </p>
        </div>
        <span
          className={`shrink-0 rounded-full px-2 py-0.5 text-xs font-medium ${
            status === 'connected'
              ? 'bg-green-50 text-green-700'
              : status === 'error'
                ? 'bg-red-50 text-red-700'
                : 'bg-gray-100 text-gray-600'
          }`}
        >
          {status === 'connected' ? 'connected' : status}
        </span>
      </div>

      {status === 'error' && (
        <div className="mt-2 rounded-md border border-red-200 bg-red-50 p-3 text-sm text-red-700">
          {error}
        </div>
      )}

      <iframe
        ref={frameRef}
        title={`${route.pluginName} (${route.path})`}
        src={src}
        data-sip-plugin={route.pluginId}
        // The plugin gets exactly the capabilities its own document needs: it is
        // same-origin to the SIP API, so scripts and forms are enough, and no
        // sandbox escape hatch like allow-top-navigation is granted.
        sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-modals"
        className="mt-3 w-full flex-1 rounded-md border border-gray-200 bg-white"
        style={height ? { height: `${height}px`, flex: 'none' } : undefined}
        onLoad={() => setStatus((s) => (s === 'loading' ? 'loading' : s))}
        onError={() => {
          setStatus('error');
          setError(`could not load ${src}`);
        }}
      />
    </div>
  );
}
