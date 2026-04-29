'use client';

import { useState } from 'react';
import { API_BASE, getToken } from '@/lib/api';

interface Source {
  id: string;
  type: string;
  title: string;
  relevance: number;
  field?: string;
  source_scope?: string;
  retrieval_type?: string;
  verified_by_sql?: boolean;
  quote?: string;
  timestamp?: string;
}

interface VerifiedClaim {
  claim: string;
  status: string;
  supporting_sources: string[];
}

interface ChatMessage {
  role: 'user' | 'assistant';
  text: string;
  sources?: Source[];
  retrievalPaths?: string[];
  verificationStatus?: string;
  verifiedClaims?: VerifiedClaim[];
}

export default function AIChatPage() {
  const [message, setMessage] = useState('');
  const [history, setHistory] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);
  const [streamingText, setStreamingText] = useState('');
  const [streamingSources, setStreamingSources] = useState<Source[]>([]);
  const [streamingPaths, setStreamingPaths] = useState<string[]>([]);
  const [streamingVerification, setStreamingVerification] = useState<string | null>(null);
  const [streamingClaims, setStreamingClaims] = useState<VerifiedClaim[]>([]);

  async function send() {
    if (!message.trim() || loading) return;
    const userMsg = message;
    setMessage('');
    setHistory((prev) => [...prev, { role: 'user', text: userMsg }]);
    setLoading(true);
    setStreamingText('');
    setStreamingSources([]);
    setStreamingPaths([]);
    setStreamingVerification(null);
    setStreamingClaims([]);

    let finalText = '';
    let finalSources: Source[] = [];
    let finalPaths: string[] = [];
    let finalVerification: string | undefined;
    let finalClaims: VerifiedClaim[] = [];

    try {
      const token = getToken();
      const res = await fetch(`${API_BASE}/ai/chat`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify({ message: userMsg }),
      });

      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: { message: res.statusText } }));
        throw new Error(err.error?.message || res.statusText);
      }

      const contentType = res.headers.get('content-type') || '';
      if (contentType.includes('text/event-stream')) {
        const reader = res.body?.getReader();
        const decoder = new TextDecoder();
        let buffer = '';

        if (!reader) {
          throw new Error('No response body');
        }

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });

          const lines = buffer.split('\n');
          buffer = lines.pop() || '';

          for (const line of lines) {
            const trimmed = line.trim();
            if (trimmed.startsWith('data: ')) {
              const jsonStr = trimmed.slice(6);
              if (!jsonStr) continue;
              try {
                const parsed = JSON.parse(jsonStr);

                // Context event: retrieval paths + record count
                if (parsed.type === 'context') {
                  if (parsed.paths) {
                    finalPaths = parsed.paths;
                    setStreamingPaths(finalPaths);
                  }
                }

                // Chunk event: streaming text
                if (parsed.type === 'chunk' && parsed.text) {
                  finalText += parsed.text;
                  setStreamingText(finalText);
                }

                // Verification event
                if (parsed.type === 'verification') {
                  finalVerification = parsed.status;
                  setStreamingVerification(parsed.status);
                }

                // Done event: sources, verification status, retrieval paths
                if (parsed.type === 'done') {
                  if (parsed.sources) {
                    finalSources = parsed.sources;
                    setStreamingSources(finalSources);
                  }
                  if (parsed.verification_status) {
                    finalVerification = parsed.verification_status;
                    setStreamingVerification(parsed.verification_status);
                  }
                  if (parsed.retrieval_paths) {
                    finalPaths = parsed.retrieval_paths;
                    setStreamingPaths(finalPaths);
                  }
                  // Extract verified claims from the structured answer if present
                  if (parsed.verified_claims) {
                    finalClaims = parsed.verified_claims;
                    setStreamingClaims(finalClaims);
                  }
                }
              } catch {
                // Ignore malformed JSON lines
              }
            }
          }
        }

        setHistory((prev) => [
          ...prev,
          {
            role: 'assistant',
            text: finalText,
            sources: finalSources.length > 0 ? finalSources : undefined,
            retrievalPaths: finalPaths.length > 0 ? finalPaths : undefined,
            verificationStatus: finalVerification,
            verifiedClaims: finalClaims.length > 0 ? finalClaims : undefined,
          },
        ]);
        setStreamingText('');
        setStreamingSources([]);
        setStreamingPaths([]);
        setStreamingVerification(null);
        setStreamingClaims([]);
      } else {
        const data = await res.json();
        setHistory((prev) => [
          ...prev,
          { role: 'assistant', text: data.answer || JSON.stringify(data) },
        ]);
      }
    } catch (e: any) {
      setHistory((prev) => [...prev, { role: 'assistant', text: `Error: ${e.message}` }]);
    } finally {
      setLoading(false);
    }
  }

  function verificationBadge(status: string) {
    const colors: Record<string, string> = {
      VERIFIED: 'bg-green-100 text-green-800',
      PARTIALLY_VERIFIED: 'bg-yellow-100 text-yellow-800',
      UNSUPPORTED: 'bg-red-100 text-red-800',
      CONTRADICTED: 'bg-red-200 text-red-900',
    };
    return (
      <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${colors[status] || 'bg-gray-100 text-gray-800'}`}>
        {status.replace(/_/g, ' ')}
      </span>
    );
  }

  return (
    <div className="max-w-3xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">AI Assistant</h1>
      <div className="bg-white rounded-lg shadow p-4 h-[500px] overflow-y-auto mb-4 space-y-4">
        {history.length === 0 && !loading && (
          <div className="text-gray-500 text-center mt-20">
            Ask me about asset history, work orders, or maintenance recommendations.
          </div>
        )}
        {history.map((msg, i) => (
          <div key={i} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
            <div className={`max-w-[80%] p-3 rounded-lg ${msg.role === 'user' ? 'bg-blue-600 text-white' : 'bg-gray-100 text-gray-800'}`}>
              <div className="text-sm whitespace-pre-wrap">{msg.text}</div>

              {/* Retrieval paths */}
              {msg.retrievalPaths && msg.retrievalPaths.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {msg.retrievalPaths.map((path) => (
                    <span key={path} className="text-xs px-1.5 py-0.5 rounded bg-blue-50 text-blue-700 font-mono">
                      {path}
                    </span>
                  ))}
                </div>
              )}

              {/* Verification status */}
              {msg.verificationStatus && (
                <div className="mt-2">
                  {verificationBadge(msg.verificationStatus)}
                </div>
              )}

              {/* Verified claims */}
              {msg.verifiedClaims && msg.verifiedClaims.length > 0 && (
                <div className="mt-2 pt-2 border-t border-gray-200">
                  <div className="text-xs font-medium text-gray-500 mb-1">Verified Claims:</div>
                  <div className="space-y-1">
                    {msg.verifiedClaims.map((claim, ci) => (
                      <div key={ci} className="text-xs">
                        <span className={claim.status === 'VERIFIED' ? 'text-green-700' : claim.status === 'CONTRADICTED' ? 'text-red-700' : 'text-yellow-700'}>
                          [{claim.status}] {claim.claim}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Sources */}
              {msg.sources && msg.sources.length > 0 && (
                <div className="mt-2 pt-2 border-t border-gray-200">
                  <div className="text-xs font-medium text-gray-500 mb-1">Sources:</div>
                  <div className="space-y-1.5">
                    {msg.sources.map((source) => (
                      <div key={source.id} className="text-xs border-l-2 border-blue-300 pl-2">
                        <div className="text-blue-700 font-medium">
                          {source.title}
                          {source.verified_by_sql && (
                            <span className="ml-1 text-green-600" title="Verified by SQL">&#10003;</span>
                          )}
                        </div>
                        <div className="text-gray-500 flex flex-wrap gap-x-2">
                          {source.type && <span>{source.type}</span>}
                          {source.retrieval_type && <span className="font-mono">{source.retrieval_type}</span>}
                          {source.source_scope && <span>{source.source_scope}</span>}
                          {source.timestamp && <span>{new Date(source.timestamp).toLocaleDateString()}</span>}
                        </div>
                        {source.quote && (
                          <div className="text-gray-600 italic mt-0.5">
                            &ldquo;{source.quote}&rdquo;
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        ))}
        {loading && streamingText && (
          <div className="flex justify-start">
            <div className="max-w-[80%] p-3 rounded-lg bg-gray-100 text-gray-800">
              <div className="text-sm whitespace-pre-wrap">{streamingText}</div>
              {/* Streaming paths */}
              {streamingPaths.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {streamingPaths.map((path) => (
                    <span key={path} className="text-xs px-1.5 py-0.5 rounded bg-blue-50 text-blue-700 font-mono">
                      {path}
                    </span>
                  ))}
                </div>
              )}
              {streamingVerification && (
                <div className="mt-2">{verificationBadge(streamingVerification)}</div>
              )}
              <div className="mt-1">
                <span className="inline-block w-2 h-2 bg-gray-400 rounded-full animate-pulse" />
              </div>
            </div>
          </div>
        )}
        {loading && !streamingText && (
          <div className="flex justify-start">
            <div className="bg-gray-100 p-3 rounded-lg">
              <div className="text-sm text-gray-500">Thinking...</div>
            </div>
          </div>
        )}
      </div>
      <div className="flex space-x-2">
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && send()}
          placeholder="Ask about Pump A maintenance history..."
          className="flex-1 rounded-md border border-gray-300 px-4 py-2"
          disabled={loading}
        />
        <button
          onClick={send}
          disabled={loading}
          className="bg-blue-600 text-white px-6 py-2 rounded-md hover:bg-blue-700 disabled:opacity-50"
        >
          Send
        </button>
      </div>
    </div>
  );
}
