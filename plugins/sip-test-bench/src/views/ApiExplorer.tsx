import { useCallback, useEffect, useMemo, useState } from 'react';
import type { BenchSession } from '../App';
import { api, errorMessage, unwrapData, type ApiResponse } from '../api';
import { Banner, Card, HttpMeta, Pill, RawJson } from '../ui';

interface EndpointDef {
  group: string;
  method: string;
  path: string;
  note?: string;
}

/** Curated read/write surface of the running SIP API (paths taken from the Axum router). */
const ENDPOINTS: EndpointDef[] = [
  { group: 'System', method: 'GET', path: '/api/v1/health' },
  { group: 'System', method: 'GET', path: '/api/v1/health/ready' },
  { group: 'System', method: 'GET', path: '/api/v1/capabilities' },
  { group: 'System', method: 'GET', path: '/api/v1/ui/capabilities' },
  { group: 'System', method: 'GET', path: '/api/v1/ui/theme' },
  { group: 'System', method: 'GET', path: '/api/v1/ui/extension-points' },

  { group: 'Plugins', method: 'GET', path: '/api/v1/plugins' },
  { group: 'Plugins', method: 'GET', path: '/api/v1/plugins/enabled' },
  { group: 'Plugins', method: 'GET', path: '/api/v1/plugins/sip-test-bench' },
  { group: 'Plugins', method: 'GET', path: '/api/v1/ui/plugins' },
  { group: 'Plugins', method: 'GET', path: '/api/v1/ui/navigation' },

  { group: 'Auth', method: 'GET', path: '/api/v1/auth/me' },
  {
    group: 'Auth',
    method: 'POST',
    path: '/api/v1/auth/login',
    note: 'body: {"email":"<an account in this tenant>","password":"<its password>"}',
  },
  { group: 'Auth', method: 'POST', path: '/api/v1/auth/refresh' },
  { group: 'Auth', method: 'POST', path: '/api/v1/auth/logout' },

  { group: 'Assets', method: 'GET', path: '/api/v1/assets' },
  { group: 'Assets', method: 'GET', path: '/api/v1/assets/{id}' },
  { group: 'Assets', method: 'GET', path: '/api/v1/assets/{id}/children' },
  { group: 'Assets', method: 'GET', path: '/api/v1/assets/{id}/work-orders' },
  { group: 'Assets', method: 'POST', path: '/api/v1/assets' },
  { group: 'Assets', method: 'POST', path: '/api/v1/assets/{id}/archive' },
  { group: 'Assets', method: 'GET', path: '/api/v1/asset-types' },
  { group: 'Assets', method: 'GET', path: '/api/v1/asset-types/{id}' },
  { group: 'Assets', method: 'GET', path: '/api/v1/manufacturers' },
  { group: 'Assets', method: 'GET', path: '/api/v1/manufacturers/{id}/models' },
  { group: 'Assets', method: 'GET', path: '/api/v1/models' },
  { group: 'Assets', method: 'GET', path: '/api/v1/models/{id}' },

  { group: 'Locations', method: 'GET', path: '/api/v1/locations' },
  { group: 'Locations', method: 'GET', path: '/api/v1/locations/{id}' },
  { group: 'Locations', method: 'GET', path: '/api/v1/locations/{id}/assets' },
  { group: 'Locations', method: 'GET', path: '/api/v1/locations/{id}/children' },

  { group: 'Work orders', method: 'GET', path: '/api/v1/work-orders' },
  { group: 'Work orders', method: 'GET', path: '/api/v1/work-orders/{id}' },
  { group: 'Work orders', method: 'GET', path: '/api/v1/work-orders/{id}/assignments' },
  { group: 'Work orders', method: 'GET', path: '/api/v1/work-orders/{id}/parts' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/start' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/complete' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/close' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/hold' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/resume' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/cancel' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/reopen' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/review' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/publish' },
  { group: 'Work orders', method: 'POST', path: '/api/v1/work-orders/{id}/archive' },

  { group: 'Parts', method: 'GET', path: '/api/v1/parts' },
  { group: 'Parts', method: 'GET', path: '/api/v1/parts/{id}' },

  { group: 'Scheduling', method: 'GET', path: '/api/v1/schedules' },
  { group: 'Scheduling', method: 'GET', path: '/api/v1/schedules/{id}' },
  { group: 'Scheduling', method: 'POST', path: '/api/v1/schedules/{id}/archive' },
  { group: 'Scheduling', method: 'GET', path: '/api/v1/inspections' },
  { group: 'Scheduling', method: 'GET', path: '/api/v1/inspections/{id}' },
  { group: 'Scheduling', method: 'GET', path: '/api/v1/inspections/{id}/items' },

  { group: 'Documents & activity', method: 'GET', path: '/api/v1/documents' },
  { group: 'Documents & activity', method: 'GET', path: '/api/v1/documents/{id}' },
  { group: 'Documents & activity', method: 'POST', path: '/api/v1/documents/{id}/archive' },
  { group: 'Documents & activity', method: 'GET', path: '/api/v1/activities' },
  { group: 'Documents & activity', method: 'GET', path: '/api/v1/activities/{entity_type}/{entity_id}' },

  { group: 'Org', method: 'GET', path: '/api/v1/organizations' },
  { group: 'Org', method: 'GET', path: '/api/v1/organizations/me' },
  { group: 'Org', method: 'GET', path: '/api/v1/users' },
  { group: 'Org', method: 'GET', path: '/api/v1/users/{id}' },
  { group: 'Org', method: 'GET', path: '/api/v1/teams' },
  { group: 'Org', method: 'GET', path: '/api/v1/teams/{id}' },

  { group: 'Migrations', method: 'GET', path: '/api/v1/migrations' },
  {
    group: 'Migrations',
    method: 'POST',
    path: '/api/v1/migrations',
    note: 'body: {"name":..,"source_system":..,"source_object_type":..,"description":..}',
  },
  { group: 'Migrations', method: 'GET', path: '/api/v1/migrations/{job_id}' },
  { group: 'Migrations', method: 'POST', path: '/api/v1/migrations/{job_id}/source-records' },
  { group: 'Migrations', method: 'GET', path: '/api/v1/migrations/{job_id}/source-records' },
  { group: 'Migrations', method: 'POST', path: '/api/v1/migrations/{job_id}/mappings' },
  { group: 'Migrations', method: 'GET', path: '/api/v1/migrations/{job_id}/mappings' },
  { group: 'Migrations', method: 'POST', path: '/api/v1/migrations/{job_id}/validate' },
  { group: 'Migrations', method: 'GET', path: '/api/v1/migrations/{job_id}/validation-issues' },
  { group: 'Migrations', method: 'POST', path: '/api/v1/migrations/{job_id}/dry-run' },
  { group: 'Migrations', method: 'POST', path: '/api/v1/migrations/{job_id}/import' },
  { group: 'Migrations', method: 'POST', path: '/api/v1/migrations/{job_id}/rollback' },
  { group: 'Migrations', method: 'POST', path: '/api/v1/migrations/{job_id}/cancel' },
  { group: 'Migrations', method: 'GET', path: '/api/v1/migrations/{job_id}/report' },
  { group: 'Migrations', method: 'GET', path: '/api/v1/migrations/{job_id}/duplicates' },
  { group: 'Migrations', method: 'GET', path: '/api/v1/migrations/{job_id}/staged-records' },
  { group: 'Migrations', method: 'GET', path: '/api/v1/migrations/{job_id}/external-id-maps' },

  { group: 'AI', method: 'GET', path: '/api/v1/ai/conversations' },
  { group: 'AI', method: 'GET', path: '/api/v1/ai/conversations/{id}' },
  { group: 'AI', method: 'GET', path: '/api/v1/ai/messages/{id}/retrieval-trace' },
  { group: 'AI', method: 'GET', path: '/api/v1/ai/messages/{id}/verification-trace' },
  { group: 'AI', method: 'POST', path: '/api/v1/ai/messages/{id}/feedback' },
  {
    group: 'AI',
    method: 'POST',
    path: '/api/v1/ai/chat',
    note: 'SSE stream — use the Query Console for a readable view of it; this pane shows the raw bytes',
  },

  { group: 'Export', method: 'GET', path: '/api/v1/export' },
  { group: 'Export', method: 'GET', path: '/api/v1/export/work-orders' },
];

const METHODS = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE'];


interface LogEntry {
  id: number;
  res: ApiResponse;
  tokenLabel: string;
}

let logId = 0;

export default function ApiExplorer({
  session,
  onUseToken,
}: {
  session: BenchSession;
  onUseToken: (token: string, label: string) => void;
}) {
  const [method, setMethod] = useState('GET');
  const [path, setPath] = useState('/api/v1/health');
  const [body, setBody] = useState('');
  const [bodyError, setBodyError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const [log, setLog] = useState<LogEntry[]>([]);

  const [loginEmail, setLoginEmail] = useState('');
  const [loginPassword, setLoginPassword] = useState('');
  // Whether this deployment advertises itself as a development/demo environment.
  // null = not yet known, so the affordance stays hidden until the API says so.
  const [demoMode, setDemoMode] = useState<boolean | null>(null);
  // Identities offered as suggestions, read from the tenant the current token
  // belongs to. Deliberately not hardcoded: an account list compiled into the
  // plugin would name one tenant's users in a public repository.
  const [identities, setIdentities] = useState<string[]>([]);
  const [loginBusy, setLoginBusy] = useState(false);
  const [loginResult, setLoginResult] = useState<ApiResponse | null>(null);
  const [loginToken, setLoginToken] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    // Capability discovery decides whether test-only affordances belong here at
    // all. The server is the authority; this only mirrors what it advertises.
    void api('/api/v1/capabilities', { token: session.token, apiBase: session.apiBase }).then((res) => {
      if (cancelled) return;
      const data = unwrapData(res.json) as { features?: { demo_mode?: unknown } } | null;
      setDemoMode(data?.features?.demo_mode === true);
    });
    void api('/api/v1/users', { token: session.token, apiBase: session.apiBase }).then((res) => {
      if (cancelled) return;
      const data = unwrapData(res.json);
      const list = Array.isArray(data) ? data : [];
      setIdentities(
        list
          .map((u) => (u && typeof u === 'object' ? (u as { email?: unknown }).email : null))
          .filter((e): e is string => typeof e === 'string' && e.length > 0)
          .sort(),
      );
    });
    return () => {
      cancelled = true;
    };
  }, [session.token, session.apiBase]);

  const groups = useMemo(() => {
    const map = new Map<string, EndpointDef[]>();
    for (const ep of ENDPOINTS) {
      const existing = map.get(ep.group);
      if (existing) existing.push(ep);
      else map.set(ep.group, [ep]);
    }
    return [...map.entries()];
  }, []);

  const send = useCallback(async () => {
    let parsedBody: unknown;
    setBodyError(null);
    const isWrite = method !== 'GET' && method !== 'HEAD';
    if (isWrite && body.trim()) {
      try {
        parsedBody = JSON.parse(body);
      } catch (err) {
        // A malformed body is the user's typo, not an API result: refuse to send
        // rather than silently posting something else.
        setBodyError(`Request body is not valid JSON: ${err instanceof Error ? err.message : String(err)}`);
        return;
      }
    }
    setSending(true);
    const res = await api(path, {
      method,
      token: session.token,
      apiBase: session.apiBase,
      body: isWrite && parsedBody !== undefined ? parsedBody : undefined,
    });
    setSending(false);
    logId += 1;
    setLog((prev) => [{ id: logId, res, tokenLabel: session.identity }, ...prev].slice(0, 40));
  }, [method, path, body, session.token, session.apiBase, session.identity]);

  const doLogin = useCallback(async () => {
    setLoginBusy(true);
    setLoginResult(null);
    setLoginToken(null);
    const res = await api('/api/v1/auth/login', {
      method: 'POST',
      token: null,
      apiBase: session.apiBase,
      body: { email: loginEmail.trim(), password: loginPassword },
    });
    setLoginBusy(false);
    setLoginResult(res);
    const token = (res.json as { token?: unknown } | null)?.token;
    if (res.ok && typeof token === 'string' && token) {
      setLoginToken(token);
    }
  }, [loginEmail, loginPassword, session.apiBase]);

  const selectedEndpoint = ENDPOINTS.find((e) => e.method === method && e.path === path);

  return (
    <div>
      <Card
        title="Request"
        lead="Every path below comes from the SIP Axum router. Path parameters are left as {id} / {job_id} placeholders — edit them in the path field before sending."
      >
        <div className="row">
          <div className="field" style={{ flex: '0 0 130px' }}>
            <label htmlFor="method">Method</label>
            <select id="method" value={method} onChange={(e) => setMethod(e.target.value)}>
              {METHODS.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>
          </div>
          <div className="field grow">
            <label htmlFor="path">Path</label>
            <input
              id="path"
              type="text"
              className="mono"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') void send();
              }}
            />
          </div>
          <button className="primary" onClick={() => void send()} disabled={sending || !path.trim()}>
            {sending ? <span className="spinner" /> : null} Send
          </button>
        </div>

        <div className="field" style={{ marginTop: 12 }}>
          <label htmlFor="endpoint-picker">Pick an endpoint</label>
          <select
            id="endpoint-picker"
            value={selectedEndpoint ? `${selectedEndpoint.method} ${selectedEndpoint.path}` : ''}
            onChange={(e) => {
              const found = ENDPOINTS.find((x) => `${x.method} ${x.path}` === e.target.value);
              if (found) {
                setMethod(found.method);
                setPath(found.path);
              }
            }}
          >
            <option value="">— select —</option>
            {groups.map(([group, eps]) => (
              <optgroup key={group} label={group}>
                {eps.map((ep) => (
                  <option key={`${ep.method} ${ep.path}`} value={`${ep.method} ${ep.path}`}>
                    {ep.method} {ep.path}
                  </option>
                ))}
              </optgroup>
            ))}
          </select>
        </div>

        {selectedEndpoint?.note ? (
          <p className="small muted" style={{ marginTop: 6 }}>
            {selectedEndpoint.note}
          </p>
        ) : null}

        {method !== 'GET' && method !== 'HEAD' ? (
          <div className="field" style={{ marginTop: 12 }}>
            <label htmlFor="body">JSON body</label>
            <textarea
              id="body"
              className="mono"
              rows={6}
              spellCheck={false}
              placeholder='{"example": "value"}'
              value={body}
              onChange={(e) => setBody(e.target.value)}
            />
          </div>
        ) : null}

        {bodyError ? (
          <Banner kind="error" title="Not sent">
            <p>{bodyError}</p>
          </Banner>
        ) : null}

        <p className="small muted" style={{ marginTop: 10 }}>
          Requests go to <code>{session.apiBase}</code> as{' '}
          <strong>{session.identity}</strong>
          {session.token ? <> (token …{session.token.slice(-6)})</> : <> with no token — expect 401</>}.
        </p>
      </Card>

      {demoMode === false ? (
        <Card title="Login as a test identity — disabled">
          <Banner kind="warn" title="This deployment is not a demo environment">
            <p className="small">
              Switching identities here is a test affordance, so it is only offered when the API
              advertises <code>features.demo_mode</code> as true. A shipped build does not, which
              keeps a role picker out of a production interface. The API itself is unchanged — this
              panel simply is not shown.
            </p>
          </Banner>
        </Card>
      ) : (
      <Card
        title="Login as a test identity"
        lead="POST /api/v1/auth/login. On success the returned token can be adopted for every subsequent request in this app, which is how permission and row-level-security behaviour gets compared across roles."
      >
        <div className="row">
          <div className="field grow">
            <label htmlFor="login-email">Identity</label>
            <input
              id="login-email"
              type="text"
              list="login-email-options"
              className="mono"
              placeholder="an account in this tenant"
              value={loginEmail}
              onChange={(e) => setLoginEmail(e.target.value)}
            />
            <datalist id="login-email-options">
              {identities.map((email) => (
                <option key={email} value={email} />
              ))}
            </datalist>
          </div>
          <div className="field grow">
            <label htmlFor="login-password">Password</label>
            <input
              id="login-password"
              type="text"
              className="mono"
              value={loginPassword}
              onChange={(e) => setLoginPassword(e.target.value)}
            />
          </div>
          <button onClick={() => void doLogin()} disabled={loginBusy}>
            {loginBusy ? <span className="spinner" /> : null} Log in
          </button>
          {loginToken ? (
            <button
              className="primary"
              onClick={() => onUseToken(loginToken, `${loginEmail} (logged in via API Explorer)`)}
            >
              Use this token for subsequent requests
            </button>
          ) : null}
        </div>

        {loginResult ? (
          <div style={{ marginTop: 12 }}>
            <HttpMeta
              method={loginResult.method}
              status={loginResult.status}
              ok={loginResult.ok && !loginResult.error}
              elapsedMs={loginResult.elapsedMs}
              url={loginResult.url}
              error={loginResult.error}
            />
            {loginToken ? (
              <Banner kind="info" title={`Logged in as ${loginEmail}`}>
                <p className="small">
                  This token identifies the identity above. Adopt it to send the following requests
                  as that role, which is how permission and row-level-security differences show up.
                </p>
              </Banner>
            ) : (
              <Banner kind="error" title="Login failed">
                <p>{errorMessage(loginResult.json, loginResult.text || 'no error message in body')}</p>
              </Banner>
            )}
            <RawJson value={loginResult.json} text={loginResult.text} summary="Raw login response" />
          </div>
        ) : null}
      </Card>
      )}

      <Card
        title={`Responses (${log.length})`}
        lead="Newest first. Status code, latency and the raw body are shown exactly as received — nothing is parsed away."
        actions={
          log.length > 0 ? (
            <button className="ghost" onClick={() => setLog([])}>
              Clear
            </button>
          ) : null
        }
      >
        {log.length === 0 ? (
          <p className="muted">No requests sent yet.</p>
        ) : (
          log.map((entry) => (
            <div
              key={entry.id}
              style={{
                borderTop: '1px solid var(--border)',
                paddingTop: 12,
                marginTop: 12,
              }}
            >
              <HttpMeta
                method={entry.res.method}
                status={entry.res.status}
                statusText={entry.res.statusText}
                ok={entry.res.ok && !entry.res.error}
                elapsedMs={entry.res.elapsedMs}
                url={entry.res.url}
                error={entry.res.error}
              />
              <div className="small muted" style={{ marginBottom: 6 }}>
                sent as {entry.tokenLabel}
              </div>
              {entry.res.error ? (
                <Banner kind="error" title="Transport failure — no HTTP response">
                  <p>{entry.res.error}</p>
                </Banner>
              ) : null}
              <RawJson
                value={entry.res.json}
                text={entry.res.text || '(empty body)'}
                summary={`Raw body (${entry.res.text.length} bytes)`}
                open
                tall
              />
            </div>
          ))
        )}
      </Card>

      <Card title="Endpoint coverage" lead="The curated list this explorer offers, for reference.">
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Group</th>
                <th>Method</th>
                <th>Path</th>
              </tr>
            </thead>
            <tbody>
              {ENDPOINTS.map((ep) => (
                <tr key={`${ep.method} ${ep.path}`}>
                  <td className="small">{ep.group}</td>
                  <td>
                    <Pill kind={ep.method === 'GET' ? 'idle' : 'warn'}>{ep.method}</Pill>
                  </td>
                  <td>
                    <code className="small">{ep.path}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
}
