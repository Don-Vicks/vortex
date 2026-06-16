import { useLifecycleLogs } from './hooks/useLifecycleLogs';
import type { LifecycleEvent } from './types/lifecycle';

function StatusBadge({ status }: { status: string }) {
  const colors: Record<string, string> = {
    Finalized: 'text-green-400',
    Confirmed: 'text-cyan-400',
    Processed: 'text-yellow-400',
    Failed: 'text-red-400',
    Pending: 'text-gray-400',
  };
  return <span className={colors[status] ?? 'text-gray-400'}>{status}</span>;
}

function EventRow({ event }: { event: LifecycleEvent }) {
  const isFailed = event.status === 'Failed';
  
  return (
    <tr className="border-b border-gray-800 hover:bg-gray-900 transition-colors">
      <td className="px-4 py-3 font-mono text-xs text-gray-400">
        {event.signature.slice(0, 12)}...
      </td>
      <td className="px-4 py-3 font-mono text-xs text-green-300">
        {event.tip_lamports.toLocaleString()}
      </td>
      <td className="px-4 py-3 text-xs">
        <StatusBadge status={event.status} />
      </td>
      <td className="px-4 py-3 text-xs text-gray-400">
        {isFailed && event.failure ? (
          <span className="text-red-400">{event.failure.error_type}</span>
        ) : event.latency_to_confirmed_ms ? (
          `${event.latency_to_confirmed_ms}ms`
        ) : (
          '—'
        )}
      </td>
      <td className="px-4 py-3 text-xs text-gray-500 max-w-xs truncate" title={event.tip_reasoning}>
        {event.tip_reasoning}
      </td>
    </tr>
  );
}

export default function App() {
  const { events, loading } = useLifecycleLogs();
  const latest = events[events.length - 1];

  return (
    <div className="min-h-screen bg-black text-white font-mono p-6">
      <div className="max-w-6xl mx-auto">

        <h1 className="text-2xl font-bold text-green-400 mb-1">
          Solana TX Stack
        </h1>
        <p className="text-gray-500 text-sm mb-8">
          Live transaction lifecycle monitor
        </p>

        {latest && (
          <div className="mb-8 p-4 border border-cyan-900 rounded bg-gray-950">
            <p className="text-cyan-400 text-xs uppercase mb-2">
              Latest Agent Decision
            </p>
            <p className="text-white text-sm mb-2">
              Tip: {latest.tip_lamports.toLocaleString()} lamports
            </p>
            <p className="text-gray-400 text-xs leading-relaxed">
              {latest.tip_reasoning}
            </p>
          </div>
        )}

        <div className="border border-gray-800 rounded overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-gray-900 text-gray-400 text-xs uppercase">
              <tr>
                <th className="px-4 py-3 text-left">Signature</th>
                <th className="px-4 py-3 text-left">Tip (lamports)</th>
                <th className="px-4 py-3 text-left">Status</th>
                <th className="px-4 py-3 text-left">Time to Confirmed</th>
                <th className="px-4 py-3 text-left">Agent Reasoning</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr>
                  <td colSpan={5} className="px-4 py-8 text-center text-gray-500">
                    Loading...
                  </td>
                </tr>
              ) : events.length === 0 ? (
                <tr>
                  <td colSpan={5} className="px-4 py-8 text-center text-gray-500">
                    No events yet. Run the backend to start logging.
                  </td>
                </tr>
              ) : (
                [...events].reverse().map((event) => (
                  <EventRow key={event.id} event={event} />
                ))
              )}
            </tbody>
          </table>
        </div>

      </div>
    </div>
  );
}
