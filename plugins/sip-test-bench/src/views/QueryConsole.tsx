import { useCallback, useMemo, useRef, useState } from 'react';
import type { BenchSession } from '../App';
import {
  api,
  asArray,
  errorMessage,
  extractMessageId,
  streamSse,
  type ApiResponse,
  type SseEvent,
} from '../api';
import { Banner, Card, HttpMeta, Pill, RawJson, statusPillKind } from '../ui';

interface VerificationPayload {
  checks_passed?: number;
  checks_total?: number;
  status?: string;
}

interface ContextPayload {
  records_queried?: number;
  paths?: string[];
}

interface SourceRef {
  [key: string]: unknown;
}

interface ConversationSummary {
  id: string;
  title?: string | null;
  created_at?: string;
  updated_at?: string;
}

interface ConversationMessage {
  id: string;
  role?: string;
  content?: string;
  sources?: unknown;
  created_at?: string;
}

/** Does this error text look like "AI is switched off in config"? */
function looksLikeAiDisabled(text: string): boolean {
  const t = text.toLowerCase();
  return (
    t.includes('ai_enabled') ||
    t.includes('ai is disabled') ||
    t.includes('ai disabled') ||
    t.includes('not enabled') ||
    t.includes('no provider') ||
    t.includes('provider not configured') ||
    t.includes('feature disabled')
  );
}

function asRecord(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

export default function QueryConsole({ session }: { session: BenchSession }) {
  const [prompt, setPrompt] = useState('');
  const [running, setRunning] = useState(false);
  const [events, setEvents] = useState<SseEvent[]>([]);
  const [answer, setAnswer] = useState('');
  const [contextInfo, setContextInfo] = useState<ContextPayload | null>(null);
  const [verification, setVerification] = useState<VerificationPayload | null>(null);
  const [donePayload, setDonePayload] = useState<Record<string, unknown> | null>(null);
  const [streamError, setStreamError] = useState<ApiResponse | null>(null);
  const [streamMeta, setStreamMeta] = useState<{
    status: number;
    elapsedMs: number;
    error: string | null;
  } | null>(null);

  const [messageId, setMessageId] = useState('');
  const [traceResults, setTraceResults] = useState<
    Array<{ label: string; res: ApiResponse }>
  >([]);
  const [traceBusy, setTraceBusy] = useState<string | null>(null);

  const [conversations, setConversations] = useState<ConversationSummary[] | null>(null);
  const [conversationsError, setConversationsError] = useState<ApiResponse | null>(null);
  const [conversationsBusy, setConversationsBusy] = useState(false);
  const [openConversation, setOpenConversation] = useState<{
    id: string;
    res: ApiResponse;
  } | null>(null);

  const abortRef = useRef<AbortController | null>(null);

  // Never guess the message id from the chat stream — scan it, and say so plainly if absent.
  const discoveredMessageId = useMemo(() => {
    for (const evt of events) {
      const found = extractMessageId(evt.json);
      if (found) return found;
    }
    return null;
  }, [events]);

  const ask = useCallback(async () => {
    if (!prompt.trim()) return;
    setRunning(true);
    setEvents([]);
    setAnswer('');
    setContextInfo(null);
    setVerification(null);
    setDonePayload(null);
    setStreamError(null);
    setStreamMeta(null);
    setTraceResults([]);

    const controller = new AbortController();
    abortRef.current = controller;

    const collected: SseEvent[] = [];
    const result = await streamSse('/api/v1/ai/chat', {
      method: 'POST',
      token: session.token,
      apiBase: session.apiBase,
      body: { message: prompt },
      signal: controller.signal,
      onHttpError: (res) => setStreamError(res),
      onEvent: (evt) => {
        collected.push(evt);
        setEvents([...collected]);
        const payload = asRecord(evt.json);
        const type = payload && typeof payload.type === 'string' ? payload.type : null;
        if (type === 'chunk' && typeof payload?.text === 'string') {
          setAnswer((prev) => prev + (payload.text as string));
        } else if (type === 'context') {
          setContextInfo({
            records_queried:
              typeof payload?.records_queried === 'number' ? payload.records_queried : undefined,
            paths: Array.isArray(payload?.paths) ? (payload.paths as string[]) : undefined,
          });
        } else if (type === 'verification') {
          setVerification({
            checks_passed:
              typeof payload?.checks_passed === 'number' ? payload.checks_passed : undefined,
            checks_total:
              typeof payload?.checks_total === 'number' ? payload.checks_total : undefined,
            status: typeof payload?.status === 'string' ? payload.status : undefined,
          });
        } else if (type === 'done') {
          setDonePayload(payload);
        }
      },
    });

    setStreamMeta({ status: result.status, elapsedMs: result.elapsedMs, error: result.error });
    setRunning(false);
    abortRef.current = null;
  }, [prompt, session.token, session.apiBase]);

  const stop = useCallback(() => {
    abortRef.current?.abort();
  }, []);

  const fetchTrace = useCallback(
    async (label: 'retrieval' | 'verification', id: string) => {
      // Take the message id as an argument rather than reading state: the click
      // handlers that set it are setting the same state, and a stale read would
      // silently skip the request.
      const messageIdValue = id.trim();
      if (!messageIdValue) return;
      setTraceBusy(label);
      const res = await api(`/api/v1/ai/messages/${messageIdValue}/${label}-trace`, {
        token: session.token,
        apiBase: session.apiBase,
      });
      setTraceBusy(null);
      setTraceResults((prev) => [...prev, { label, res }]);
    },
    [session.token, session.apiBase],
  );

  const loadConversations = useCallback(async () => {
    setConversationsBusy(true);
    setConversationsError(null);
    const res = await api('/api/v1/ai/conversations', {
      token: session.token,
      apiBase: session.apiBase,
    });
    setConversationsBusy(false);
    if (res.error || !res.ok) {
      setConversations(null);
      setConversationsError(res);
      return;
    }
    const rows = asArray(res.json)
      .map((row) => asRecord(row))
      .filter((row): row is Record<string, unknown> => row !== null)
      .map((row) => ({
        id: String(row.id ?? ''),
        title: typeof row.title === 'string' ? row.title : null,
        created_at: typeof row.created_at === 'string' ? row.created_at : undefined,
        updated_at: typeof row.updated_at === 'string' ? row.updated_at : undefined,
      }))
      .filter((row) => row.id);
    setConversations(rows);
  }, [session.token, session.apiBase]);

  const openConversationById = useCallback(
    async (id: string) => {
      const res = await api(`/api/v1/ai/conversations/${id}`, {
        token: session.token,
        apiBase: session.apiBase,
      });
      setOpenConversation({ id, res });
    },
    [session.token, session.apiBase],
  );

  const answerSources = useMemo<SourceRef[]>(() => {
    const raw = donePayload?.sources;
    if (!Array.isArray(raw)) return [];
    return raw.map((s) => (asRecord(s) ?? { raw: s }) as SourceRef);
  }, [donePayload]);

  const unknownEvents = useMemo(
    () =>
      events.filter((evt) => {
        const payload = asRecord(evt.json);
        if (!payload) return true; // non-JSON SSE payload: show it verbatim
        const type = payload.type;
        return typeof type !== 'string' || !['context', 'chunk', 'verification', 'done', 'error'].includes(type);
      }),
    [events],
  );

  const errorEvent = useMemo(() => {
    for (const evt of events) {
      const payload = asRecord(evt.json);
      if (payload && payload.type === 'error') return payload;
    }
    return null;
  }, [events]);

  const aiDisabled =
    (streamError && looksLikeAiDisabled(streamError.text)) ||
    (conversationsError && looksLikeAiDisabled(conversationsError.text)) ||
    (errorEvent && looksLikeAiDisabled(JSON.stringify(errorEvent)));

  const conversationMessages =
    openConversation && openConversation.res.json
      ? ((asRecord((openConversation.res.json as { data?: unknown }).data) as
          | (Record<string, unknown> & { messages?: ConversationMessage[] })
          | null) ?? null)
      : null;

  return (
    <div>
      <Card
        title="Ask the AI"
        lead={
          <>
            Streams <code>POST /api/v1/ai/chat</code> as an SSE request and renders each event type.
            Known types are{' '}
            <code>context</code>, <code>chunk</code>, <code>verification</code>, <code>done</code> and{' '}
            <code>error</code>; anything else is shown verbatim in the raw log so nothing is dropped.
          </>
        }
        actions={
          <div className="row tight">
            <button className="primary" onClick={() => void ask()} disabled={running || !prompt.trim()}>
              {running ? <span className="spinner" /> : null} {running ? 'Streaming…' : 'Ask'}
            </button>
            {running ? <button onClick={stop}>Stop</button> : null}
          </div>
        }
      >
        <div className="field">
          <label htmlFor="prompt">Prompt</label>
          <textarea
            id="prompt"
            rows={3}
            value={prompt}
            placeholder="e.g. Which assets had the most corrective work orders, and what components were replaced?"
            onChange={(e) => setPrompt(e.target.value)}
          />
        </div>

        {!session.token ? (
          <Banner kind="warn" title="No bearer token">
            <p>
              <code>/api/v1/ai/chat</code> requires authentication. Supply a token above or use{' '}
              <em>login as</em> below.
            </p>
          </Banner>
        ) : null}

        {aiDisabled ? (
          <Banner kind="warn" title="SIP reports the AI subsystem is not available">
            <p>
              The endpoint said so verbatim:{' '}
              <code>
                {streamError
                  ? errorMessage(streamError.json, streamError.text || 'no body')
                  : conversationsError
                    ? errorMessage(conversationsError.json, conversationsError.text || 'no body')
                    : JSON.stringify(errorEvent)}
              </code>
            </p>
            <p className="small">
              This means the API's AI feature flag is off — set <code>SIP_AI_ENABLED=true</code> in the
              environment SIP runs with (and a provider/API key if one is required), restart the API,
              then retry. No answer can be produced until then; this pane will not invent one.
            </p>
          </Banner>
        ) : null}

        {streamMeta ? (
          <div style={{ marginTop: 14 }}>
            <HttpMeta
              method="POST"
              status={streamMeta.status}
              ok={streamMeta.status >= 200 && streamMeta.status < 300 && !streamMeta.error}
              elapsedMs={streamMeta.elapsedMs}
              error={streamMeta.error}
              url={`${session.apiBase}/api/v1/ai/chat`}
            />
            {streamError ? (
              <Banner kind="error" title={`Non-stream response (HTTP ${streamError.status})`}>
                <p className="mono small">{streamError.text || '(empty body)'}</p>
              </Banner>
            ) : null}
          </div>
        ) : null}

        {events.length > 0 ? (
          <div className="stat-row" style={{ marginTop: 12 }}>
            <div className="stat">
              <div className="stat-label">context</div>
              <div className="stat-value small">
                {contextInfo
                  ? `${contextInfo.records_queried ?? '?'} records queried`
                  : 'no context event'}
              </div>
              {contextInfo?.paths?.length ? (
                <div className="small muted">{contextInfo.paths.join(' · ')}</div>
              ) : null}
            </div>
            <div className="stat">
              <div className="stat-label">verification</div>
              <div className="stat-value small">
                {verification ? (
                  <>
                    {verification.checks_passed ?? '?'}/{verification.checks_total ?? '?'}{' '}
                    <Pill kind={statusPillKind(verification.status ?? '')}>{verification.status ?? 'unknown'}</Pill>
                  </>
                ) : (
                  'no verification event'
                )}
              </div>
            </div>
            <div className="stat">
              <div className="stat-label">events</div>
              <div className="stat-value">{events.length}</div>
              <div className="small muted">
                {events.filter((e) => asRecord(e.json)?.type === 'chunk').length} chunks ·{' '}
                {unknownEvents.length} unknown
              </div>
            </div>
          </div>
        ) : null}

        <h3>Answer</h3>
        {answer ? (
          <div className="answer">{answer}</div>
        ) : (
          <p className="muted">
            {running ? 'Waiting for the first chunk…' : 'No answer text received yet.'}
          </p>
        )}

        {answerSources.length > 0 ? (
          <>
            <h3>Sources ({answerSources.length})</h3>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>Source</th>
                    <th>Detail</th>
                  </tr>
                </thead>
                <tbody>
                  {answerSources.map((src, i) => (
                    <tr key={i}>
                      <td>
                        <code>{String(src.sip_entity_type ?? src.entity_type ?? src.type ?? `source ${i + 1}`)}</code>
                        {src.sip_entity_id || src.entity_id || src.id ? (
                          <div className="small muted">
                            {String(src.sip_entity_id ?? src.entity_id ?? src.id ?? '')}
                          </div>
                        ) : null}
                      </td>
                      <td className="small">
                        {typeof src.title === 'string' ? <div>{src.title}</div> : null}
                        {typeof src.snippet === 'string' ? <div className="muted">{src.snippet}</div> : null}
                        <RawJson value={src} summary="Raw source object" />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </>
        ) : donePayload ? (
          <p className="muted small">The done event carried no sources array.</p>
        ) : null}

        {donePayload ? (
          <RawJson value={donePayload} summary="Raw done event" open />
        ) : null}

        <h3>Traces</h3>
        {discoveredMessageId ? (
          <>
            <p className="small">
              A message id was present in the stream: <code>{discoveredMessageId}</code>
            </p>
            <div className="row tight">
              <button
                onClick={() => {
                  setMessageId(discoveredMessageId);
                  void fetchTrace('retrieval', discoveredMessageId);
                }}
                disabled={traceBusy !== null}
              >
                {traceBusy === 'retrieval' ? <span className="spinner" /> : null} Retrieval trace
              </button>
              <button
                onClick={() => {
                  setMessageId(discoveredMessageId);
                  void fetchTrace('verification', discoveredMessageId);
                }}
                disabled={traceBusy !== null}
              >
                {traceBusy === 'verification' ? <span className="spinner" /> : null} Verification trace
              </button>
            </div>
          </>
        ) : (
          <Banner kind="info" title="No message id in the chat stream">
            <p>
              The <code>done</code> event from <code>/api/v1/ai/chat</code> carries only{' '}
              <code>sources</code>, <code>verification_status</code> and <code>retrieval_paths</code> —
              no message id — so this app will not guess one. Open the conversation below (its assistant
              messages include ids) or paste an id to fetch the traces.
            </p>
          </Banner>
        )}

        {!discoveredMessageId ? (
          <div className="row" style={{ marginTop: 10 }}>
            <div className="field grow">
              <label htmlFor="msg-id">Message id</label>
              <input
                id="msg-id"
                type="text"
                className="mono"
                placeholder="uuid from a conversation message"
                value={messageId}
                onChange={(e) => setMessageId(e.target.value)}
              />
            </div>
            <div className="row tight">
              <button
                onClick={() => void fetchTrace('retrieval', messageId)}
                disabled={!messageId.trim() || traceBusy !== null}
              >
                Retrieval trace
              </button>
              <button
                onClick={() => void fetchTrace('verification', messageId)}
                disabled={!messageId.trim() || traceBusy !== null}
              >
                Verification trace
              </button>
            </div>
          </div>
        ) : null}

        {traceResults.map((t, i) => (
          <div key={`${t.label}-${i}`} style={{ marginTop: 12 }}>
            <HttpMeta
              method={t.res.method}
              status={t.res.status}
              ok={t.res.ok && !t.res.error}
              elapsedMs={t.res.elapsedMs}
              url={t.res.url}
              error={t.res.error}
            />
            <RawJson
              value={t.res.json}
              text={t.res.text}
              summary={`${t.label}-trace raw JSON`}
              open
              tall
            />
          </div>
        ))}

        <h3>Raw event log ({events.length})</h3>
        {events.length === 0 ? (
          <p className="muted">No events yet.</p>
        ) : (
          <pre className="mono-block tall">
            {events
              .map((evt, i) => {
                const payload = asRecord(evt.json);
                const type = payload && typeof payload.type === 'string' ? payload.type : 'UNKNOWN';
                return `#${i + 1} [${type}] ${evt.data}`;
              })
              .join('\n')}
          </pre>
        )}
        {unknownEvents.length > 0 ? (
          <>
            <h3>Unknown event types ({unknownEvents.length})</h3>
            <pre className="mono-block">{unknownEvents.map((e) => e.data).join('\n')}</pre>
          </>
        ) : null}
      </Card>

      <Card
        title="Conversations"
        lead="GET /api/v1/ai/conversations, then GET /api/v1/ai/conversations/{id} to read the stored messages (the assistant messages there carry the message ids the trace endpoints need)."
        actions={
          <button onClick={() => void loadConversations()} disabled={conversationsBusy}>
            {conversationsBusy ? <span className="spinner" /> : null} Load conversations
          </button>
        }
      >
        {conversationsError ? (
          <Banner kind="error" title={`GET /api/v1/ai/conversations → HTTP ${conversationsError.status}`}>
            <p>{errorMessage(conversationsError.json, conversationsError.text || 'no error message in body')}</p>
            <RawJson value={conversationsError.json} text={conversationsError.text} summary="Raw error body" />
          </Banner>
        ) : null}

        {conversations ? (
          conversations.length === 0 ? (
            <p className="muted">The API returned an empty conversation list for this identity.</p>
          ) : (
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>Title</th>
                    <th>Id</th>
                    <th>Updated</th>
                    <th />
                  </tr>
                </thead>
                <tbody>
                  {conversations.map((c) => (
                    <tr key={c.id}>
                      <td>{c.title ?? <span className="muted">untitled</span>}</td>
                      <td>
                        <code className="small">{c.id}</code>
                      </td>
                      <td className="small muted">
                        {c.updated_at ? new Date(c.updated_at).toLocaleString() : '—'}
                      </td>
                      <td>
                        <button className="ghost" onClick={() => void openConversationById(c.id)}>
                          Open
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )
        ) : (
          <p className="muted">Conversations not loaded yet.</p>
        )}

        {openConversation ? (
          <div style={{ marginTop: 14 }}>
            <HttpMeta
              method={openConversation.res.method}
              status={openConversation.res.status}
              ok={openConversation.res.ok && !openConversation.res.error}
              elapsedMs={openConversation.res.elapsedMs}
              url={openConversation.res.url}
              error={openConversation.res.error}
            />
            {conversationMessages?.messages?.length ? (
              <div className="table-wrap">
                <table>
                  <thead>
                    <tr>
                      <th>Role</th>
                      <th>Message id</th>
                      <th>Content</th>
                    </tr>
                  </thead>
                  <tbody>
                    {conversationMessages.messages.map((m) => (
                      <tr key={m.id}>
                        <td>
                          <Pill kind={m.role === 'assistant' ? 'info' : 'idle'}>{m.role ?? '?'}</Pill>
                        </td>
                        <td>
                          <code className="small">{m.id}</code>
                          <button
                            className="ghost"
                            onClick={() => void fetchTrace('retrieval', m.id)}
                          >
                            Use for traces
                          </button>
                        </td>
                        <td className="small" style={{ whiteSpace: 'pre-wrap' }}>
                          {m.content ?? ''}
                          {m.sources ? <RawJson value={m.sources} summary="Message sources" /> : null}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <RawJson
                value={openConversation.res.json}
                text={openConversation.res.text}
                summary="Raw conversation response"
                open
              />
            )}
          </div>
        ) : null}
      </Card>
    </div>
  );
}
