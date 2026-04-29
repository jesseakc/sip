'use client';

import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { apiFetch } from '@/lib/api';

export default function AssetDetailPage() {
  const { id } = useParams();
  const [asset, setAsset] = useState<any>(null);
  const [workOrders, setWorkOrders] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!id) return;
    apiFetch(`/assets/${id}`)
      .then((data) => {
        setAsset(data);
        // For MVP, fetch all WOs and filter client-side
        return apiFetch('/work-orders');
      })
      .then((data) => {
        const all = data.data || [];
        setWorkOrders(all.filter((wo: any) => wo.asset_id === id));
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [id]);

  async function updateStatus(newStatus: string) {
    try {
      await apiFetch(`/assets/${id}`, {
        method: 'PATCH',
        body: JSON.stringify({ status: newStatus }),
      });
      const updated = await apiFetch(`/assets/${id}`);
      setAsset(updated);
    } catch (e: any) {
      alert(e.message);
    }
  }

  if (loading) return <div>Loading...</div>;
  if (!asset) return <div>Asset not found</div>;

  return (
    <div>
      <Link href="/assets" className="text-blue-600 hover:underline">&larr; Back to Assets</Link>
      <h1 className="text-2xl font-bold mt-4 mb-2">{asset.name}</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div className="bg-white p-4 rounded shadow">
          <div className="text-sm text-gray-500">Status</div>
          <div className="text-lg font-semibold">{asset.status}</div>
          <div className="mt-2 space-x-2">
            {asset.status === 'OPERATIONAL' && (
              <button onClick={() => updateStatus('DEGRADED')} className="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded">Mark Degraded</button>
            )}
            {asset.status === 'DEGRADED' && (
              <button onClick={() => updateStatus('MAINTENANCE')} className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">Maintenance</button>
            )}
            {asset.status === 'MAINTENANCE' && (
              <button onClick={() => updateStatus('OPERATIONAL')} className="text-xs bg-green-100 text-green-800 px-2 py-1 rounded">Operational</button>
            )}
          </div>
        </div>
        <div className="bg-white p-4 rounded shadow">
          <div className="text-sm text-gray-500">Criticality</div>
          <div className="text-lg font-semibold">{asset.criticality}</div>
        </div>
        <div className="bg-white p-4 rounded shadow">
          <div className="text-sm text-gray-500">Serial Number</div>
          <div className="text-lg font-semibold">{asset.serial_number || 'N/A'}</div>
        </div>
      </div>

      <div className="bg-white p-6 rounded shadow mb-6">
        <h2 className="text-lg font-semibold mb-4">Description</h2>
        <p className="text-gray-700">{asset.description || 'No description'}</p>
        {asset.attributes && (
          <pre className="mt-4 bg-gray-50 p-3 rounded text-sm">{JSON.stringify(asset.attributes, null, 2)}</pre>
        )}
      </div>

      <div className="bg-white p-6 rounded shadow">
        <h2 className="text-lg font-semibold mb-4">Work Orders</h2>
        <div className="space-y-2">
          {workOrders.map((wo) => (
            <Link key={wo.id} href={`/work-orders/${wo.id}`} className="block p-3 border rounded hover:bg-gray-50">
              <div className="flex justify-between">
                <span className="font-medium">{wo.display_number}</span>
                <span className={`text-xs px-2 py-1 rounded ${getStatusColor(wo.status)}`}>{wo.status}</span>
              </div>
              <div className="text-sm text-gray-600">{wo.title}</div>
            </Link>
          ))}
          {workOrders.length === 0 && <p className="text-gray-500">No work orders</p>}
        </div>
      </div>
    </div>
  );
}

function getStatusColor(status: string) {
  switch (status) {
    case 'OPEN': return 'bg-blue-100 text-blue-800';
    case 'IN_PROGRESS': return 'bg-yellow-100 text-yellow-800';
    case 'COMPLETED': return 'bg-green-100 text-green-800';
    case 'CLOSED': return 'bg-gray-100 text-gray-800';
    default: return 'bg-gray-100 text-gray-800';
  }
}
