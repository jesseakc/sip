'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { apiFetch } from '@/lib/api';

export default function DashboardPage() {
  const [stats, setStats] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    async function load() {
      try {
        const [assets, workOrders] = await Promise.all([
          apiFetch('/assets'),
          apiFetch('/work-orders'),
        ]);
        setStats({ assets: assets.data || [], workOrders: workOrders.data || [] });
      } catch (e) {
        console.error(e);
      } finally {
        setLoading(false);
      }
    }
    load();
  }, []);

  if (loading) return <div>Loading...</div>;

  const openWOs = stats?.workOrders?.filter((wo: any) => ['OPEN', 'ASSIGNED', 'IN_PROGRESS'].includes(wo.status)) || [];
  const overdueWOs = stats?.workOrders?.filter((wo: any) => wo.due_at && new Date(wo.due_at) < new Date() && !['COMPLETED', 'CLOSED', 'CANCELLED'].includes(wo.status)) || [];

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Dashboard</h1>
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
        <div className="bg-white p-6 rounded-lg shadow">
          <div className="text-sm text-gray-500">Total Assets</div>
          <div className="text-3xl font-bold">{stats?.assets?.length || 0}</div>
        </div>
        <div className="bg-white p-6 rounded-lg shadow">
          <div className="text-sm text-gray-500">Open Work Orders</div>
          <div className="text-3xl font-bold text-blue-600">{openWOs.length}</div>
        </div>
        <div className="bg-white p-6 rounded-lg shadow">
          <div className="text-sm text-gray-500">Overdue</div>
          <div className="text-3xl font-bold text-red-600">{overdueWOs.length}</div>
        </div>
        <div className="bg-white p-6 rounded-lg shadow">
          <div className="text-sm text-gray-500">Total Work Orders</div>
          <div className="text-3xl font-bold">{stats?.workOrders?.length || 0}</div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-white p-6 rounded-lg shadow">
          <h2 className="text-lg font-semibold mb-4">Recent Work Orders</h2>
          <div className="space-y-2">
            {stats?.workOrders?.slice(0, 5).map((wo: any) => (
              <Link key={wo.id} href={`/work-orders/${wo.id}`} className="block p-3 border rounded hover:bg-gray-50">
                <div className="flex justify-between">
                  <span className="font-medium">{wo.display_number}</span>
                  <span className={`text-xs px-2 py-1 rounded ${getStatusColor(wo.status)}`}>{wo.status}</span>
                </div>
                <div className="text-sm text-gray-600">{wo.title}</div>
              </Link>
            ))}
          </div>
        </div>

        <div className="bg-white p-6 rounded-lg shadow">
          <h2 className="text-lg font-semibold mb-4">Recent Assets</h2>
          <div className="space-y-2">
            {stats?.assets?.slice(0, 5).map((asset: any) => (
              <Link key={asset.id} href={`/assets/${asset.id}`} className="block p-3 border rounded hover:bg-gray-50">
                <div className="flex justify-between">
                  <span className="font-medium">{asset.name}</span>
                  <span className={`text-xs px-2 py-1 rounded ${getAssetStatusColor(asset.status)}`}>{asset.status}</span>
                </div>
              </Link>
            ))}
          </div>
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

function getAssetStatusColor(status: string) {
  switch (status) {
    case 'OPERATIONAL': return 'bg-green-100 text-green-800';
    case 'DEGRADED': return 'bg-yellow-100 text-yellow-800';
    case 'DOWN': return 'bg-red-100 text-red-800';
    case 'MAINTENANCE': return 'bg-blue-100 text-blue-800';
    default: return 'bg-gray-100 text-gray-800';
  }
}
