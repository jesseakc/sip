'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { API_BASE, apiFetch } from '@/lib/api';
import { type LucideIcon, Plus, ArrowRight, FileText, AlertCircle } from 'lucide-react';

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

export default function MigrationStudioPage() {
  const [jobs, setJobs] = useState<MigrationJob[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    apiFetch('/migrations')
      .then((json) => setJobs(json.data || []))
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="p-6">
        <div className="animate-pulse space-y-4">
          {[1,2,3].map((i) => (
            <div key={i} className="h-20 bg-gray-100 rounded-lg" />
          ))}
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
          Failed to load migration jobs: {error}
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 max-w-5xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Migration Studio</h1>
          <p className="text-gray-500 text-sm mt-1">Import customer data from CRMs and CMMS systems</p>
        </div>
        <Link
          href="/migration-studio/new"
          className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 font-medium text-sm"
        >
          <Plus className="w-4 h-4" />
          <span>New Migration Job</span>
        </Link>
      </div>

      {/* Job List */}
      {jobs.length === 0 ? (
        <div className="text-center py-16 bg-white rounded-lg border border-gray-200">
          <FileText className="w-12 h-12 mx-auto text-gray-300" />
          <h3 className="mt-4 text-lg font-medium text-gray-900">No migration jobs yet</h3>
          <p className="mt-1 text-sm text-gray-500">Create a new migration job to start importing data.</p>
          <Link
            href="/migration-studio/new"
            className="mt-4 inline-flex items-center space-x-2 text-blue-600 hover:text-blue-700 font-medium text-sm"
          >
            <Plus className="w-4 h-4" />
            <span>Create your first job</span>
          </Link>
        </div>
      ) : (
        <div className="space-y-3">
          {jobs.map((job) => (
            <Link
              key={job.id}
              href={`/migration-studio/${job.id}`}
              className="block bg-white rounded-lg border border-gray-200 hover:border-blue-300 hover:shadow-sm transition p-4"
            >
              <div className="flex items-center justify-between">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center space-x-3">
                    <h3 className="text-sm font-semibold text-gray-900 truncate">{job.name}</h3>
                    <span className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${STATUS_COLORS[job.status] || 'bg-gray-100 text-gray-700'}`}>
                      {job.status.replace(/_/g, ' ')}
                    </span>
                  </div>
                  <div className="flex items-center space-x-4 mt-1 text-xs text-gray-500">
                    <span>Source: {job.source_system}</span>
                    <span>Records: {job.source_record_count}</span>
                    {job.imported_record_count > 0 && (
                      <span className="text-green-600">Imported: {job.imported_record_count}</span>
                    )}
                    {job.error_count > 0 && (
                      <span className="text-red-600">Errors: {job.error_count}</span>
                    )}
                  </div>
                </div>
                <ArrowRight className="w-5 h-5 text-gray-300 flex-shrink-0 ml-4" />
              </div>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
