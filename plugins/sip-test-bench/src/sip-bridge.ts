/**
 * sip-bridge.ts — postMessage handshake between the Test Bench and the SIP shell.
 *
 * The shell creates this app's iframe with
 *   sandbox="allow-scripts allow-same-origin allow-forms allow-popups allow-modals"
 * and holds the auth token; the app must never invent one. Protocol:
 *
 *   app  -> shell : {type:'sip-plugin:hello', pluginId:'sip-test-bench'}
 *   shell-> app   : {type:'sip-plugin:context', pluginId:'sip-test-bench',
 *                    token, apiBase, user, theme}
 *   app  -> shell : {type:'sip-plugin:ready'}
 *   app  -> shell : {type:'sip-plugin:resize', height:<px>}   (on content change)
 *
 * If no context arrives within CONTEXT_TIMEOUT_MS the app still renders, but in
 * standalone mode: the user supplies a token + API base by hand and it is kept
 * in sessionStorage so the app is usable opened directly at localhost:5173.
 */

export const PLUGIN_ID = 'sip-test-bench';

export const MESSAGE_HELLO = 'sip-plugin:hello';
export const MESSAGE_CONTEXT = 'sip-plugin:context';
export const MESSAGE_READY = 'sip-plugin:ready';
export const MESSAGE_RESIZE = 'sip-plugin:resize';

/** Hello retry cadence and cap (~20 attempts ≈ 10s). */
export const HELLO_INTERVAL_MS = 500;
export const HELLO_MAX_ATTEMPTS = 20;
/** How long to wait before declaring standalone mode. */
export const CONTEXT_TIMEOUT_MS = 10_000;

const SS_TOKEN = 'sip-test-bench.token';
const SS_API_BASE = 'sip-test-bench.apiBase';
const SS_USER = 'sip-test-bench.user';

export interface PluginUser {
  id?: string;
  email?: string;
  name?: string;
  role?: string;
  organization_id?: string;
  [key: string]: unknown;
}

export interface PluginTheme {
  mode?: 'light' | 'dark';
  primary?: string;
  [key: string]: unknown;
}

export interface ShellContext {
  token: string;
  apiBase: string;
  user: PluginUser | null;
  theme: PluginTheme | null;
}

export type BridgeState =
  | 'connecting' // hello sent, awaiting context
  | 'connected' // context received
  | 'standalone'; // no context (dev / opened directly)

export type ConnectionState = 'waiting' | 'connecting' | 'connected' | 'standalone';

export interface BridgeSnapshot {
  state: BridgeState;
  connection: ConnectionState;
  context: ShellContext | null;
  /** Non-null explains why we are in standalone mode (or what failed). */
  notice: string | null;
  attempts: number;
}

/** Strip any trailing slash so we can join paths predictably. */
export function normalizeApiBase(base: string): string {
  const trimmed = (base || '').trim();
  if (!trimmed) return '';
  return trimmed.replace(/\/+$/, '');
}

/** Default API base when opened standalone (matches the dev backend default). */
export function defaultApiBase(): string {
  try {
    // If the app is being served by the SIP API itself, same-origin is correct.
    const { origin, pathname } = window.location;
    if (pathname.includes('/api/v1/ui/plugins/')) return origin;
  } catch {
    /* ignore */
  }
  return 'http://localhost:8080';
}

function readSession(key: string): string | null {
  try {
    return window.sessionStorage.getItem(key);
  } catch {
    return null;
  }
}

function writeSession(key: string, value: string | null): void {
  try {
    if (value === null) window.sessionStorage.removeItem(key);
    else window.sessionStorage.setItem(key, value);
  } catch {
    /* sessionStorage can be unavailable in sandboxed frames — non-fatal */
  }
}

export function loadStoredContext(): ShellContext | null {
  const token = readSession(SS_TOKEN);
  if (!token) return null;
  const apiBase = readSession(SS_API_BASE) ?? defaultApiBase();
  let user: PluginUser | null = null;
  const rawUser = readSession(SS_USER);
  if (rawUser) {
    try {
      user = JSON.parse(rawUser) as PluginUser;
    } catch {
      user = null;
    }
  }
  return { token, apiBase, user, theme: null };
}

const DEFAULT_LISTENERS: Array<(snap: BridgeSnapshot) => void> = [];
const DEFAULT_BRIDGE_KEY = Symbol.for('sip-test-bench.bridge');

function isContextMessage(data: unknown): data is {
  type: string;
  pluginId?: string;
  token?: string;
  apiBase?: string;
  user?: PluginUser;
  theme?: PluginTheme;
} {
  return !!data && typeof data === 'object' && (data as { type?: unknown }).type === MESSAGE_CONTEXT;
}

/**
 * Create (once) and return the shared bridge. Multiple React roots / strict-mode
 * double-invocation must not install duplicate listeners or duplicate hello loops.
 */
export function createBridge(): {
  subscribe(fn: (snap: BridgeSnapshot) => void): () => void;
  getSnapshot(): BridgeSnapshot;
  setStandaloneContext(next: { token: string; apiBase: string; user?: PluginUser | null }): void;
  clearStandaloneContext(): void;
  notifyResize(): void;
} {
  const existing = (window as unknown as Record<symbol, unknown>)[DEFAULT_BRIDGE_KEY];
  if (existing) {
    return existing as ReturnType<typeof createBridge>;
  }

  let snapshot: BridgeSnapshot = {
    state: 'connecting',
    connection: 'waiting',
    context: null,
    notice: null,
    attempts: 0,
  };

  const listeners = new Set<(snap: BridgeSnapshot) => void>();
  DEFAULT_LISTENERS.length = 0;

  function emit() {
    for (const fn of listeners) fn(snapshot);
  }

  function setState(patch: Partial<BridgeSnapshot>) {
    snapshot = { ...snapshot, ...patch };
    emit();
  }

  const isEmbedded = (() => {
    try {
      return window.parent !== window;
    } catch {
      // Cross-origin access to window.parent throws in exotic sandboxes; if we
      // cannot tell, assume embedded (we can still post and just get no answer).
      return true;
    }
  })();

  let helloTimer: number | null = null;
  let timeoutTimer: number | null = null;
  let resolved = false;

  function postToParent(message: Record<string, unknown>) {
    if (!isEmbedded) return;
    try {
      // The shell is same-origin under the app's production mount; '*' is the
      // only targetOrigin that works while the shell's origin is unknown.
      window.parent.postMessage(message, '*');
    } catch {
      /* ignore: parent may be gone */
    }
  }

  function stopHello() {
    if (helloTimer !== null) {
      window.clearInterval(helloTimer);
      helloTimer = null;
    }
  }

  function acceptContext(ctx: ShellContext) {
    resolved = true;
    stopHello();
    if (timeoutTimer !== null) {
      window.clearTimeout(timeoutTimer);
      timeoutTimer = null;
    }
    setState({
      state: 'connected',
      connection: 'connected',
      context: ctx,
      notice: null,
    });
    if (ctx.theme?.mode) {
      document.documentElement.dataset.theme = ctx.theme.mode;
    }
    postToParent({ type: MESSAGE_READY, pluginId: PLUGIN_ID });
    notifyResize();
  }

  function onMessage(event: MessageEvent) {
    const data = event.data;
    if (!isContextMessage(data)) return;
    // Reject context addressed to a different plugin sharing the same window.
    if (data.pluginId !== undefined && data.pluginId !== PLUGIN_ID) return;
    if (typeof data.token !== 'string' || !data.token) {
      setState({ notice: 'Shell sent a context message without a token; ignoring it.' });
      return;
    }
    const apiBase = normalizeApiBase(data.apiBase || defaultApiBase());
    acceptContext({
      token: data.token,
      apiBase,
      user: data.user ?? null,
      theme: data.theme ?? null,
    });
  }

  window.addEventListener('message', onMessage);

  // ── Startup: hello loop ────────────────────────────────────────────────
  if (!isEmbedded) {
    // Opened directly (dev server / static host) — no shell to talk to.
    const stored = loadStoredContext();
    setState({
      state: 'standalone',
      connection: 'standalone',
      context: stored,
      notice: 'Not running inside the SIP shell (no parent frame). Standalone mode.',
    });
  } else {
    setState({ connection: 'connecting' });
    let attempts = 0;
    const sendHello = () => {
      attempts += 1;
      postToParent({ type: MESSAGE_HELLO, pluginId: PLUGIN_ID });
      setState({ attempts });
      if (attempts >= HELLO_MAX_ATTEMPTS) stopHello();
    };
    sendHello();
    helloTimer = window.setInterval(sendHello, HELLO_INTERVAL_MS);

    timeoutTimer = window.setTimeout(() => {
      if (resolved) return;
      stopHello();
      const stored = loadStoredContext();
      setState({
        state: 'standalone',
        connection: 'standalone',
        context: stored,
        notice:
          'No context received from the SIP shell within ~10s. Running in standalone/dev mode — enter a token and API base to make requests by hand.',
      });
    }, CONTEXT_TIMEOUT_MS);
  }

  // ── Resize reporting ───────────────────────────────────────────────────
  let lastHeight = -1;
  let resizeQueued = false;
  function notifyResize() {
    try {
      const height = Math.max(
        document.documentElement.scrollHeight,
        document.body ? document.body.scrollHeight : 0,
      );
      if (height === lastHeight) return;
      lastHeight = height;
      postToParent({ type: MESSAGE_RESIZE, pluginId: PLUGIN_ID, height });
    } catch {
      /* ignore */
    }
  }

  const scheduleResize = () => {
    if (resizeQueued) return;
    resizeQueued = true;
    window.requestAnimationFrame(() => {
      resizeQueued = false;
      notifyResize();
    });
  };

  if (typeof ResizeObserver !== 'undefined' && document.body) {
    const observer = new ResizeObserver(scheduleResize);
    observer.observe(document.documentElement);
    observer.observe(document.body);
  }
  window.addEventListener('resize', scheduleResize);
  // Content can grow without the observed boxes changing (e.g. a modal portal);
  // a slow poll is cheap and keeps the shell's frame height honest.
  window.setInterval(scheduleResize, 1000);

  const bridge = {
    subscribe(fn: (snap: BridgeSnapshot) => void) {
      listeners.add(fn);
      fn(snapshot);
      return () => listeners.delete(fn);
    },
    getSnapshot() {
      return snapshot;
    },
    setStandaloneContext(next: { token: string; apiBase: string; user?: PluginUser | null }) {
      const ctx: ShellContext = {
        token: next.token,
        apiBase: normalizeApiBase(next.apiBase || defaultApiBase()),
        user: next.user ?? null,
        theme: null,
      };
      writeSession(SS_TOKEN, ctx.token);
      writeSession(SS_API_BASE, ctx.apiBase);
      writeSession(SS_USER, ctx.user ? JSON.stringify(ctx.user) : null);
      setState({ context: ctx, connection: 'standalone', state: 'standalone', notice: null });
    },
    clearStandaloneContext() {
      writeSession(SS_TOKEN, null);
      writeSession(SS_API_BASE, null);
      writeSession(SS_USER, null);
      setState({ context: null, notice: 'Standalone credentials cleared.' });
    },
    notifyResize,
  };

  (window as unknown as Record<symbol, unknown>)[DEFAULT_BRIDGE_KEY] = bridge;
  return bridge;
}
