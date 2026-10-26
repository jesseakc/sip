import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  createBridge,
  type BridgeSnapshot,
  type PluginUser,
  defaultApiBase,
  normalizeApiBase,
} from './sip-bridge';
import { api, errorMessage } from './api';

import Datasets from './views/Datasets';
import QueryConsole from './views/QueryConsole';
import ApiExplorer from './views/ApiExplorer';
import Checks from './views/Checks';

/** What every view needs to make requests. */
export interface BenchSession {
  /** Bearer token in use, or null when nothing has been supplied yet. */
  token: string | null;
  apiBase: string;
  /** Human-readable description of whose token this is. */
  identity: string;
}

export const VIEW_IDS = ['datasets', 'query', 'api', 'checks'] as const;
export type ViewId = (typeof VIEW_IDS)[number];

const NAV: Array<{ id: ViewId; href: string; label: string }> = [
  { id: 'datasets', href: '#/datasets', label: 'Datasets' },
  { id: 'query', href: '#/query', label: 'Query Console' },
  { id: 'api', href: '#/api', label: 'API Explorer' },
  { id: 'checks', href: '#/checks', label: 'Checks' },
];

function parseHash(hash: string): ViewId {
  const cleaned = hash.replace(/^#\/?/, '').split('?')[0].split('/')[0];
  const found = VIEW_IDS.find((v) => v === cleaned);
  return found ?? 'datasets';
}

function useRoute(): ViewId {
  const [view, setView] = useState<ViewId>(() => parseHash(window.location.hash));
  useEffect(() => {
    const onHash = () => setView(parseHash(window.location.hash));
    window.addEventListener('hashchange', onHash);
    // Normalise a bare URL to an explicit route so links highlight correctly.
    if (!window.location.hash) window.location.hash = '#/datasets';
    return () => window.removeEventListener('hashchange', onHash);
  }, []);
  return view;
}

function useBridge(): BridgeSnapshot {
  const bridge = useMemo(() => createBridge(), []);
  const [snap, setSnap] = useState<BridgeSnapshot>(() => bridge.getSnapshot());
  useEffect(() => bridge.subscribe(setSnap), [bridge]);
  return snap;
}

function userLabel(user: PluginUser | null): string | null {
  if (!user) return null;
  const email = typeof user.email === 'string' ? user.email : null;
  const role = typeof user.role === 'string' ? user.role : null;
  const name = typeof user.name === 'string' ? user.name : null;
  if (email && role) return `${email} (${role})`;
  if (email) return email;
  if (name) return name;
  return null;
}

export default function App() {
  const bridge = useMemo(() => createBridge(), []);
  const snap = useBridge();
  const view = useRoute();

  // Token override: set by the shell context, by the standalone credential
  // form, or by "login as" in the API Explorer.
  const [override, setOverride] = useState<{ token: string; label: string } | null>(null);

  const [manualToken, setManualToken] = useState('');
  const [manualBase, setManualBase] = useState('');
  const [manualError, setManualError] = useState<string | null>(null);
  const [probe, setProbe] = useState<{ status: number; label: string; raw: unknown } | null>(null);
  const [probing, setProbing] = useState(false);

  const shellToken = snap.context?.token ?? null;
  const token = override?.token ?? shellToken;
  const apiBase = normalizeApiBase(snap.context?.apiBase ?? '') || defaultApiBase();

  const identity = useMemo(() => {
    if (override) return override.label;
    const fromShell = userLabel(snap.context?.user ?? null);
    if (fromShell) return fromShell;
    if (shellToken) return 'shell token (identity unknown — use Identify or API Explorer → login as)';
    return 'no token';
  }, [override, snap.context, shellToken]);

  const session: BenchSession = { token, apiBase, identity };

  // Seed the manual form when we fall back to standalone with stored values.
  useEffect(() => {
    if (snap.state === 'standalone') {
      const stored = bridge.getSnapshot().context;
      if (stored) {
        setManualToken((prev) => prev || stored.token);
        setManualBase((prev) => prev || stored.apiBase);
      } else if (!manualBase) {
        setManualBase(defaultApiBase());
      }
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [snap.state]);

  // Keep the host frame sized to our content whenever the view changes.
  useEffect(() => {
    bridge.notifyResize();
  }, [bridge, view]);

  const onUseToken = useCallback((next: string, label: string) => {
    setOverride({ token: next, label });
  }, []);

  const applyManual = useCallback(() => {
    const t = manualToken.trim();
    const b = manualBase.trim() || defaultApiBase();
    if (!t) {
      setManualError('A token is required — SIP rejects unauthenticated API calls with 401.');
      return;
    }
    setManualError(null);
    bridge.setStandaloneContext({ token: t, apiBase: b });
    setOverride({ token: t, label: `manual token (${b})` });
  }, [bridge, manualToken, manualBase]);

  const identify = useCallback(async () => {
    if (!token) return;
    setProbing(true);
    setProbe(null);
    const res = await api('/api/v1/auth/me', { token, apiBase });
    setProbing(false);
    if (res.error) {
      setProbe({ status: 0, label: res.error, raw: res.text });
      return;
    }
    const me = (res.json as { data?: Record<string, unknown> } | null)?.data ?? res.json;
    const record = (me ?? {}) as Record<string, unknown>;
    const email = typeof record.email === 'string' ? record.email : null;
    const role = typeof record.role === 'string' ? record.role : null;
    const label = email ? `${email}${role ? ` (${role})` : ''}` : errorMessage(res.json, 'identity not present in /auth/me payload');
    setProbe({ status: res.status, label, raw: res.json ?? res.text });
    if (res.ok && email) {
      setOverride({ token: token, label });
    }
  }, [token, apiBase]);

  const connectionKind = snap.state === 'connected' ? 'connected' : snap.state === 'connecting' ? 'connecting' : 'standalone';
  const connectionText =
    snap.state === 'connected'
      ? 'connected to shell'
      : snap.state === 'connecting'
        ? `waiting for shell (hello ${snap.attempts}/20)`
        : 'standalone / dev mode';

  const viewNode = (() => {
    switch (view) {
      case 'datasets':
        return <Datasets session={session} />;
      case 'query':
        return <QueryConsole session={session} />;
      case 'api':
        return <ApiExplorer session={session} onUseToken={onUseToken} />;
      case 'checks':
        return <Checks session={session} />;
    }
  })();

  return (
    <div className="app">
      <header className="topbar">
        <h1>SIP Test Bench</h1>
        <span className="tag">sip-test-bench v0.1.0 · standalone UI plugin</span>
      </header>

      <nav className="nav">
        {NAV.map((item) => (
          <a key={item.id} href={item.href} className={view === item.id ? 'active' : ''}>
            {item.label}
          </a>
        ))}
      </nav>

      <div className="status-strip">
        <div className="item">
          <span className={`dot ${connectionKind}`} />
          <span className="label">connection</span>
          <span className="value">{connectionText}</span>
        </div>
        <div className="item">
          <span className="label">api base</span>
          <span className="value">{apiBase}</span>
        </div>
        <div className="item">
          <span className="label">identity</span>
          <span className="value">{identity}</span>
          {token ? <span className="small muted">token …{token.slice(-6)}</span> : null}
        </div>
        <div className="item">
          <button className="ghost" onClick={identify} disabled={!token || probing}>
            {probing ? <span className="spinner" /> : null} Identify
          </button>
        </div>
      </div>

      {probe ? (
        <div className={`banner ${probe.status === 200 ? 'info' : 'warn'}`}>
          <strong>
            GET /api/v1/auth/me → {probe.status === 0 ? 'transport error' : probe.status}
          </strong>
          <p>{probe.label}</p>
          <details className="raw">
            <summary>Raw /auth/me response</summary>
            <pre className="mono-block">{typeof probe.raw === 'string' ? probe.raw : JSON.stringify(probe.raw, null, 2)}</pre>
          </details>
        </div>
      ) : null}

      {snap.state === 'standalone' ? (
        <div className="banner warn">
          <strong>Standalone mode — not hosted by the SIP shell</strong>
          <p>
            {snap.notice ??
              'This app is running outside the SIP shell, so no token was handed to it by postMessage.'}
          </p>
          <div className="row" style={{ marginTop: 10 }}>
            <div className="field grow">
              <label htmlFor="manual-token">Bearer token</label>
              <input
                id="manual-token"
                type="password"
                className="mono"
                autoComplete="off"
                placeholder="paste a JWT from POST /api/v1/auth/login"
                value={manualToken}
                onChange={(e) => setManualToken(e.target.value)}
              />
            </div>
            <div className="field grow">
              <label htmlFor="manual-base">API base</label>
              <input
                id="manual-base"
                type="text"
                className="mono"
                placeholder={defaultApiBase()}
                value={manualBase}
                onChange={(e) => setManualBase(e.target.value)}
              />
            </div>
            <div className="row tight">
              <button className="primary" onClick={applyManual}>
                Use these credentials
              </button>
              <button
                onClick={() => {
                  bridge.clearStandaloneContext();
                  setOverride(null);
                  setManualToken('');
                  setManualError(null);
                }}
              >
                Clear
              </button>
            </div>
          </div>
          {manualError ? (
            <p className="small" style={{ color: 'var(--danger)', marginTop: 8 }}>
              {manualError}
            </p>
          ) : null}
          <p className="small muted" style={{ marginTop: 8 }}>
            Credentials are kept in <code>sessionStorage</code> for this tab only.
          </p>
        </div>
      ) : null}

      {snap.state === 'connecting' ? (
        <div className="banner">
          <strong>Handshaking with the SIP shell</strong>
          <p>
            Sent <code>{'sip-plugin:hello'}</code> {snap.attempts} time(s); waiting for{' '}
            <code>{'sip-plugin:context'}</code>. The app renders regardless and will fall back to
            standalone mode after ~10s.
          </p>
        </div>
      ) : null}

      {viewNode}

      <footer className="small muted" style={{ marginTop: 28 }}>
        Every value on this page comes from a live HTTP response. Requests are issued from the browser
        with <code>Authorization: Bearer &lt;token&gt;</code>; status codes and raw bodies are shown
        verbatim, including failures. Source: <code>plugins/sip-test-bench</code>.
      </footer>
    </div>
  );
}
