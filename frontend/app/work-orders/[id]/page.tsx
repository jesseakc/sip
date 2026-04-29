'use client';

import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { apiFetch } from '@/lib/api';

export default function WorkOrderDetailPage() {
  const { id } = useParams();
  const [wo, setWo] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!id) return;
    apiFetch(`/work-orders/${id}`)
      .then((data) => setWo(data))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [id]);

  async function transition(status: string, notes?: string) {
    try {
      await apiFetch(`/work-orders/${id}`, {
        method: 'PATCH',
        body: JSON.stringify({ status, resolution_notes: notes }),
      });
      const updated = await apiFetch(`/work-orders/${id}`);
      setWo(updated);
    } catch (e: any) {
      alert(e.message);
    }
  }

  if (loading) return <div>Loading...</div>;
  if (!wo) return <div>Work order not found</div>;

  return (
    <div>
      <Link href="/work-orders" className="text-blue-600 hover:underline">&larr; Back to Work Orders</Link>
      <h1 className="text-2xl font-bold mt-4 mb-2">{wo.display_number}</h1>
      <p className="text-gray-600 mb-6">{wo.title}</p>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
        <div className="bg-white p-4 rounded shadow">
          <div className="text-sm text-gray-500">Status</div>
          <div className="text-lg font-semibold">{wo.status}</div>
        </div>
        <div className="bg-white p-4 rounded shadow">
          <div className="text-sm text-gray-500">Priority</div>
          <div className="text-lg font-semibold">{wo.priority}</div>
        </div>
        <div className="bg-white p-4 rounded shadow">
          <div className="text-sm text-gray-500">Type</div>
          <div className="text-lg font-semibold">{wo.type}</div>
        </div>
        <div className="bg-white p-4 rounded shadow">
          <div className="text-sm text-gray-500">Due</div>
          <div className="text-lg font-semibold">{wo.due_at ? new Date(wo.due_at).toLocaleDateString() : '-'}</div>
        </div>
      </div>

      <div className="bg-white p-6 rounded shadow mb-6">
        <h2 className="text-lg font-semibold mb-4">Description</h2>
        <p className="text-gray-700">{wo.description}</p>
      </div>

      {wo.resolution_notes && (
        <div className="bg-white p-6 rounded shadow mb-6">
          <h2 className="text-lg font-semibold mb-4">Resolution Notes</h2>
          <p className="text-gray-700">{wo.resolution_notes}</p>
        </div>
      )}

      <div className="bg-white p-6 rounded shadow mb-6">
        <h2 className="text-lg font-semibold mb-4">Actions</h2>
        <div className="space-x-2">
          {wo.status === 'DRAFT' && (
            <button onClick={() => transition('OPEN')} className="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">Publish</button>
          )}
          {wo.status === 'OPEN' && (
            <button onClick={() => transition('ASSIGNED')} className="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">Assign</button>
          )}
          {wo.status === 'ASSIGNED' && (
            <button onClick={() => transition('ACCEPTED')} className="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">Accept</button>
          )}
          {wo.status === 'ACCEPTED' && (
            <button onClick={() => transition('IN_PROGRESS')} className="px-3 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700">Start</button>
          )}
          {wo.status === 'IN_PROGRESS' && (
            <>
              <button onClick={() => transition('ON_HOLD')} className="px-3 py-2 bg-orange-600 text-white rounded hover:bg-orange-700">Hold</button>
              <button onClick={() => transition('COMPLETED', prompt('Resolution notes:') || '')} className="px-3 py-2 bg-green-600 text-white rounded hover:bg-green-700">Complete</button>
            </>
          )}
          {wo.status === 'ON_HOLD' && (
            <button onClick={() => transition('IN_PROGRESS')} className="px-3 py-2 bg-yellow-600 text-white rounded hover:bg-yellow-700">Resume</button>
          )}
          {wo.status === 'COMPLETED' && (
            <button onClick={() => transition('REVIEWED')} className="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">Review</button>
          )}
          {wo.status === 'REVIEWED' && (
            <button onClick={() => transition('CLOSED')} className="px-3 py-2 bg-gray-600 text-white rounded hover:bg-gray-700">Close</button>
          )}
          {wo.status === 'CLOSED' && (
            <button onClick={() => transition('OPEN')} className="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">Reopen</button>
          )}
          {!['CANCELLED', 'CLOSED'].includes(wo.status) && (
            <button onClick={() => transition('CANCELLED', 'Cancelled by user')} className="px-3 py-2 bg-red-600 text-white rounded hover:bg-red-700">Cancel</button>
          )}
        </div>
      </div>
    </div>
  );
}
