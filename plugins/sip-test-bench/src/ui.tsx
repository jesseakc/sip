/**
 * ui.tsx — small shared presentational primitives.
 *
 * Deliberately tiny and dependency-free: the Test Bench renders real API
 * responses, so its own chrome should not be the interesting part.
 */
import type { ReactNode } from 'react';

export function Card({
  title,
  lead,
  children,
  actions,
}: {
  title?: ReactNode;
  lead?: ReactNode;
  children: ReactNode;
  actions?: ReactNode;
}) {
  return (
    <section className="card">
      {(title || actions) && (
        <div className="row tight" style={{ justifyContent: 'space-between', marginBottom: 4 }}>
          {typeof title === 'string' ? <h2>{title}</h2> : title}
          {actions}
        </div>
      )}
      {lead ? <p className="lead">{lead}</p> : null}
      {children}
    </section>
  );
}

export function Banner({
  kind = 'info',
  title,
  children,
}: {
  kind?: 'info' | 'warn' | 'error';
  title?: ReactNode;
  children?: ReactNode;
}) {
  return (
    <div className={`banner ${kind}`}>
      {title ? <strong>{title}</strong> : null}
      {children}
    </div>
  );
}

export type PillKind = 'pass' | 'fail' | 'run' | 'warn' | 'idle' | 'info' | 'skip';

export function Pill({ kind = 'idle', children }: { kind?: PillKind; children: ReactNode }) {
  return <span className={`pill ${kind}`}>{children}</span>;
}

export function statusPillKind(status: string): PillKind {
  const s = status.toLowerCase();
  if (['pass', 'passed', 'ok', 'success', 'completed', 'imported', 'validated', 'done'].includes(s)) {
    return 'pass';
  }
  if (['fail', 'failed', 'error', 'rejected', 'cancelled'].includes(s)) return 'fail';
  if (['running', 'in_progress', 'pending', 'queued', 'started'].includes(s)) return 'run';
  if (['skipped', 'skip', 'partial'].includes(s)) return 'skip';
  if (['warn', 'warning'].includes(s)) return 'warn';
  return 'idle';
}

/**
 * Render a value verbatim. Raw JSON is the product of a test bench, so this
 * pretty-prints what it can and falls back to the exact bytes it was given.
 */
export function RawJson({
  value,
  summary = 'Raw response',
  text,
  tall = false,
  open = false,
}: {
  value?: unknown;
  summary?: string;
  /** Exact raw body to show when it is not JSON (e.g. a 500 HTML page). */
  text?: string;
  tall?: boolean;
  open?: boolean;
}) {
  let body: string;
  if (value !== undefined && value !== null) {
    try {
      body = JSON.stringify(value, null, 2);
    } catch {
      body = String(value);
    }
  } else if (text !== undefined) {
    body = text || '(empty body)';
  } else {
    body = '(no body)';
  }
  return (
    <details className="raw" open={open}>
      <summary>{summary}</summary>
      <pre className={`mono-block${tall ? ' tall' : ''}`}>{body}</pre>
    </details>
  );
}

/** HTTP status + latency line shared by every response pane. */
export function HttpMeta({
  status,
  statusText,
  ok,
  elapsedMs,
  url,
  method,
  error,
}: {
  status: number;
  statusText?: string;
  ok: boolean;
  elapsedMs?: number;
  url?: string;
  method?: string;
  error?: string | null;
}) {
  const kind: PillKind = error ? 'fail' : ok ? 'pass' : 'fail';
  return (
    <div className="row tight" style={{ marginBottom: 6 }}>
      {method ? <Pill kind="idle">{method}</Pill> : null}
      <Pill kind={kind}>
        {error ? 'TRANSPORT ERROR' : `${status}${statusText ? ` ${statusText}` : ''}`}
      </Pill>
      {typeof elapsedMs === 'number' ? <span className="small muted">{elapsedMs} ms</span> : null}
      {error ? <span className="small" style={{ color: 'var(--danger)' }}>{error}</span> : null}
      {url ? <span className="small muted mono">{url}</span> : null}
    </div>
  );
}

export function Stat({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="stat">
      <div className="stat-label">{label}</div>
      <div className="stat-value">{value}</div>
    </div>
  );
}
