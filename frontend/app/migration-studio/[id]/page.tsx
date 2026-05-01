'use client';

import { useEffect, useState, useCallback } from 'react';
import { useParams, useRouter } from 'next/navigation';
import Link from 'next/link';
import { apiFetch } from '@/lib/api';
import {
  ArrowLeft, Play, FileCheck, AlertTriangle, CheckCircle,
  XCircle, Clock, RotateCcw, Ban, Download, Eye, Loader2,
} from 'lucide-react';

interface MigrationJob {
  id: string;
  name: string;
  description?: string;
  source_system: string;
  source_object_type: string;
  status: string;
  source_record_count: number;
  valid_record_count: number;
  imported_record_count: number;
  error_count: number;
  created_at: string;
  updated_at: string;
}

interface MigrationReport {
  job: MigrationJob;
  import_results: Array<{ action: string; sip_entity_type: string; error_message?: string }>;
  validation_issue_count: number;
  duplicate_candidate_count: number;
  external_id_map_count: number;
}

const STEPS = [
  { key: 'uploaded', label: 'Source Records', icon: FileCheck },
  { key: 'mapped', label: 'Field Mapping', icon: FileCheck },
  { key: 'validated', label: 'Validation', icon: AlertTriangle },
  { key: 'ready_for_import', label: 'Dry Run', icon: Play },
  { key: 'importing', label: 'Import', icon: Loader2 },
  { key: 'completed', label: 'Report', icon: Download },
];

const STATUS_COLORS: Record<string, string> = {
  draft: 'bg-gray-100 text-gray-700',
  uploaded: 'bg-blue-50 text-blue-700',
  mapped: 'bg-indigo-50 text-indigo-700',
  validated: 'bg-purple-50 text-purple-700',
  ready_for_import: 'bg-teal-50 text-teal-700',
  importing: 'bg-yellow-50 text-yellow-700',
  completed: 'bg-green-50 text-green-700',
  completed_with_warnings: 'bg-lime-50 text-lime-700',
  failed: 'bg-red-50 text-red-700',
  cancelled: 'bg-gray-100 text-gray-500',
  rolled_back: 'bg-orange-50 text-orange-700',
};

export default function MigrationJobDetailPage() {
  const params = useParams();
  const router = useRouter();
  const jobId = params.id as string;

  const [job, setJob] = useState<MigrationJob | null>(null);
  const [sourceRecords, setSourceRecords] = useState<any[]>([]);
  const [stagedRecords, setStagedRecords] = useState<any[]>([]);
  const [mappings, setMappings] = useState<any[]>([]);
  const [issues, setIssues] = useState<any[]>([]);
  const [duplicates, setDuplicates] = useState<any[]>([]);
  const [report, setReport] = useState<MigrationReport | null>(null);
  const [activeTab, setActiveTab] = useState('source');
  const [loading, setLoading] = useState(true);
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [mappingForm, setMappingForm] = useState<Array<{ source_field: string; target_field: string }>>([]);

  const fetchJob = useCallback(async () => {
    try {
      const json = await apiFetch(`/migrations/jobs/${jobId}`);
      setJob(json.data);
      return json.data;
    } catch (e: any) {
      setError(e.message);
      return null;
    }
  }, [jobId]);

  useEffect(() => {
    (async () => {
      setLoading(true);
      await fetchJob();
      // Fetch source records
      try {
        const src = await apiFetch(`/migrations/jobs/${jobId}/source-records`);
        setSourceRecords(src.data || []);
        // Auto-populate mapping form from source field names
        if (src.data?.length > 0) {
          const fields = Object.keys(src.data[0]);
          setMappingForm(fields.map(f => ({ source_field: f, target_field: '' })));
        }
      } catch {}
      // Fetch staged records
      try {
        const staged = await apiFetch(`/migrations/jobs/${jobId}/staged-records`);
        setStagedRecords(staged.data || []);
      } catch {}
      // Fetch mappings
      try {
        const maps = await apiFetch(`/migrations/jobs/${jobId}/field-mappings`);
        setMappings(maps.data || []);
      } catch {}
      // Fetch issues
      try {
        const issueData = await apiFetch(`/migrations/jobs/${jobId}/validation-issues`);
        setIssues(issueData.data || []);
      } catch {}
      // Fetch duplicates
      try {
        const dupData = await apiFetch(`/migrations/jobs/${jobId}/duplicates`);
        setDuplicates(dupData.data || []);
      } catch {}
      setLoading(false);
    })();
  }, [jobId, fetchJob]);

  async function handleAction(action: string, endpoint: string) {
    setActionLoading(action);
    setError(null);
    try {
      await apiFetch(endpoint, { method: 'POST' });
      await fetchJob();
      if (action === 'validate') {
        // Refresh issues after validation
        try {
          const issueData = await apiFetch(`/migrations/jobs/${jobId}/validation-issues`);
          setIssues(issueData.data || []);
        } catch {}
      }
      if (action === 'execute') {
        // Fetch report after import
        try {
          const reportData = await apiFetch(`/migrations/jobs/${jobId}/report`);
          setReport(reportData.data);
        } catch {}
      }
    } catch (e: any) {
      setError(e.message);
    }
    setActionLoading(null);
  }

  async function handleSaveMappings() {
    setActionLoading('mapping');
    setError(null);
    try {
      const validMappings = mappingForm
        .filter(m => m.target_field)
        .map(m => ({
          id: crypto.randomUUID(),
          organization_id: '', // filled by server
          job_id: jobId,
          target_entity_type: job?.source_object_type || 'asset',
          source_field: m.source_field,
          target_field: m.target_field,
          transform_expression: null,
          default_value: null,
          is_required: false,
          created_at: new Date().toISOString(),
        }));

      if (validMappings.length > 0) {
        await apiFetch(`/migrations/jobs/${jobId}/field-mappings`, {
          method: 'POST',
          body: JSON.stringify({ mappings: validMappings }),
        });
      }
      await fetchJob();
    } catch (e: any) {
      setError(e.message);
    }
    setActionLoading(null);
  }

  if (loading) {
    return (
      <div className="p-6 max-w-5xl mx-auto">
        <div className="animate-pulse space-y-4">
          <div className="h-8 bg-gray-100 rounded w-1/3" />
          <div className="h-64 bg-gray-100 rounded-lg" />
        </div>
      </div>
    );
  }

  if (!job) {
    return (
      <div className="p-6 max-w-5xl mx-auto">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
          Migration job not found.
        </div>
        <Link href="/migration-studio" className="mt-4 inline-flex items-center text-blue-600">
          <ArrowLeft className="w-4 h-4 mr-1" /> Back to Migration Studio
        </Link>
      </div>
    );
  }

  const canMap = ['uploaded', 'mapped', 'draft'].includes(job.status);
  const canValidate = ['uploaded', 'mapped', 'validated'].includes(job.status);
  const canDryRun = ['validated', 'ready_for_import'].includes(job.status);
  const canExecute = ['validated', 'ready_for_import'].includes(job.status);
  const canCancel = !['completed', 'completed_with_warnings', 'failed', 'cancelled', 'rolled_back'].includes(job.status);
  const canRollback = ['completed', 'completed_with_warnings', 'failed'].includes(job.status);

  const recordFields = sourceRecords.length > 0 ? Object.keys(sourceRecords[0]) : [];
  const targetFields = [
    'name', 'serial_number', 'description', 'status', 'criticality',
    'asset_type_id', 'model_id', 'manufacturer_id', 'location_id',
    'purchase_date', 'warranty_expiry', 'notes', 'tags',
    'firmware_version', 'software_version', 'hardware_version',
    'title', 'priority', 'type', 'resolution_notes', 'due_date',
  ];

  return (
    <div className="p-6 max-w-5xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <Link href="/migration-studio" className="flex items-center space-x-2 text-sm text-gray-500 hover:text-gray-700 mb-2">
            <ArrowLeft className="w-4 h-4" />
            <span>Migration Studio</span>
          </Link>
          <h1 className="text-2xl font-bold text-gray-900">{job.name}</h1>
          <div className="flex items-center space-x-3 mt-1">
            <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${STATUS_COLORS[job.status]}`}>
              {job.status.replace(/_/g, ' ')}
            </span>
            <span className="text-xs text-gray-500">{job.source_record_count} records</span>
          </div>
        </div>
        {/* Action Buttons */}
        <div className="flex items-center space-x-2">
          {canCancel && (
            <button
              onClick={() => handleAction('cancel', `/migrations/jobs/${jobId}/cancel`)}
              disabled={actionLoading === 'cancel'}
              className="flex items-center space-x-1 px-3 py-1.5 text-sm font-medium text-red-700 bg-red-50 border border-red-200 rounded-md hover:bg-red-100 disabled:opacity-50"
            >
              {actionLoading === 'cancel' ? <Loader2 className="w-4 h-4 animate-spin" /> : <Ban className="w-4 h-4" />}
              <span>Cancel</span>
            </button>
          )}
          {canRollback && (
            <button
              onClick={() => handleAction('rollback', `/migrations/jobs/${jobId}/rollback`)}
              disabled={actionLoading === 'rollback'}
              className="flex items-center space-x-1 px-3 py-1.5 text-sm font-medium text-orange-700 bg-orange-50 border border-orange-200 rounded-md hover:bg-orange-100 disabled:opacity-50"
            >
              {actionLoading === 'rollback' ? <Loader2 className="w-4 h-4 animate-spin" /> : <RotateCcw className="w-4 h-4" />}
              <span>Rollback</span>
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-3 text-red-700 text-sm mb-4">
          {error}
        </div>
      )}

      {/* Progress Steps */}
      <div className="flex items-center mb-6 overflow-x-auto">
        {STEPS.map((step, idx) => {
          const isComplete = STEPS.findIndex(s => s.key === job.status) >= STEPS.findIndex(s => s.key === step.key || (step.key === 'completed' && ['completed', 'completed_with_warnings'].includes(job.status)));
          const isCurrent = job.status === step.key || (step.key === 'completed' && ['completed', 'completed_with_warnings'].includes(job.status));
          const Icon = step.icon;
          return (
            <div key={step.key} className="flex items-center flex-shrink-0">
              <div className={`flex items-center space-x-1.5 ${idx > 0 ? 'ml-2' : ''}`}>
                <div className={`w-6 h-6 rounded-full flex items-center justify-center ${
                  isComplete ? 'bg-green-500 text-white' : isCurrent ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-500'
                }`}>
                  {isComplete ? <CheckCircle className="w-4 h-4" /> : <span className="text-xs">{idx + 1}</span>}
                </div>
                <span className={`text-xs font-medium ${isComplete ? 'text-green-700' : isCurrent ? 'text-blue-700' : 'text-gray-400'}`}>
                  {step.label}
                </span>
              </div>
              {idx < STEPS.length - 1 && (
                <div className={`w-6 h-0.5 mx-1 ${isComplete ? 'bg-green-300' : 'bg-gray-200'}`} />
              )}
            </div>
          );
        })}
      </div>

      {/* Tab Navigation */}
      <div className="flex border-b border-gray-200 mb-6">
        {['source', 'mapping', 'validation', 'dryrun', 'report'].map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-4 py-2 text-sm font-medium border-b-2 -mb-px ${
              activeTab === tab ? 'border-blue-500 text-blue-600' : 'border-transparent text-gray-500 hover:text-gray-700'
            }`}
          >
            {tab === 'source' && 'Source Records'}
            {tab === 'mapping' && 'Field Mapping'}
            {tab === 'validation' && 'Validation'}
            {tab === 'dryrun' && 'Dry Run'}
            {tab === 'report' && 'Report'}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      {activeTab === 'source' && (
        <div>
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-900">
              Source Records ({sourceRecords.length})
            </h3>
          </div>
          {sourceRecords.length === 0 ? (
            <div className="text-center py-12 bg-gray-50 rounded-lg">
              <FileCheck className="w-10 h-10 mx-auto text-gray-300" />
              <p className="mt-2 text-sm text-gray-500">No source records uploaded yet.</p>
            </div>
          ) : (
            <div className="overflow-x-auto bg-white rounded-lg border border-gray-200">
              <table className="w-full text-sm">
                <thead>
                  <tr className="bg-gray-50 border-b">
                    {recordFields.slice(0, 8).map((field) => (
                      <th key={field} className="px-3 py-2 text-left font-medium text-gray-600 text-xs uppercase">
                        {field}
                      </th>
                    ))}
                    {recordFields.length > 8 && <th className="px-3 py-2 text-left font-medium text-gray-400 text-xs">...</th>}
                  </tr>
                </thead>
                <tbody>
                  {sourceRecords.slice(0, 20).map((rec, idx) => (
                    <tr key={idx} className="border-b last:border-0 hover:bg-gray-50">
                      {recordFields.slice(0, 8).map((field) => (
                        <td key={field} className="px-3 py-1.5 text-gray-700 max-w-[200px] truncate">
                          {String(rec[field] ?? '')}
                        </td>
                      ))}
                      {recordFields.length > 8 && <td className="px-3 py-1.5 text-gray-400">...</td>}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}

      {activeTab === 'mapping' && (
        <div>
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-900">Field Mapping</h3>
            {canMap && (
              <button
                onClick={handleSaveMappings}
                disabled={actionLoading === 'mapping'}
                className="flex items-center space-x-1 px-3 py-1.5 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:opacity-50"
              >
                {actionLoading === 'mapping' ? <Loader2 className="w-4 h-4 animate-spin" /> : <FileCheck className="w-4 h-4" />}
                <span>Save & Validate</span>
              </button>
            )}
          </div>
          <p className="text-xs text-gray-500 mb-4">
            Map source fields to SIP canonical fields. Only mapped fields with a target field selected will be saved.
          </p>
          {recordFields.length === 0 ? (
            <div className="text-center py-12 bg-gray-50 rounded-lg">
              <p className="text-sm text-gray-500">Upload source records first to enable field mapping.</p>
            </div>
          ) : (
            <div className="bg-white rounded-lg border border-gray-200 p-4 space-y-2">
              {mappingForm.map((mapping, idx) => (
                <div key={idx} className="flex items-center space-x-3">
                  <div className="w-48 flex-shrink-0">
                    <input
                      type="text"
                      value={mapping.source_field}
                      readOnly
                      className="w-full px-2 py-1.5 bg-gray-50 border border-gray-200 rounded text-xs font-mono text-gray-600"
                    />
                  </div>
                  <span className="text-gray-400">→</span>
                  <select
                    value={mapping.target_field}
                    onChange={(e) => {
                      const updated = [...mappingForm];
                      updated[idx].target_field = e.target.value;
                      setMappingForm(updated);
                    }}
                    className="flex-1 px-2 py-1.5 border border-gray-300 rounded text-xs focus:ring-2 focus:ring-blue-500"
                  >
                    <option value="">-- Skip --</option>
                    <optgroup label="Asset Fields">
                      {['name', 'serial_number', 'description', 'status', 'criticality', 'purchase_date', 'warranty_expiry', 'notes', 'tags', 'firmware_version'].map(f => (
                        <option key={f} value={f}>{f}</option>
                      ))}
                    </optgroup>
                    <optgroup label="Work Order Fields">
                      {['title', 'description', 'priority', 'type', 'status', 'resolution_notes', 'due_date', 'completed_date', 'labor_hours'].map(f => (
                        <option key={f} value={f}>{f}</option>
                      ))}
                    </optgroup>
                    <optgroup label="Location Fields">
                      {['name', 'address_line1', 'city', 'state', 'postal_code', 'country'].map(f => (
                        <option key={f} value={f}>{f}</option>
                      ))}
                    </optgroup>
                  </select>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {activeTab === 'validation' && (
        <div>
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-900">
              Validation Issues ({issues.length})
            </h3>
            {canValidate && (
              <button
                onClick={() => handleAction('validate', `/migrations/jobs/${jobId}/validate`)}
                disabled={actionLoading === 'validate'}
                className="flex items-center space-x-1 px-3 py-1.5 text-sm font-medium text-white bg-purple-600 rounded-md hover:bg-purple-700 disabled:opacity-50"
              >
                {actionLoading === 'validate' ? <Loader2 className="w-4 h-4 animate-spin" /> : <AlertTriangle className="w-4 h-4" />}
                <span>Validate</span>
              </button>
            )}
          </div>
          {issues.length === 0 ? (
            <div className="text-center py-12 bg-gray-50 rounded-lg">
              <CheckCircle className="w-10 h-10 mx-auto text-green-300" />
              <p className="mt-2 text-sm text-gray-500">
                {job.status === 'validated' || job.status === 'ready_for_import' ? 'No validation issues found.' : 'Run validation to check for issues.'}
              </p>
            </div>
          ) : (
            <div className="space-y-2">
              {issues.map((issue: any, idx: number) => (
                <div key={idx} className={`bg-white rounded-lg border p-3 ${
                  issue.severity === 'error' ? 'border-red-200 bg-red-50' : issue.severity === 'warning' ? 'border-yellow-200 bg-yellow-50' : 'border-blue-200 bg-blue-50'
                }`}>
                  <div className="flex items-center space-x-2">
                    {issue.severity === 'error' ? <XCircle className="w-4 h-4 text-red-500" /> : issue.severity === 'warning' ? <AlertTriangle className="w-4 h-4 text-yellow-500" /> : <Eye className="w-4 h-4 text-blue-500" />}
                    <span className="text-sm font-medium">{issue.message}</span>
                  </div>
                  {issue.field && <p className="text-xs text-gray-500 mt-1 ml-6">Field: {issue.field}</p>}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {activeTab === 'dryrun' && (
        <div>
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-900">Dry Run</h3>
            {canDryRun && (
              <button
                onClick={() => handleAction('dryrun', `/migrations/jobs/${jobId}/dry-run`)}
                disabled={actionLoading === 'dryrun'}
                className="flex items-center space-x-1 px-3 py-1.5 text-sm font-medium text-white bg-teal-600 rounded-md hover:bg-teal-700 disabled:opacity-50"
              >
                {actionLoading === 'dryrun' ? <Loader2 className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4" />}
                <span>Run Dry Run</span>
              </button>
            )}
          </div>
          <p className="text-xs text-gray-500 mb-4">
            Dry run validates and counts records without making permanent changes.
          </p>
          <div className="grid grid-cols-2 gap-4">
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <p className="text-xs text-gray-500">Source Records</p>
              <p className="text-2xl font-bold text-gray-900">{job.source_record_count}</p>
            </div>
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <p className="text-xs text-gray-500">Valid Records</p>
              <p className="text-2xl font-bold text-green-600">{job.valid_record_count}</p>
            </div>
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <p className="text-xs text-gray-500">Imported Records</p>
              <p className="text-2xl font-bold text-blue-600">{job.imported_record_count}</p>
            </div>
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <p className="text-xs text-gray-500">Errors</p>
              <p className="text-2xl font-bold text-red-600">{job.error_count}</p>
            </div>
          </div>
        </div>
      )}

      {activeTab === 'report' && (
        <div>
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-900">Import Report</h3>
            {canExecute && (
              <button
                onClick={() => handleAction('execute', `/migrations/jobs/${jobId}/execute`)}
                disabled={actionLoading === 'execute'}
                className="flex items-center space-x-1 px-4 py-2 text-sm font-medium text-white bg-green-600 rounded-md hover:bg-green-700 disabled:opacity-50 shadow-sm"
              >
                {actionLoading === 'execute' ? <Loader2 className="w-4 h-4 animate-spin" /> : <Play className="w-4 h-4" />}
                <span>Execute Import</span>
              </button>
            )}
          </div>
          <p className="text-xs text-gray-500 mb-4">
            {canExecute ? 'Ready to import. This will create permanent SIP records.' : 'Import report summary.'}
          </p>
          <div className="grid grid-cols-3 gap-4 mb-4">
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <p className="text-xs text-gray-500">Created</p>
              <p className="text-2xl font-bold text-green-600">
                {report?.import_results?.filter((r: any) => r.action === 'created').length || job.imported_record_count}
              </p>
            </div>
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <p className="text-xs text-gray-500">Skipped</p>
              <p className="text-2xl font-bold text-gray-400">
                {report?.import_results?.filter((r: any) => r.action === 'skipped').length || 0}
              </p>
            </div>
            <div className="bg-white rounded-lg border border-gray-200 p-4">
              <p className="text-xs text-gray-500">Failed</p>
              <p className="text-2xl font-bold text-red-600">
                {report?.import_results?.filter((r: any) => r.action === 'failed').length || job.error_count}
              </p>
            </div>
          </div>
          {job.status === 'completed' || job.status === 'completed_with_warnings' ? (
            <div className="bg-green-50 border border-green-200 rounded-lg p-4">
              <div className="flex items-center space-x-2">
                <CheckCircle className="w-5 h-5 text-green-600" />
                <span className="font-medium text-green-800">Import Complete</span>
              </div>
              <p className="text-sm text-green-700 mt-1">
                {job.imported_record_count} records imported successfully.
                {job.error_count > 0 && ` ${job.error_count} errors encountered.`}
              </p>
            </div>
          ) : job.status === 'failed' ? (
            <div className="bg-red-50 border border-red-200 rounded-lg p-4">
              <div className="flex items-center space-x-2">
                <XCircle className="w-5 h-5 text-red-600" />
                <span className="font-medium text-red-800">Import Failed</span>
              </div>
              <p className="text-sm text-red-700 mt-1">Review errors and try again.</p>
            </div>
          ) : null}
        </div>
      )}
    </div>
  );
}
