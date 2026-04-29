'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { apiFetch } from '@/lib/api';

export default function AssetsPage() {
  const [assets, setAssets] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    apiFetch('/assets')
      .then((data) => setAssets(data.data || []))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div>Loading assets...</div>;

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Assets</h1>
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <table className="min-w-full">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Name</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Status</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Criticality</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Type</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {assets.map((asset) => (
              <tr key={asset.id} className="hover:bg-gray-50">
                <td className="px-6 py-4">
                  <Link href={`/assets/${asset.id}`} className="text-blue-600 hover:underline font-medium">
                    {asset.name}
                  </Link>
                </td>
                <td className="px-6 py-4">
                  <span className={`px-2 py-1 text-xs rounded ${getAssetStatusColor(asset.status)}`}>{asset.status}</span>
                </td>
                <td className="px-6 py-4">{asset.criticality}</td>
                <td className="px-6 py-4">{asset.asset_type_id}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function getAssetStatusColor(status: string) {
  switch (status) {
    case 'OPERATIONAL': return 'bg-green-100 text-green-800';
    case 'DEGRADED': return 'bg-yellow-100 text-yellow-800';
    case 'DOWN': return 'bg-red-100 text-red-800';
    case 'MAINTENANCE': return 'bg-blue-100 text-blue-800';
    default: return 'bg-gray-100 text-gray-800';
  }
}
