import { useCallback, useEffect, useMemo, useState } from 'react';
import type { BenchSession } from '../App';
import { api, asArray, errorMessage, unwrapData, type ApiResponse } from '../api';
import { Banner, Card, Pill, RawJson, Stat } from '../ui';

interface FixtureDataset {
  object_type: string;
  file: string;
  count: number;
  order: number;
}

interface FixtureIndex {
  source?: string;
  generated_at?: string;
  datasets: FixtureDataset[];
}

interface CheckOutcome {
  /** Did the HTTP call itself succeed (transport + 2xx)? */
  httpOk: boolean;
  /** Did the assertion hold? */
  passed: boolean;
  detail: string;
}

interface CheckDef {
  id: string;
  title: string;
  method: string;
  path: string;
  assertion: string;
  evaluate: (res: ApiResponse) => CheckOutcome;
}

interface CheckResult {
  id: string;
  title: string;
  assertion: string;
  method: string;
  path: string;
  res: ApiResponse;
  outcome: CheckOutcome;
}

function httpFailed(res: ApiResponse): CheckOutcome | null {
  if (res.error) {
    return { httpOk: false, passed: false, detail: `transport error: ${res.error}` };
  }
  if (!res.ok) {
    return {
      httpOk: false,
      passed: false,
      detail: `HTTP ${res.status} ${res.statusText} — ${errorMessage(res.json, res.text.slice(0, 400) || 'no body')}`,
    };
  }
  return null;
}

function countOf(res: ApiResponse): number | null {
  const data = unwrapData(res.json);
  return Array.isArray(data) ? data.length : null;
}

function recordOf(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

function fixtureUrl(file: string): string {
  return new URL(`fixtures/${file}`, document.baseURI).toString();
}

export default function Checks({ session }: { session: BenchSession }) {
  const [running, setRunning] = useState(false);
  const [freshBusy, setFreshBusy] = useState(false);
  const [results, setResults] = useState<CheckResult[]>([]);
  const [fixture, setFixture] = useState<FixtureIndex | null>(null);
  const [fixtureError, setFixtureError] = useState<string | null>(null);

  const loadFixture = useCallback(async () => {
    try {
      const res = await fetch(fixtureUrl('index.json'), { headers: { Accept: 'application/json' } });
      if (!res.ok) {
        setFixture(null);
        setFixtureError(`GET ${fixtureUrl('index.json')} → HTTP ${res.status}`);
        return;
      }
      const parsed = (await res.json()) as FixtureIndex;
      setFixture(Array.isArray(parsed.datasets) ? parsed : null);
      setFixtureError(Array.isArray(parsed.datasets) ? null : 'index.json has no datasets array');
    } catch (err) {
      setFixture(null);
      setFixtureError(`GET ${fixtureUrl('index.json')} failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, []);

  useEffect(() => {
    void loadFixture();
  }, [loadFixture]);

  const checks: CheckDef[] = useMemo(
    () => [
      {
        id: 'health',
        title: 'API health endpoint responds',
        method: 'GET',
        path: '/api/v1/health',
        assertion: 'HTTP 200 and a JSON body',
        evaluate: (res) => {
          const failed = httpFailed(res);
          if (failed) return failed;
          return {
            httpOk: true,
            passed: res.json !== null,
            detail: res.json !== null ? 'health payload parsed as JSON' : 'body was not JSON',
          };
        },
      },
      {
        id: 'capabilities',
        title: 'Capabilities are advertised',
        method: 'GET',
        path: '/api/v1/capabilities',
        assertion: 'HTTP 200 with a data payload (object or array)',
        evaluate: (res) => {
          const failed = httpFailed(res);
          if (failed) return failed;
          const data = unwrapData(res.json);
          const present = data !== null && data !== undefined;
          return {
            httpOk: true,
            passed: present,
            detail: present
              ? `capabilities payload present (${Array.isArray(data) ? `${data.length} entries` : 'object'})`
              : 'no data field in the response',
          };
        },
      },
      {
        id: 'ui-plugins',
        title: 'sip-test-bench is a registered UI plugin with a mounted route',
        method: 'GET',
        path: '/api/v1/ui/plugins',
        assertion:
          'a plugin with id "sip-test-bench" is listed, and it exposes ui.routes containing a route whose mount points at its built app',
        evaluate: (res) => {
          const failed = httpFailed(res);
          if (failed) return failed;
          const plugins = asArray(res.json).map((row) => recordOf(row)).filter((r): r is Record<string, unknown> => r !== null);
          const self = plugins.find((p) => p.id === 'sip-test-bench');
          if (!self) {
            return {
              httpOk: true,
              passed: false,
              detail: `plugin list has ${plugins.length} entr${plugins.length === 1 ? 'y' : 'ies'} and none is sip-test-bench (ids: ${plugins
                .map((p) => String(p.id))
                .join(', ') || 'none'})`,
            };
          }
          const ui = recordOf(self.ui);
          if (!ui) {
            return { httpOk: true, passed: false, detail: 'plugin entry is present but has no ui object' };
          }
          const routes = Array.isArray(ui.routes) ? ui.routes : [];
          if (routes.length === 0) {
            return {
              httpOk: true,
              passed: false,
              detail: 'plugin entry is present with ui, but ui.routes is empty — the shell cannot route to it',
            };
          }
          const mounted = routes
            .map((r) => recordOf(r))
            .filter((r): r is Record<string, unknown> => r !== null)
            .filter((r) => typeof r.mount === 'string' && (r.mount as string).length > 0);
          if (mounted.length === 0) {
            return {
              httpOk: true,
              passed: false,
              detail: `ui.routes has ${routes.length} route(s) but none declares a mount`,
            };
          }
          return {
            httpOk: true,
            passed: true,
            detail: `${routes.length} route(s); mount(s): ${mounted.map((r) => String(r.mount)).join(', ')}`,
          };
        },
      },
      {
        id: 'ui-navigation',
        title: 'Sidebar navigation is aggregated',
        method: 'GET',
        path: '/api/v1/ui/navigation',
        assertion: 'HTTP 200 with a navigation payload',
        evaluate: (res) => {
          const failed = httpFailed(res);
          if (failed) return failed;
          const nav = unwrapData(res.json);
          const entries = Array.isArray(nav) ? nav : null;
          return {
            httpOk: true,
            passed: nav !== null && nav !== undefined,
            detail: entries
              ? `${entries.length} navigation item(s)`
              : 'navigation payload present (non-array shape)',
          };
        },
      },
      {
        id: 'assets',
        title: 'Assets are readable',
        method: 'GET',
        path: '/api/v1/assets',
        assertion: 'HTTP 200 with a data[] array',
        evaluate: (res) => {
          const failed = httpFailed(res);
          if (failed) return failed;
          const count = countOf(res);
          return {
            httpOk: true,
            passed: count !== null,
            detail: count === null ? 'response had no data[] array' : `${count} asset(s) visible to this identity`,
          };
        },
      },
      {
        id: 'work-orders',
        title: 'Work orders are readable',
        method: 'GET',
        path: '/api/v1/work-orders',
        assertion: 'HTTP 200 with a data[] array',
        evaluate: (res) => {
          const failed = httpFailed(res);
          if (failed) return failed;
          const count = countOf(res);
          return {
            httpOk: true,
            passed: count !== null,
            detail: count === null ? 'response had no data[] array' : `${count} work order(s) visible to this identity`,
          };
        },
      },
      {
        id: 'parts',
        title: 'Parts are readable',
        method: 'GET',
        path: '/api/v1/parts',
        assertion: 'HTTP 200 with a data[] array',
        evaluate: (res) => {
          const failed = httpFailed(res);
          if (failed) return failed;
          const count = countOf(res);
          return {
            httpOk: true,
            passed: count !== null,
            detail: count === null ? 'response had no data[] array' : `${count} part(s) visible to this identity`,
          };
        },
      },
    ],
    [],
  );

  const runChecks = useCallback(async () => {
    setRunning(true);
    setResults([]);
    const collected: CheckResult[] = [];
    for (const check of checks) {
      const res = await api(check.path, { method: check.method, token: session.token, apiBase: session.apiBase });
      const outcome = check.evaluate(res);
      const result: CheckResult = {
        id: check.id,
        title: check.title,
        assertion: check.assertion,
        method: check.method,
        path: check.path,
        res,
        outcome,
      };
      collected.push(result);
      setResults([...collected]);
    }
    setRunning(false);
  }, [checks, session.token, session.apiBase]);

  // ── Fixture / migration checks run alongside the HTTP battery ────────────

  const [migrations, setMigrations] = useState<ApiResponse | null>(null);
  const [entityCounts, setEntityCounts] = useState<
    Array<{ entity: string; fixtureType: string; res: ApiResponse; count: number | null }>
  >([]);

  const runFixtureChecks = useCallback(async () => {
    setFreshBusy(true);
    const mig = await api('/api/v1/migrations', { token: session.token, apiBase: session.apiBase });
    setMigrations(mig);

    const targets = [
      { entity: 'locations', fixtureType: 'location', path: '/api/v1/locations' },
      { entity: 'assets', fixtureType: 'asset', path: '/api/v1/assets' },
      { entity: 'parts', fixtureType: 'part', path: '/api/v1/parts' },
      { entity: 'work orders', fixtureType: 'work_order', path: '/api/v1/work-orders' },
    ];
    const rows: typeof entityCounts = [];
    for (const t of targets) {
      const res = await api(t.path, { token: session.token, apiBase: session.apiBase });
      const count = res.ok && !res.error ? countOf(res) : null;
      rows.push({ entity: t.entity, fixtureType: t.fixtureType, res, count });
      setEntityCounts([...rows]);
    }
    setFreshBusy(false);
  }, [session.token, session.apiBase]);

  const runEverything = useCallback(async () => {
    await loadFixture();
    await runChecks();
    await runFixtureChecks();
  }, [loadFixture, runChecks, runFixtureChecks]);

  const passed = results.filter((r) => r.outcome.passed).length;
  const failed = results.length - passed;

  const migrationJobs = migrations ? asArray(migrations.json).map(recordOf).filter((r): r is Record<string, unknown> => r !== null) : [];
  const jobStatus = (job: Record<string, unknown>): string =>
    typeof job.status === 'string' ? job.status.toLowerCase() : '';
  const completedJobs = migrationJobs.filter((j) =>
    ['completed', 'imported', 'done', 'applied'].includes(jobStatus(j)),
  );
  const importedTotals = migrationJobs.reduce<{ imported: number }>(
    (acc, j) => {
      const imported = typeof j.imported_record_count === 'number' ? j.imported_record_count : 0;
      acc.imported += imported;
      return acc;
    },
    { imported: 0 },
  );

  const fixtureByType = new Map<string, number>();
  fixture?.datasets.forEach((d) => fixtureByType.set(d.object_type, d.count));

  return (
    <div>
      <Card
        title="Read-only checks"
        lead="Each check issues one GET against the running API and asserts something specific about the response. The HTTP status, latency and raw body are kept as evidence — a FAIL is shown with the exact response that caused it, never summarised away."
        actions={
          <button className="primary" onClick={() => void runEverything()} disabled={running}>
            {running ? <span className="spinner" /> : null} Run checks
          </button>
        }
      >
        {results.length > 0 ? (
          <div className="stat-row">
            <Stat label="checks run" value={results.length} />
            <Stat label="passed" value={passed} />
            <Stat label="failed" value={failed} />
          </div>
        ) : null}

        {!session.token ? (
          <Banner kind="warn" title="No bearer token">
            <p>
              Checks that hit authenticated endpoints will FAIL with HTTP 401. That is a real result and
              is reported as one — supply a token to make the battery meaningful.
            </p>
          </Banner>
        ) : null}

        {results.length === 0 ? (
          <p className="muted">{running ? 'Running…' : 'No checks run yet.'}</p>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Check</th>
                  <th>Result</th>
                  <th>Evidence</th>
                </tr>
              </thead>
              <tbody>
                {results.map((r) => (
                  <tr key={r.id}>
                    <td>
                      <div>{r.title}</div>
                      <div className="small muted mono">
                        {r.method} {r.path}
                      </div>
                    </td>
                    <td>
                      <Pill kind={r.outcome.passed ? 'pass' : 'fail'}>
                        {r.outcome.passed ? 'pass' : 'fail'}
                      </Pill>
                      <div className="small muted" style={{ marginTop: 4 }}>
                        HTTP {r.res.error ? '—' : r.res.status} · {r.res.elapsedMs} ms
                      </div>
                    </td>
                    <td className="small">
                      <div>assertion: {r.assertion}</div>
                      <div style={{ color: r.outcome.passed ? 'var(--success)' : 'var(--danger)' }}>
                        {r.outcome.detail}
                      </div>
                      <RawJson value={r.res.json} text={r.res.text} summary="Raw response" />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </Card>

      <Card
        title="Migration jobs"
        lead="GET /api/v1/migrations — the fixture check asks whether any job has reached a completed state, which is what a successful fixture load looks like."
        actions={
          <button onClick={() => void runFixtureChecks()} disabled={freshBusy}>
            {freshBusy ? <span className="spinner" /> : null} Refresh
          </button>
        }
      >
        {migrations === null ? (
          <p className="muted">Not loaded yet — run the checks.</p>
        ) : migrations.error || !migrations.ok ? (
          <Banner kind="error" title={`GET /api/v1/migrations → HTTP ${migrations.status}`}>
            <p>{errorMessage(migrations.json, migrations.text || 'no body')}</p>
            <RawJson value={migrations.json} text={migrations.text} summary="Raw response" open />
          </Banner>
        ) : (
          <>
            <div className="stat-row">
              <Stat label="jobs" value={migrationJobs.length} />
              <Stat label="completed" value={completedJobs.length} />
              <Stat label="records imported (sum)" value={importedTotals.imported} />
            </div>
            {migrationJobs.length === 0 ? (
              <p className="muted">
                The API reports zero migration jobs — the fixture has not been loaded through the
                migration pipeline yet (see Datasets → Load into SIP).
              </p>
            ) : (
              <div className="table-wrap">
                <table>
                  <thead>
                    <tr>
                      <th>Job</th>
                      <th>Object type</th>
                      <th>Status</th>
                      <th className="num">Source</th>
                      <th className="num">Valid</th>
                      <th className="num">Imported</th>
                      <th className="num">Errors</th>
                    </tr>
                  </thead>
                  <tbody>
                    {migrationJobs.map((j, i) => (
                      <tr key={String(j.id ?? i)}>
                        <td>
                          <code className="small">{String(j.id ?? '')}</code>
                          {typeof j.name === 'string' ? (
                            <div className="small muted">{j.name}</div>
                          ) : null}
                        </td>
                        <td>
                          <code className="small">{String(j.source_object_type ?? '—')}</code>
                        </td>
                        <td>
                          <Pill
                            kind={
                              ['completed', 'imported', 'done', 'applied'].includes(jobStatus(j))
                                ? 'pass'
                                : ['failed', 'error'].includes(jobStatus(j))
                                  ? 'fail'
                                  : ['running', 'in_progress'].includes(jobStatus(j))
                                    ? 'run'
                                    : 'idle'
                            }
                          >
                            {String(j.status ?? 'unknown')}
                          </Pill>
                        </td>
                        <td className="num">{String(j.source_record_count ?? '—')}</td>
                        <td className="num">{String(j.valid_record_count ?? '—')}</td>
                        <td className="num">{String(j.imported_record_count ?? '—')}</td>
                        <td className="num">{String(j.error_count ?? '—')}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
            <RawJson value={migrations.json} summary="Raw /api/v1/migrations response" />
          </>
        )}
      </Card>

      <Card
        title="Fixture vs entity counts"
        lead="Counts are the length of each endpoint's data[] array — no pagination is applied by SIP's list handlers today, so the array length is the row count. Fixture numbers come from the shipped fixtures/index.json."
      >
        {fixtureError ? (
          <Banner kind="error" title="Fixture index unavailable">
            <p>{fixtureError}</p>
          </Banner>
        ) : null}

        {entityCounts.length === 0 ? (
          <p className="muted">Not loaded yet — run the checks.</p>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Entity</th>
                  <th className="num">Live count</th>
                  <th className="num">Fixture</th>
                  <th>Verdict</th>
                </tr>
              </thead>
              <tbody>
                {entityCounts.map((row) => {
                  const fixtureCount = fixtureByType.get(row.fixtureType);
                  const verdict =
                    row.count === null
                      ? 'unavailable'
                      : fixtureCount === undefined
                        ? 'no fixture count'
                        : row.count === fixtureCount
                          ? 'lines up'
                          : `differs by ${row.count - fixtureCount}`;
                  return (
                    <tr key={row.entity}>
                      <td>
                        {row.entity}
                        <div className="small muted mono">{row.res.url}</div>
                      </td>
                      <td className="num">
                        {row.count === null ? (
                          <Pill kind="fail">{row.res.error ? 'transport' : row.res.status}</Pill>
                        ) : (
                          row.count
                        )}
                      </td>
                      <td className="num">{fixtureCount ?? '—'}</td>
                      <td>
                        <Pill
                          kind={
                            row.count === null
                              ? 'fail'
                              : fixtureCount !== undefined && row.count === fixtureCount
                                ? 'pass'
                                : 'warn'
                          }
                        >
                          {verdict}
                        </Pill>
                        {row.count === null ? (
                          <div className="small" style={{ color: 'var(--danger)' }}>
                            {row.res.error
                              ? row.res.error
                              : `HTTP ${row.res.status} — ${errorMessage(row.res.json, 'no body')}`}
                          </div>
                        ) : null}
                        <RawJson value={row.res.json} text={row.res.text} summary="Raw response" />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}

        {fixture ? (
          <p className="small muted" style={{ marginTop: 8 }}>
            Fixture source: <code>{fixture.source ?? 'undeclared'}</code>; index generated{' '}
            {fixture.generated_at ? new Date(fixture.generated_at).toLocaleString() : 'unknown'}.
          </p>
        ) : null}
      </Card>
    </div>
  );
}
