'use client';

import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import Link from 'next/link';
import { apiFetch } from '@/lib/api';

export default function LocationDetailPage() {
  const { id } = useParams();
  const [location, setLocation] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!id) return;
    apiFetch(`/locations/${id}`)
      .then((data) => setLocation(data))
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) return <div>Loading...</div>;
  if (!location) return <div>Location not found</div>;

  return (
    <div>
      <Link href="/locations" className="text-blue-600 hover:underline">&larr; Back to Locations</Link>
      <h1 className="text-2xl font-bold mt-4 mb-2">{location.name}</h1>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div className="bg-white p-4 rounded shadow">
          <div className="text-sm text-gray-500">Name</div>
          <div className="text-lg font-semibold">{location.name}</div>
        </div>
        <div className="bg-white p-4 rounded shadow">
          <div className="text-sm text-gray-500">Timezone</div>
          <div className="text-lg font-semibold">{location.timezone || 'N/A'}</div>
        </div>
      </div>

      <div className="bg-white p-6 rounded shadow mb-6">
        <h2 className="text-lg font-semibold mb-4">Description</h2>
        <p className="text-gray-700">{location.description || 'No description'}</p>
      </div>

      <div className="bg-white p-6 rounded shadow">
        <h2 className="text-lg font-semibold mb-4">Address</h2>
        <p className="text-gray-700">{location.address || 'No address'}</p>
      </div>
    </div>
  );
}
