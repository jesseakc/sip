'use client';

import { useEffect, useState } from 'react';
import { apiFetch } from '@/lib/api';

export default function SettingsPage() {
  const [org, setOrg] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    apiFetch('/organizations/me')
      .then((data) => setOrg(data))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div>Loading settings...</div>;

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Settings</h1>
      <div className="bg-white rounded-lg shadow p-6 max-w-2xl">
        <h2 className="text-lg font-semibold mb-4">Organization</h2>
        {org ? (
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-500">Name</label>
              <div className="mt-1 text-gray-900">{org.name}</div>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-500">Timezone</label>
              <div className="mt-1 text-gray-900">{org.timezone || '-'}</div>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-500">Default Currency</label>
              <div className="mt-1 text-gray-900">{org.default_currency || '-'}</div>
            </div>
          </div>
        ) : (
          <p className="text-gray-500">No organization data available.</p>
        )}
      </div>
    </div>
  );
}
