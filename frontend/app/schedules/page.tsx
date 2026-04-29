'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { apiFetch } from '@/lib/api';

export default function SchedulesPage() {
  const [schedules, setSchedules] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    apiFetch('/schedules')
      .then((data) => setSchedules(data.data || []))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div>Loading schedules...</div>;

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Schedules</h1>
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <table className="min-w-full">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Name</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Asset ID</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Trigger Type</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Enabled</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Next Due</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {schedules.map((schedule) => (
              <tr key={schedule.id} className="hover:bg-gray-50">
                <td className="px-6 py-4">
                  <Link href={`/schedules/${schedule.id}`} className="text-blue-600 hover:underline font-medium">
                    {schedule.name}
                  </Link>
                </td>
                <td className="px-6 py-4">{schedule.asset_id || '-'}</td>
                <td className="px-6 py-4">{schedule.trigger_type || '-'}</td>
                <td className="px-6 py-4">{schedule.enabled ? 'Yes' : 'No'}</td>
                <td className="px-6 py-4">{schedule.next_due ? new Date(schedule.next_due).toLocaleString() : '-'}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
