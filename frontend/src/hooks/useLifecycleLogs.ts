import { useEffect, useState } from 'react';
import type { LifecycleEvent } from '../types/lifecycle';

export function useLifecycleLogs() {
  const [events, setEvents] = useState<LifecycleEvent[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchLogs = async () => {
      try {
        const res = await fetch('/logs/lifecycle.json');
        const data = await res.json();
        setEvents(data);
      } catch (err) {
        console.error('Failed to fetch logs:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchLogs();
    const interval = setInterval(fetchLogs, 2000);
    return () => clearInterval(interval);
  }, []);

  return { events, loading };
}
