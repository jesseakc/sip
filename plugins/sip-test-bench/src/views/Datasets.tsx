import { useCallback, useEffect, useRef, useState } from 'react';
import type { BenchSession } from '../App';
import { api, errorMessage, extractJobId } from '../api';
import { Banner, Card, Pill, RawJson, Stat } from '../ui';

// ── Fixture load plan types ──────────────────────────────────────────────────

interface FixtureDataset {
  object_type: string;
  declared_object_type?: string | null;
  file: string;
  count: number;
  order: number;
  available?: boolean;
  note?: string;
  source_system?: string | null;
  canonical_version?: string | null;
}

interface FixtureIndex {
  source?: string;
  canonical_version?: string;
  generated_at?: string;
  source_dir?: string;
  total_records?: number;
  datasets: FixtureDataset[];
}

// ── Jobs created in this browser session (survives view switches) ────────────

export interface CreatedJob {
  objectType: string;
  objectTypeLabel: string;
  jobId: string;
  name: string;
  createdAt: string;
}

const jobLog: CreatedJob[] = [];

export function getCreatedJobs(): CreatedJob[] {
  return jobLog.slice();
}

// ── Pipeline run bookkeeping ─────────────────────────────────────────────────

type StepName = 'create job' | 'upload records' | 'validate' | 'dry-run' | 'import' | 'report';

interface StepResult {
  name: StepName;
  method: string;
  url: string;
  status: number;
  ok: boolean;
  elapsedMs: number;
  json: unknown;
  text: string;
  error: string | null;
  detail?: string;
}

type RunStatus = 'pending' | 'running' | 'done' | 'failed';

interface ImportCounts {
  created: number | null;
  updated: number | null;
  skipped: number | null;
  failed: number | null;
}

interface DatasetRun {
  objectType: string;
  file: string;
  order: number;
  fixtureCount: number;
  status: RunStatus;
  steps: StepResult[];
  jobId: string | null;
  upload: { chunksSent: number; chunksTotal: number; recordsAccepted: number | null };
  counts: ImportCounts | null;
  failure: string | null;
}

const CHUNK_SIZE = 500;

function labelFor(objectType: string): string {
  return objectType.replace(/_/g, ' ');
}

/** Build the request record: canonical fields + the two transport keys SIP needs. */
function toTransportRecords(
  records: Array<Record<string, unknown>>,
  objectType: string,
): Array<Record<string, unknown>> {
  return records.map((record) => {
    const out: Record<string, unknown> = { ...record, source_object_type: objectType };
    // The canonical fixture records carry external_id already; restate it
    // explicitly (as the pipeline contract requires) and never invent one.
    if (typeof record.external_id === 'string') out.external_id = record.external_id;
    else delete out.external_id;
    return out;
  });
}

function fixtureUrl(file: string): string {
  return new URL(`fixtures/${file}`, document.baseURI).toString();
}

function num(value: unknown): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  return null;
}

function extractCounts(json: unknown): ImportCounts | null {
  const data = (json as { data?: Record<string, unknown> } | null)?.data;
  if (!data || typeof data !== 'object') return null;
  const results = (data as Record<string, unknown>).import_results;
  const source = (results && typeof results === 'object' ? results : data) as Record<string, unknown>;
  const anyKey = ['created', 'updated', 'skipped', 'failed'].some((k) => k in source);
  if (!anyKey) return null;
  return {
    created: num(source.created),
    updated: num(source.updated),
    skipped: num(source.skipped),
    failed: num(source.failed),
  };
}

function countKey(res: { json: unknown; ok: boolean; error: string | null; status: number }):
  | { kind: 'ok'; count: number }
  | { kind: 'error'; message: string } {
  if (res.error) return { kind: 'error', message: res.error };
  if (!res.ok) return { kind: 'error', message: `HTTP ${res.status}` };
  const data = (res.json as { data?: unknown } | null)?.data;
  if (Array.isArray(data)) return { kind: 'ok', count: data.length };
  return { kind: 'error', message: 'response had no data[] array to count' };
}

interface EntityCount {
  entity: string;
  path: string;
  count: number | null;
  note: string;
  status: number;
  raw: unknown;
  text: string;
}

export default function Datasets({ session }: { session: BenchSession }) {
  const [index, setIndex] = useState<FixtureIndex | null>(null);
  const [indexError, setIndexError] = useState<string | null>(null);
  const [indexLoading, setIndexLoading] = useState(true);

  const [runs, setRuns] = useState<DatasetRun[]>([]);
  const [running, setRunning] = useState(false);
  const [rollbackBusy, setRollbackBusy] = useState(false);
  const [rollbackResults, setRollbackResults] = useState<StepResult[]>([]);

  const [entities, setEntities] = useState<EntityCount[] | null>(null);
  const [entitiesLoading, setEntitiesLoading] = useState(false);

  const cancelRef = useRef(false);

  const loadIndex = useCallback(async () => {
    setIndexLoading(true);
    setIndexError(null);
    const url = fixtureUrl('index.json');
    try {
      const res = await fetch(url, { headers: { Accept: 'application/json' } });
      const text = await res.text();
      if (!res.ok) {
        setIndex(null);
        setIndexError(`GET ${url} → HTTP ${res.status} ${res.statusText}`);
        return;
      }
      const parsed = JSON.parse(text) as FixtureIndex;
      if (!parsed || !Array.isArray(parsed.datasets)) {
        setIndex(null);
        setIndexError(`GET ${url} returned JSON with no "datasets" array.`);
        return;
      }
      setIndex(parsed);
    } catch (err) {
      setIndex(null);
      setIndexError(
        `GET ${url} failed: ${err instanceof Error ? err.message : String(err)}. ` +
          'The fixture sync step (npm run sync-fixtures, wired into prebuild) did not run, or dist/fixtures/ was not shipped.',
      );
    } finally {
      setIndexLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadIndex();
  }, [loadIndex]);

  const loadEntityCounts = useCallback(async () => {
    // Nothing to count before the shell hands over a token: unauthenticated
    // probes only produce 401s. This callback is re-created when the token
    // arrives, so the effect below runs again at that point.
    if (!session.token) return;
    setEntitiesLoading(true);
    const targets: Array<{ entity: string; path: string }> = [
      { entity: 'locations', path: '/api/v1/locations' },
      { entity: 'assets', path: '/api/v1/assets' },
      { entity: 'parts', path: '/api/v1/parts' },
      { entity: 'work orders', path: '/api/v1/work-orders' },
    ];
    const out: EntityCount[] = [];
    for (const target of targets) {
      const res = await api(target.path, { token: session.token, apiBase: session.apiBase });
      const counted = countKey(res);
      out.push({
        entity: target.entity,
        path: target.path,
        count: counted.kind === 'ok' ? counted.count : null,
        note:
          counted.kind === 'ok'
            ? ''
            : res.error
              ? `transport error: ${res.error}`
              : `HTTP ${res.status}: ${errorMessage(res.json, counted.message)}`,
        status: res.status,
        raw: res.json,
        text: res.text,
      });
      setEntities([...out]);
    }
    setEntitiesLoading(false);
  }, [session.token, session.apiBase]);

  useEffect(() => {
    void loadEntityCounts();
    // Reload entity counts whenever the token changes (different identity, different RLS scope).
  }, [loadEntityCounts]);

  // ── Pipeline ──────────────────────────────────────────────────────────────

  const initialRuns = useCallback((datasets: FixtureDataset[]): DatasetRun[] => {
    return [...datasets]
      .sort((a, b) => a.order - b.order)
      .map((d) => ({
        objectType: d.object_type,
        file: d.file,
        order: d.order,
        fixtureCount: d.count,
        status: 'pending' as RunStatus,
        steps: [],
        jobId: null,
        upload: { chunksSent: 0, chunksTotal: 0, recordsAccepted: null },
        counts: null,
        failure: null,
      }));
  }, []);

  const patchRun = useCallback((objectType: string, patch: Partial<DatasetRun>) => {
    setRuns((prev) => prev.map((r) => (r.objectType === objectType ? { ...r, ...patch } : r)));
  }, []);

  const pushStep = useCallback((objectType: string, step: StepResult) => {
    setRuns((prev) =>
      prev.map((r) => (r.objectType === objectType ? { ...r, steps: [...r.steps, step] } : r)),
    );
  }, []);

  const runPipeline = useCallback(async () => {
    if (!index) return;
    cancelRef.current = false;
    setRunning(true);
    setRollbackResults([]);

    const ordered = [...index.datasets].sort((a, b) => a.order - b.order);
    setRuns(initialRuns(index.datasets));

    const sourceSystem = index.source ?? 'SMRT-Maintenance-Logs';

    for (const dataset of ordered) {
      if (cancelRef.current) break;
      const objectType = dataset.object_type;
      patchRun(objectType, { status: 'running' });

      // Read the canonical records once, straight from the shipped fixture.
      let records: Array<Record<string, unknown>>;
      try {
        const fileRes = await fetch(fixtureUrl(dataset.file), {
          headers: { Accept: 'application/json' },
        });
        if (!fileRes.ok) {
          patchRun(objectType, {
            status: 'failed',
            failure: `could not read ${dataset.file}: HTTP ${fileRes.status}`,
          });
          continue;
        }
        const parsed = (await fileRes.json()) as { records?: unknown };
        records = Array.isArray(parsed.records)
          ? (parsed.records as Array<Record<string, unknown>>)
          : [];
      } catch (err) {
        patchRun(objectType, {
          status: 'failed',
          failure: `could not read ${dataset.file}: ${err instanceof Error ? err.message : String(err)}`,
        });
        continue;
      }

      const stamp = new Date().toISOString().replace(/[:.]/g, '-');
      const name = `test-bench ${objectType} ${stamp}`;

      // (a) create the job
      const createRes = await api('/api/v1/migrations', {
        method: 'POST',
        token: session.token,
        apiBase: session.apiBase,
        body: {
          name,
          source_system: sourceSystem,
          source_object_type: objectType,
          description: `SIP Test Bench fixture load of ${dataset.file} (${records.length} canonical records)`,
        },
      });
      pushStep(objectType, {
        name: 'create job',
        method: createRes.method,
        url: createRes.url,
        status: createRes.status,
        ok: createRes.ok,
        elapsedMs: createRes.elapsedMs,
        json: createRes.json,
        text: createRes.text,
        error: createRes.error,
        detail: createRes.ok ? name : undefined,
      });

      if (createRes.error || !createRes.ok) {
        patchRun(objectType, {
          status: 'failed',
          failure: createRes.error
            ? `create job transport error: ${createRes.error}`
            : `create job failed: HTTP ${createRes.status} — ${errorMessage(createRes.json, 'no error message in body')}`,
        });
        continue;
      }

      const jobId = extractJobId(createRes.json);
      if (!jobId) {
        // Never guess an id: show the raw body and stop this dataset.
        patchRun(objectType, {
          status: 'failed',
          failure:
            'could not find a job id in the create response (looked for data.id, id, job_id); raw response is shown in the step log',
        });
        continue;
      }
      patchRun(objectType, { jobId });
      jobLog.push({
        objectType,
        objectTypeLabel: labelFor(objectType),
        jobId,
        name,
        createdAt: new Date().toISOString(),
      });

      // (b) upload source records in chunks
      const transport = toTransportRecords(records, objectType);
      const chunks: Array<Array<Record<string, unknown>>> = [];
      for (let i = 0; i < transport.length; i += CHUNK_SIZE) {
        chunks.push(transport.slice(i, i + CHUNK_SIZE));
      }
      patchRun(objectType, {
        upload: { chunksSent: 0, chunksTotal: chunks.length, recordsAccepted: null },
      });

      let uploadFailed: string | null = null;
      let accepted = 0;
      for (let i = 0; i < chunks.length; i += 1) {
        const chunk = chunks[i];
        const chunkRes = await api(`/api/v1/migrations/${jobId}/source-records`, {
          method: 'POST',
          token: session.token,
          apiBase: session.apiBase,
          body: { records: chunk },
        });
        const chunkOk = chunkRes.ok && !chunkRes.error;
        if (chunkOk) accepted += num((chunkRes.json as { data?: { record_count?: unknown } } | null)?.data?.record_count) ?? chunk.length;
        patchRun(objectType, {
          upload: { chunksSent: i + 1, chunksTotal: chunks.length, recordsAccepted: accepted },
        });
        if (!chunkOk) {
          pushStep(objectType, {
            name: 'upload records',
            method: chunkRes.method,
            url: chunkRes.url,
            status: chunkRes.status,
            ok: false,
            elapsedMs: chunkRes.elapsedMs,
            json: chunkRes.json,
            text: chunkRes.text,
            error: chunkRes.error,
            detail: `chunk ${i + 1}/${chunks.length} (${chunk.length} records) failed; upload stopped`,
          });
          uploadFailed = chunkRes.error
            ? `chunk ${i + 1}/${chunks.length} transport error: ${chunkRes.error}`
            : `chunk ${i + 1}/${chunks.length} → HTTP ${chunkRes.status} — ${errorMessage(chunkRes.json, 'no error message in body')}`;
          break;
        }
      }

      if (uploadFailed) {
        patchRun(objectType, { status: 'failed', failure: uploadFailed });
        continue;
      }
      pushStep(objectType, {
        name: 'upload records',
        method: 'POST',
        url: `${session.apiBase}/api/v1/migrations/${jobId}/source-records`,
        status: 200,
        ok: true,
        elapsedMs: 0,
        json: { data: { record_count: accepted, status: 'uploaded' } },
        text: '',
        error: null,
        detail: `${chunks.length} chunk(s) of ≤${CHUNK_SIZE} records, ${accepted} accepted in total`,
      });

      // (c..f) validate, dry-run, import, report
      const progression: Array<{ name: StepName; path: string }> = [
        { name: 'validate', path: `/api/v1/migrations/${jobId}/validate` },
        { name: 'dry-run', path: `/api/v1/migrations/${jobId}/dry-run` },
        { name: 'import', path: `/api/v1/migrations/${jobId}/import` },
      ];

      let stepFailed = false;
      for (const step of progression) {
        const res = await api(step.path, {
          method: 'POST',
          token: session.token,
          apiBase: session.apiBase,
        });
        pushStep(objectType, {
          name: step.name,
          method: res.method,
          url: res.url,
          status: res.status,
          ok: res.ok && !res.error,
          elapsedMs: res.elapsedMs,
          json: res.json,
          text: res.text,
          error: res.error,
        });
        if (res.error || !res.ok) {
          patchRun(objectType, {
            status: 'failed',
            failure: res.error
              ? `${step.name} transport error: ${res.error}`
              : `${step.name} failed: HTTP ${res.status} — ${errorMessage(res.json, 'no error message in body')}`,
          });
          stepFailed = true;
          break;
        }
      }
      if (stepFailed) continue;

      const reportRes = await api(`/api/v1/migrations/${jobId}/report`, {
        token: session.token,
        apiBase: session.apiBase,
      });
      pushStep(objectType, {
        name: 'report',
        method: reportRes.method,
        url: reportRes.url,
        status: reportRes.status,
        ok: reportRes.ok && !reportRes.error,
        elapsedMs: reportRes.elapsedMs,
        json: reportRes.json,
        text: reportRes.text,
        error: reportRes.error,
      });

      patchRun(objectType, {
        status: reportRes.ok && !reportRes.error ? 'done' : 'failed',
        counts: extractCounts(reportRes.json),
        failure:
          reportRes.ok && !reportRes.error
            ? null
            : `report failed: HTTP ${reportRes.status} — ${errorMessage(reportRes.json, 'no error message in body')}`,
      });
    }

    setRunning(false);
    void loadEntityCounts();
  }, [index, initialRuns, patchRun, pushStep, session.token, session.apiBase, loadEntityCounts]);

  const rollbackAll = useCallback(async () => {
    const jobs = getCreatedJobs();
    if (jobs.length === 0) return;
    setRollbackBusy(true);
    const results: StepResult[] = [];
    for (const job of jobs) {
      const res = await api(`/api/v1/migrations/${job.jobId}/rollback`, {
        method: 'POST',
        token: session.token,
        apiBase: session.apiBase,
      });
      const result: StepResult = {
        name: 'import',
        method: res.method,
        url: res.url,
        status: res.status,
        ok: res.ok && !res.error,
        elapsedMs: res.elapsedMs,
        json: res.json,
        text: res.text,
        error: res.error,
        detail: `${job.objectTypeLabel} — job ${job.jobId}`,
      };
      results.push(result);
      setRollbackResults([...results]);
    }
    setRollbackBusy(false);
    void loadEntityCounts();
  }, [session.token, session.apiBase, loadEntityCounts]);

  const fixtureCounts = new Map<string, { count: number; file: string }>();
  if (index) {
    for (const d of index.datasets) {
      fixtureCounts.set(d.object_type, { count: d.count, file: d.file });
    }
  }

  const jobCount = getCreatedJobs().length;
  const anyRun = runs.length > 0;
  const totals = runs.reduce(
    (acc, r) => {
      acc.created += r.counts?.created ?? 0;
      acc.skipped += r.counts?.skipped ?? 0;
      acc.failed += r.counts?.failed ?? 0;
      return acc;
    },
    { created: 0, skipped: 0, failed: 0 },
  );
  const completedRuns = runs.filter((r) => r.status === 'done').length;

  return (
    <div>
      <Card
        title="Fixtures"
        lead={
          <>
            Canonical records shipped with this plugin at build time by{' '}
            <code>scripts/sync-fixtures.mjs</code> (wired into <code>prebuild</code>), fetched from{' '}
            <code>{fixtureUrl('index.json')}</code>.
          </>
        }
        actions={
          <div className="row tight">
            <button onClick={() => void loadIndex()} disabled={indexLoading}>
              {indexLoading ? <span className="spinner" /> : null} Reload fixture index
            </button>
          </div>
        }
      >
        {indexError ? (
          <Banner kind="error" title="Fixture index unavailable">
            <p>{indexError}</p>
            <p className="small">
              Run <code>npm run sync-fixtures</code> (or <code>npm run build</code>, which runs it
              first) so <code>public/fixtures/</code> is populated and copied into <code>dist/</code>.
            </p>
          </Banner>
        ) : null}

        {index ? (
          <>
            <div className="stat-row">
              <Stat label="source" value={<span className="small">{index.source ?? 'not declared'}</span>} />
              <Stat label="datasets" value={index.datasets.length} />
              <Stat
                label="records"
                value={
                  num(index.total_records) ??
                  index.datasets.reduce((sum, d) => sum + (d.count ?? 0), 0)
                }
              />
              <Stat
                label="generated"
                value={
                  <span className="small">
                    {index.generated_at ? new Date(index.generated_at).toLocaleString() : 'unknown'}
                  </span>
                }
              />
            </div>

            <h3>Load plan (dependency order)</h3>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th className="num">Order</th>
                    <th>Object type</th>
                    <th>File</th>
                    <th className="num">Fixture records</th>
                    <th>Declared type</th>
                  </tr>
                </thead>
                <tbody>
                  {[...index.datasets]
                    .sort((a, b) => a.order - b.order)
                    .map((d) => (
                      <tr key={d.file}>
                        <td className="num">{d.order}</td>
                        <td>
                          <code>{d.object_type}</code>
                        </td>
                        <td>
                          <code>{d.file}</code>
                        </td>
                        <td className="num">{d.count}</td>
                        <td>
                          {d.declared_object_type ? (
                            <code>{d.declared_object_type}</code>
                          ) : (
                            <span className="muted">—</span>
                          )}
                          {d.available === false ? (
                            <>
                              {' '}
                              <Pill kind="warn">unavailable</Pill>
                            </>
                          ) : null}
                        </td>
                      </tr>
                    ))}
                </tbody>
              </table>
            </div>
            <p className="small muted" style={{ marginTop: 8 }}>
              Order matters: assets resolve to asset types and locations by external id, and work
              orders resolve to assets and parts, so each object type is created, uploaded, validated,
              dry-run and imported strictly in the order above.
            </p>
          </>
        ) : null}
      </Card>

      <Card
        title="Load into SIP"
        lead="Runs the real migration pipeline for all five object types in dependency order, with no field mappings — a job with no mappings treats each record as already canonical, which is what this fixture relies on. Records are uploaded in chunks of 500 with external_id and source_object_type added."
        actions={
          <div className="row tight">
            <button
              className="primary"
              onClick={() => void runPipeline()}
              disabled={!index || running || !session.token}
            >
              {running ? <span className="spinner" /> : null} {running ? 'Running…' : 'Load into SIP'}
            </button>
            {running ? (
              <button
                onClick={() => {
                  cancelRef.current = true;
                }}
              >
                Stop after current object type
              </button>
            ) : null}
            <button
              className="danger"
              onClick={() => void rollbackAll()}
              disabled={rollbackBusy || jobCount === 0 || !session.token}
            >
              Rollback all ({jobCount})
            </button>
          </div>
        }
      >
        {!session.token ? (
          <Banner kind="warn" title="No token">
            <p>
              The pipeline calls <code>/api/v1/migrations</code> and will return 401 without a bearer
              token. Run this app inside the SIP shell, or paste a token in standalone mode (top of the
              page), or use API Explorer → login as.
            </p>
          </Banner>
        ) : null}

        {!running && anyRun ? (
          <div className="stat-row">
            <Stat label="object types done" value={`${completedRuns} / ${runs.length}`} />
            <Stat label="created" value={totals.created} />
            <Stat label="skipped" value={totals.skipped} />
            <Stat label="failed" value={totals.failed} />
          </div>
        ) : null}

        {anyRun ? (
          <>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th className="num">#</th>
                    <th>Object type</th>
                    <th>Status</th>
                    <th>Job id</th>
                    <th className="num">Uploaded</th>
                    <th className="num">Created</th>
                    <th className="num">Skipped</th>
                    <th className="num">Failed</th>
                  </tr>
                </thead>
                <tbody>
                  {runs.map((r) => (
                    <tr key={r.objectType}>
                      <td className="num">{r.order}</td>
                      <td>
                        <code>{r.objectType}</code>
                        <div className="small muted">fixture: {r.fixtureCount}</div>
                      </td>
                      <td>
                        {r.status === 'running' ? (
                          <>
                            <Pill kind="run">running</Pill>
                            {r.upload.chunksTotal > 0 ? (
                              <div className="small muted">
                                chunk {r.upload.chunksSent}/{r.upload.chunksTotal}
                              </div>
                            ) : null}
                          </>
                        ) : (
                          <Pill kind={r.status === 'done' ? 'pass' : r.status === 'failed' ? 'fail' : 'idle'}>
                            {r.status}
                          </Pill>
                        )}
                      </td>
                      <td>
                        {r.jobId ? (
                          <code className="small">{r.jobId}</code>
                        ) : (
                          <span className="muted">—</span>
                        )}
                      </td>
                      <td className="num">
                        {r.upload.recordsAccepted === null ? (
                          <span className="muted">—</span>
                        ) : (
                          r.upload.recordsAccepted
                        )}
                      </td>
                      <td className="num">{r.counts?.created ?? <span className="muted">—</span>}</td>
                      <td className="num">{r.counts?.skipped ?? <span className="muted">—</span>}</td>
                      <td className="num">{r.counts?.failed ?? <span className="muted">—</span>}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {runs
              .filter((r) => r.failure)
              .map((r) => (
                <Banner key={r.objectType} kind="error" title={`${labelFor(r.objectType)}: ${r.failure}`}>
                  <></>
                </Banner>
              ))}

            <h3>Per-object-type evidence</h3>
            {runs
              .filter((r) => r.steps.length > 0)
              .map((r) => (
                <details key={r.objectType} className="raw">
                  <summary>
                    {labelFor(r.objectType)} — {r.steps.length} request(s),{' '}
                    {r.steps.filter((s) => s.ok).length} ok
                  </summary>
                  <div className="table-wrap" style={{ marginTop: 8 }}>
                    <table>
                      <thead>
                        <tr>
                          <th>Step</th>
                          <th>Result</th>
                          <th className="num">Time</th>
                          <th>Detail</th>
                        </tr>
                      </thead>
                      <tbody>
                        {r.steps.map((s, i) => (
                          <tr key={`${s.name}-${i}`}>
                            <td>
                              <code>{s.name}</code>
                              <div className="small muted">{s.method}</div>
                            </td>
                            <td>
                              <Pill kind={s.ok ? 'pass' : 'fail'}>
                                {s.error ? 'transport error' : s.status}
                              </Pill>
                            </td>
                            <td className="num">{s.elapsedMs} ms</td>
                            <td>
                              {s.detail ? <div className="small">{s.detail}</div> : null}
                              {s.error ? <div className="small" style={{ color: 'var(--danger)' }}>{s.error}</div> : null}
                              <RawJson key={i} value={s.json} text={s.text} summary="Raw body" />
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </details>
              ))}
          </>
        ) : (
          <p className="muted">No pipeline run yet in this tab.</p>
        )}
      </Card>

      {rollbackResults.length > 0 ? (
        <Card
          title="Rollback results"
          lead="Locations, asset types and parts have no archive operation in SIP, so their rollback is expected to report zero processed / skipped rather than an error. Assets and work orders are archived."
        >
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Object type / job</th>
                  <th>Result</th>
                  <th className="num">Time</th>
                  <th>Raw</th>
                </tr>
              </thead>
              <tbody>
                {rollbackResults.map((r, i) => (
                  <tr key={`${r.url}-${i}`}>
                    <td className="small">{r.detail}</td>
                    <td>
                      <Pill kind={r.ok ? 'pass' : 'fail'}>
                        {r.error ? 'transport error' : r.status}
                      </Pill>
                    </td>
                    <td className="num">{r.elapsedMs} ms</td>
                    <td>
                      <RawJson value={r.json} text={r.text} summary="Raw body" />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      ) : null}

      <Card
        title="SIP entity counts vs fixture"
        lead="Live counts from the running API, compared with the counts declared in fixtures/index.json. A mismatch is a finding, not an error: it means the fixture is not fully loaded, or extra rows exist."
        actions={
          <button onClick={() => void loadEntityCounts()} disabled={entitiesLoading}>
            {entitiesLoading ? <span className="spinner" /> : null} Refresh counts
          </button>
        }
      >
        {entities ? (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Entity</th>
                  <th>Endpoint</th>
                  <th className="num">SIP count</th>
                  <th className="num">Fixture count</th>
                  <th>Match</th>
                </tr>
              </thead>
              <tbody>
                {entities.map((e) => {
                  const fixtureKey =
                    e.entity === 'locations'
                      ? 'location'
                      : e.entity === 'assets'
                        ? 'asset'
                        : e.entity === 'parts'
                          ? 'part'
                          : e.entity === 'work orders'
                            ? 'work_order'
                            : null;
                  const fixture = fixtureKey ? fixtureCounts.get(fixtureKey) : undefined;
                  const match =
                    e.count === null || fixture === undefined
                      ? null
                      : e.count === fixture.count;
                  return (
                    <tr key={e.entity}>
                      <td>{e.entity}</td>
                      <td>
                        <code className="small">{e.path}</code>
                      </td>
                      <td className="num">
                        {e.count === null ? <Pill kind="fail">{e.status || 'err'}</Pill> : e.count}
                      </td>
                      <td className="num">
                        {fixture ? fixture.count : <span className="muted">—</span>}
                      </td>
                      <td>
                        {match === null ? (
                          <Pill kind="idle">n/a</Pill>
                        ) : match ? (
                          <Pill kind="pass">match</Pill>
                        ) : (
                          <Pill kind="warn">differs</Pill>
                        )}
                        {e.note ? <div className="small" style={{ color: 'var(--danger)' }}>{e.note}</div> : null}
                        <RawJson value={e.raw} text={e.text} summary="Raw response" />
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="muted">{entitiesLoading ? 'Loading counts…' : 'No counts loaded.'}</p>
        )}
        {entities && !session.token ? (
          <p className="small muted" style={{ marginTop: 8 }}>
            These requests were sent without a token, so 401s above are expected until one is supplied.
          </p>
        ) : null}
      </Card>
    </div>
  );
}
