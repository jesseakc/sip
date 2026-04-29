'use client';

import { useEffect, useState } from 'react';
import { apiFetch } from '@/lib/api';

export default function TeamsPage() {
  const [teams, setTeams] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    apiFetch('/teams')
      .then((data) => setTeams(data.data || []))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div>Loading teams...</div>;

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Teams</h1>
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <table className="min-w-full">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Name</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Description</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Skills</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200">
            {teams.map((team) => (
              <tr key={team.id} className="hover:bg-gray-50">
                <td className="px-6 py-4 font-medium">{team.name}</td>
                <td className="px-6 py-4">{team.description || '-'}</td>
                <td className="px-6 py-4">
                  {team.skills?.length > 0 ? (
                    <div className="flex flex-wrap gap-1">
                      {team.skills.map((skill: string, i: number) => (
                        <span key={i} className="px-2 py-1 text-xs bg-gray-100 text-gray-700 rounded">{skill}</span>
                      ))}
                    </div>
                  ) : '-'}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
