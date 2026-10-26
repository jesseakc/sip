/**
 * api.ts — the Test Bench's HTTP layer.
 *
 * Design rule for a *test bench*: the raw response is the product. `api()`
 * therefore never throws on an HTTP error status and never discards the body —
 * callers always get the status code, the raw text, and the parsed JSON (or
 * null when the body is not JSON). Only transport failures (DNS, CORS, abort)
 * are surfaced, and they are surfaced as data (`error`) rather than exceptions.
 */
import { normalizeApiBase } from './sip-bridge';

export interface ApiRequestOptions {
  method?: string;
  body?: unknown;
  token?: string | null;
  apiBase?: string | null;
  headers?: Record<string, string>;
  signal?: AbortSignal;
  /** Query params appended to the path (values are URL-encoded, nullish skipped). */
  query?: Record<string, string | number | boolean | null | undefined>;
}

export interface ApiResponse {
  /** Full URL that was requested. */
  url: string;
  method: string;
  status: number;
  statusText: string;
  ok: boolean;
  /** Wall-clock duration of the request in milliseconds. */
  elapsedMs: number;
  /** Parsed JSON body, or null if the body was empty / not JSON. */
  json: unknown | null;
  /** Raw response body, always preserved. */
  text: string;
  headers: Record<string, string>;
  /** Transport-level failure message (network error, abort). Null on any HTTP response. */
  error: string | null;
}

/** Join an API base and a path without producing '//' or dropping the base path. */
export function joinUrl(apiBase: string | null | undefined, path: string): string {
  const base = normalizeApiBase(apiBase ?? '');
  const p = path.startsWith('/') ? path : `/${path}`;
  if (!base) return p;
  // Call sites write '/api/v1/...' so they also work when the plugin runs
  // standalone against a bare origin. The shell, however, hands over an apiBase
  // that already ends in '/api/v1', which would produce '/api/v1/api/v1/...'.
  // Drop one copy of the prefix when both sides carry it.
  const PREFIX = '/api/v1';
  if (base.endsWith(PREFIX) && (p === PREFIX || p.startsWith(`${PREFIX}/`))) {
    return `${base}${p.slice(PREFIX.length)}`;
  }
  return `${base}${p}`;
}

function withQuery(url: string, query?: ApiRequestOptions['query']): string {
  if (!query) return url;
  const params = new URLSearchParams();
  for (const [k, v] of Object.entries(query)) {
    if (v === undefined || v === null) continue;
    params.append(k, String(v));
  }
  const qs = params.toString();
  if (!qs) return url;
  return url.includes('?') ? `${url}&${qs}` : `${url}?${qs}`;
}

function collectHeaders(res: Response): Record<string, string> {
  const out: Record<string, string> = {};
  res.headers.forEach((value, key) => {
    out[key] = value;
  });
  return out;
}

function nowMs(): number {
  return typeof performance !== 'undefined' ? performance.now() : Date.now();
}

/**
 * Perform a request and return everything about the response.
 * Never rejects: a failed fetch resolves with status 0 and a populated `error`.
 */
export async function api(path: string, options: ApiRequestOptions = {}): Promise<ApiResponse> {
  const method = (options.method ?? 'GET').toUpperCase();
  const url = withQuery(joinUrl(options.apiBase, path), options.query);

  const headers: Record<string, string> = { Accept: 'application/json', ...(options.headers ?? {}) };
  if (options.token) headers.Authorization = `Bearer ${options.token}`;

  let bodyInit: BodyInit | undefined;
  if (options.body !== undefined && options.body !== null && method !== 'GET' && method !== 'HEAD') {
    if (typeof options.body === 'string') {
      bodyInit = options.body;
      if (!headers['Content-Type'] && !headers['content-type']) {
        headers['Content-Type'] = 'application/json';
      }
    } else {
      bodyInit = JSON.stringify(options.body);
      if (!headers['Content-Type'] && !headers['content-type']) {
        headers['Content-Type'] = 'application/json';
      }
    }
  }

  const started = nowMs();
  let res: Response;
  try {
    res = await fetch(url, { method, headers, body: bodyInit, signal: options.signal });
  } catch (err) {
    const elapsedMs = Math.round(nowMs() - started);
    const message =
      err instanceof DOMException && err.name === 'AbortError'
        ? 'Request aborted'
        : err instanceof Error
          ? err.message
          : String(err);
    return {
      url,
      method,
      status: 0,
      statusText: 'FETCH_FAILED',
      ok: false,
      elapsedMs,
      json: null,
      text: '',
      headers: {},
      error: message,
    };
  }

  let text = '';
  try {
    text = await res.text();
  } catch (err) {
    return {
      url,
      method,
      status: res.status,
      statusText: res.statusText,
      ok: res.ok,
      elapsedMs: Math.round(nowMs() - started),
      json: null,
      text: '',
      headers: collectHeaders(res),
      error: `Failed to read response body: ${err instanceof Error ? err.message : String(err)}`,
    };
  }

  const elapsedMs = Math.round(nowMs() - started);
  let json: unknown | null = null;
  if (text.trim()) {
    try {
      json = JSON.parse(text);
    } catch {
      json = null;
    }
  }

  return {
    url,
    method,
    status: res.status,
    statusText: res.statusText,
    ok: res.ok,
    elapsedMs,
    json,
    text,
    headers: collectHeaders(res),
    error: null,
  };
}

export interface SseEvent {
  /** The `event:` field if the server sent one; SIP's chat endpoint only sends data. */
  event: string | null;
  /** The raw `data:` payload string. */
  data: string;
  /** JSON.parse(data) when it parses, else null. */
  json: unknown | null;
}

export interface StreamOptions extends ApiRequestOptions {
  onEvent(event: SseEvent): void;
  /** Called for a non-2xx response or transport error before the stream ends. */
  onHttpError?(response: ApiResponse): void;
}

export interface StreamResult {
  url: string;
  method: string;
  status: number;
  ok: boolean;
  elapsedMs: number;
  /** Raw body when the response was NOT a stream (e.g. a JSON error). */
  text: string;
  json: unknown | null;
  error: string | null;
  events: SseEvent[];
}

/**
 * Open a streaming POST/GET and yield parsed SSE `data:` lines.
 *
 * SSE framing is handled minimally and correctly for the shapes SIP emits:
 * blank-line separated records, `data:` possibly repeated, `:` comment lines
 * ignored. Unknown event types are handed to the caller verbatim — a test bench
 * must show what the server actually said.
 */
export async function streamSse(path: string, options: StreamOptions): Promise<StreamResult> {
  const method = (options.method ?? 'POST').toUpperCase();
  const url = withQuery(joinUrl(options.apiBase, path), options.query);
  const headers: Record<string, string> = {
    Accept: 'text/event-stream',
    ...(options.headers ?? {}),
  };
  if (options.token) headers.Authorization = `Bearer ${options.token}`;

  let bodyInit: BodyInit | undefined;
  if (options.body !== undefined && options.body !== null && method !== 'GET') {
    if (typeof options.body === 'string') {
      bodyInit = options.body;
    } else {
      bodyInit = JSON.stringify(options.body);
    }
    if (!headers['Content-Type'] && !headers['content-type']) {
      headers['Content-Type'] = 'application/json';
    }
  }

  const events: SseEvent[] = [];
  const started = nowMs();

  let res: Response;
  try {
    res = await fetch(url, { method, headers, body: bodyInit, signal: options.signal });
  } catch (err) {
    const message =
      err instanceof DOMException && err.name === 'AbortError'
        ? 'Request aborted'
        : err instanceof Error
          ? err.message
          : String(err);
    return {
      url,
      method,
      status: 0,
      ok: false,
      elapsedMs: Math.round(nowMs() - started),
      text: '',
      json: null,
      error: message,
      events,
    };
  }

  const contentType = res.headers.get('content-type') ?? '';

  if (!res.ok || !contentType.includes('text/event-stream')) {
    // Not a stream: read the whole body so the caller can show the real error.
    let text = '';
    try {
      text = await res.text();
    } catch {
      text = '';
    }
    let json: unknown | null = null;
    if (text.trim()) {
      try {
        json = JSON.parse(text);
      } catch {
        json = null;
      }
    }
    const response: ApiResponse = {
      url,
      method,
      status: res.status,
      statusText: res.statusText,
      ok: res.ok,
      elapsedMs: Math.round(nowMs() - started),
      json,
      text,
      headers: collectHeaders(res),
      error: null,
    };
    options.onHttpError?.(response);
    return {
      url,
      method,
      status: res.status,
      ok: res.ok,
      elapsedMs: response.elapsedMs,
      text,
      json,
      error: null,
      events,
    };
  }

  if (!res.body) {
    return {
      url,
      method,
      status: res.status,
      ok: res.ok,
      elapsedMs: Math.round(nowMs() - started),
      text: '',
      json: null,
      error: 'Response reported content-type text/event-stream but had no readable body.',
      events,
    };
  }

  const reader = res.body.getReader();
  const decoder = new TextDecoder();
  let buffer = '';

  function flushRecord(record: string) {
    if (!record.trim()) return;
    let eventName: string | null = null;
    const dataLines: string[] = [];
    for (const rawLine of record.split('\n')) {
      const line = rawLine.endsWith('\r') ? rawLine.slice(0, -1) : rawLine;
      if (!line || line.startsWith(':')) continue;
      const colon = line.indexOf(':');
      const field = colon === -1 ? line : line.slice(0, colon);
      let value = colon === -1 ? '' : line.slice(colon + 1);
      if (value.startsWith(' ')) value = value.slice(1);
      if (field === 'event') eventName = value;
      else if (field === 'data') dataLines.push(value);
    }
    if (dataLines.length === 0) return;
    const data = dataLines.join('\n');
    let json: unknown | null = null;
    try {
      json = JSON.parse(data);
    } catch {
      json = null;
    }
    const evt: SseEvent = { event: eventName, data, json };
    events.push(evt);
    options.onEvent(evt);
  }

  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      let idx = buffer.indexOf('\n\n');
      while (idx !== -1) {
        const record = buffer.slice(0, idx);
        buffer = buffer.slice(idx + 2);
        flushRecord(record);
        idx = buffer.indexOf('\n\n');
      }
    }
    buffer += decoder.decode();
  } catch (err) {
    const message =
      err instanceof DOMException && err.name === 'AbortError'
        ? 'Stream aborted'
        : err instanceof Error
          ? err.message
          : String(err);
    return {
      url,
      method,
      status: res.status,
      ok: res.ok,
      elapsedMs: Math.round(nowMs() - started),
      text: '',
      json: null,
      error: message,
      events,
    };
  }

  if (buffer.trim()) flushRecord(buffer);

  return {
    url,
    method,
    status: res.status,
    ok: res.ok,
    elapsedMs: Math.round(nowMs() - started),
    text: '',
    json: null,
    error: null,
    events,
  };
}

// ── Response shape helpers ───────────────────────────────────────────────────

/** Unwrap the `{data: ...}` envelope every SIP endpoint uses. */
export function unwrapData(json: unknown): unknown {
  if (json && typeof json === 'object' && 'data' in json) {
    return (json as { data: unknown }).data;
  }
  return json;
}

/** Return `data` when it is an array, else an empty array (never a fake record). */
export function asArray(json: unknown): unknown[] {
  const data = unwrapData(json);
  return Array.isArray(data) ? data : [];
}

/**
 * Find a migration job id in a create response. The API returns
 * `{data:{id}}`; older/other shapes may use `job_id` or nest the payload, so
 * probe the documented candidates and return null (never a guess) otherwise.
 */
export function extractJobId(json: unknown): string | null {
  const candidates: Array<unknown> = [];
  const data = unwrapData(json);
  candidates.push(data, json);
  for (const candidate of candidates) {
    if (!candidate || typeof candidate !== 'object') continue;
    const obj = candidate as Record<string, unknown>;
    for (const key of ['id', 'job_id', 'migration_job_id', 'uuid']) {
      const v = obj[key];
      if (typeof v === 'string' && v.trim()) return v;
    }
    // One more level of nesting, e.g. {data:{job:{id}}}
    for (const key of ['job', 'migration', 'result']) {
      const nested = obj[key];
      if (nested && typeof nested === 'object') {
        const n = nested as Record<string, unknown>;
        for (const idKey of ['id', 'job_id', 'migration_job_id']) {
          const v = n[idKey];
          if (typeof v === 'string' && v.trim()) return v;
        }
      }
    }
  }
  return null;
}

/** Find a message id in an AI stream event / response, if the server provided one. */
export function extractMessageId(...payloads: unknown[]): string | null {
  for (const payload of payloads) {
    if (!payload || typeof payload !== 'object') continue;
    const stack: Array<Record<string, unknown>> = [payload as Record<string, unknown>];
    let depth = 0;
    while (stack.length && depth < 200) {
      depth += 1;
      const obj = stack.pop()!;
      for (const key of ['message_id', 'assistant_message_id', 'id']) {
        const v = obj[key];
        if (typeof v === 'string' && v.trim() && /^[0-9a-f-]{8,}$/i.test(v)) return v;
      }
      for (const key of ['data', 'message', 'assistant_message', 'meta']) {
        const nested = obj[key];
        if (nested && typeof nested === 'object' && !Array.isArray(nested)) {
          stack.push(nested as Record<string, unknown>);
        }
      }
    }
  }
  return null;
}

/** Extract an `error.message` string from a SIP error envelope, verbatim. */
export function errorMessage(json: unknown, fallback: string): string {
  if (json && typeof json === 'object') {
    const obj = json as Record<string, unknown>;
    const err = obj.error;
    if (typeof err === 'string') return err;
    if (err && typeof err === 'object') {
      const m = (err as Record<string, unknown>).message;
      if (typeof m === 'string') return m;
    }
    if (typeof obj.message === 'string') return obj.message;
  }
  return fallback;
}
