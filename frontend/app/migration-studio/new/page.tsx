'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { apiFetch } from '@/lib/api';
import { Upload, ArrowLeft } from 'lucide-react';
import Link from 'next/link';

export default function NewMigrationJobPage() {
  const router = useRouter();
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [csvFile, setCsvFile] = useState<File | null>(null);
  const [jsonFile, setJsonFile] = useState<File | null>(null);

  const [form, setForm] = useState({
    name: '',
    source_system: 'csv',
    source_object_type: 'asset',
    description: '',
  });

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setSubmitting(true);
    setError(null);

    try {
      // 1. Create the migration job
      const createRes = await apiFetch('/migrations', {
        method: 'POST',
        body: JSON.stringify({
          name: form.name,
          source_system: form.source_system,
          source_object_type: form.source_object_type,
          description: form.description || undefined,
        }),
      });

      const job = createRes.data;
      const jobId = job.id;

      // 2. If a CSV file is provided, parse and upload
      if (csvFile) {
        const text = await csvFile.text();
        const records = parseCSV(text);
        await apiFetch(`/migrations/${jobId}/source-records`, {
          method: 'POST',
          body: JSON.stringify({ records }),
        });
      }

      // 3. If a JSON file is provided, parse and upload
      if (jsonFile) {
        const text = await jsonFile.text();
        const records = JSON.parse(text);
        const recordsArray = Array.isArray(records) ? records : [records];
        await apiFetch(`/migrations/${jobId}/source-records`, {
          method: 'POST',
          body: JSON.stringify({ records: recordsArray }),
        });
      }

      router.push(`/migration-studio/${jobId}`);
    } catch (e: any) {
      setError(e.message);
      setSubmitting(false);
    }
  }

  function parseCSV(text: string): any[] {
    const lines = text.trim().split('\n');
    if (lines.length < 2) return [];

    const headers = lines[0].split(',').map(h => h.trim().replace(/^"|"$/g, ''));
    const records: any[] = [];

    for (let i = 1; i < lines.length; i++) {
      const values = lines[i].split(',').map(v => v.trim().replace(/^"|"$/g, ''));
      if (values.length === headers.length && values.some(v => v !== '')) {
        const record: Record<string, string> = {};
        headers.forEach((h, idx) => { record[h] = values[idx] || ''; });
        records.push(record);
      }
    }

    return records;
  }

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <Link href="/migration-studio" className="flex items-center space-x-2 text-sm text-gray-500 hover:text-gray-700 mb-6">
        <ArrowLeft className="w-4 h-4" />
        <span>Back to Migration Studio</span>
      </Link>

      <h1 className="text-2xl font-bold text-gray-900 mb-6">New Migration Job</h1>

      {error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-3 text-red-700 text-sm mb-4">
          {error}
        </div>
      )}

      <form onSubmit={handleSubmit} className="space-y-5">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Job Name *</label>
          <input
            type="text"
            required
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            placeholder="e.g., Salesforce Asset Import Q2 2026"
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Source System</label>
            <select
              value={form.source_system}
              onChange={(e) => setForm({ ...form, source_system: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              <option value="csv">CSV File</option>
              <option value="json">JSON File</option>
              <option value="salesforce">Salesforce</option>
              <option value="servicenow">ServiceNow</option>
              <option value="maximo">IBM Maximo</option>
              <option value="custom">Custom</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Object Type</label>
            <select
              value={form.source_object_type}
              onChange={(e) => setForm({ ...form, source_object_type: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              <option value="asset">Asset / Equipment</option>
              <option value="work_order">Work Order / Ticket</option>
              <option value="location">Location / Site</option>
              <option value="part">Part / Inventory</option>
              <option value="contact">Contact / Customer</option>
              <option value="document">Document</option>
            </select>
          </div>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
          <textarea
            value={form.description}
            onChange={(e) => setForm({ ...form, description: e.target.value })}
            placeholder="Optional notes about this import..."
            rows={3}
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        <div className="border-t pt-5">
          <label className="block text-sm font-medium text-gray-700 mb-3">Upload Source File</label>
          <div className="grid grid-cols-2 gap-4">
            <div className="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center hover:border-blue-400 transition">
              <Upload className="w-8 h-8 mx-auto text-gray-400 mb-2" />
              <p className="text-sm font-medium text-gray-700">CSV File</p>
              <p className="text-xs text-gray-500 mt-1">.csv format</p>
              <input
                type="file"
                accept=".csv"
                onChange={(e) => setCsvFile(e.target.files?.[0] || null)}
                className="mt-3 text-sm text-gray-600 file:mr-4 file:py-1 file:px-3 file:rounded file:border-0 file:text-sm file:font-medium file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100"
              />
              {csvFile && <p className="text-xs text-green-600 mt-2">{csvFile.name}</p>}
            </div>
            <div className="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center hover:border-blue-400 transition">
              <Upload className="w-8 h-8 mx-auto text-gray-400 mb-2" />
              <p className="text-sm font-medium text-gray-700">JSON File</p>
              <p className="text-xs text-gray-500 mt-1">.json array format</p>
              <input
                type="file"
                accept=".json"
                onChange={(e) => setJsonFile(e.target.files?.[0] || null)}
                className="mt-3 text-sm text-gray-600 file:mr-4 file:py-1 file:px-3 file:rounded file:border-0 file:text-sm file:font-medium file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100"
              />
              {jsonFile && <p className="text-xs text-green-600 mt-2">{jsonFile.name}</p>}
            </div>
          </div>
        </div>

        <div className="flex items-center justify-end space-x-3 pt-4">
          <Link href="/migration-studio" className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50">
            Cancel
          </Link>
          <button
            type="submit"
            disabled={submitting}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {submitting ? 'Creating...' : 'Create Migration Job'}
          </button>
        </div>
      </form>
    </div>
  );
}
