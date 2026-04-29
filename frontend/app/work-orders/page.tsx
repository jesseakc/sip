'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { apiFetch } from '@/lib/api';

export default function WorkOrdersPage() {
  const [workOrders, setWorkOrders] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState('');

  useEffect(() => {
    apiFetch('/work-orders')
      .then((data) => setWorkOrders(data.data || []))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const filtered = filter
    ? workOrders.filter((wo) => wo.status === filter)
    : workOrders;

  if (loading) return <div>Loading work orders...</div>;

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Work Orders</h1>
      <div className="mb-4 flex space-x-2">
        <button onClick={() => setFilter('')} className={`px-3 py-1 rounded text-sm ${!filter ? 'bg-blue-600 text-white' : 'bg-gray-200'}`}>All</button>
        <button onClick={() => setFilter('OPEN')} className={`px-3 py-1 rounded text-sm ${filter === 'OPEN' ? 'bg-blue-600 text-white' : 'bg-gray-200'}`}>Open</button>
        <button onClick={() => setFilter('IN_PROGRESS')} className={`px-3 py-1 rounded text-sm ${filter === 'IN_PROGRESS' ? 'bg-blue-600 text-white' : 'bg-gray-200'}`}>In Progress</button>
        <button onClick={() => setFilter('COMPLETED')} className={`px-3 py-1 rounded text-sm ${filter === 'COMPLETED' ? 'bg-blue-600 text-white' : 'bg-gray-200'}`}>Completed</button>
      </div>
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <table className="min-w-full">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">WO #</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Title</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Status</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Priority</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Due</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {filtered.map((wo) => (
              <tr key={wo.id} className="hover:bg-gray-50">
                <td className="px-6 py-4">
                  <Link href={`/work-orders/${wo.id}`} className="text-blue-600 hover:underline font-medium">
                    {wo.display_number}
                  </Link>
                </td>
                <td className="px-6 py-4">{wo.title}</td>
                <td className="px-6 py-4">
                  <span className={`px-2 py-1 text-xs rounded ${getStatusColor(wo.status)}`}>{wo.status}</span>
                </td>
                <td className="px-6 py-4">{wo.priority}</td>
                <td className="px-6 py-4">{wo.due_at ? new Date(wo.due_at).toLocaleDateString() : '-'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function getStatusColor(status: string) {
  switch (status) {
    case 'OPEN': return 'bg-blue-100 text-blue-800';
    case 'ASSIGNED': return 'bg-purple-100 text-purple-800';
    case 'IN_PROGRESS': return 'bg-yellow-100 text-yellow-800';
    case 'ON_HOLD': return 'bg-orange-100 text-orange-800';
    case 'COMPLETED': return 'bg-green-100 text-green-800';
    case 'CLOSED': return 'bg-gray-100 text-gray-800';
    case 'CANCELLED': return 'bg-red-100 text-red-800';
    default: return 'bg-gray-100 text-gray-800';
  }
}
