import { Activity, Clock, Zap, AlertTriangle, CheckCircle2, MoreHorizontal } from 'lucide-react';
import { useLifecycleLogs } from '../hooks/useLifecycleLogs';
import type { LifecycleEvent } from '../types/lifecycle';

function StatusIcon({ status }: { status: string }) {
  switch (status) {
    case 'Finalized':
      return <CheckCircle2 className="w-4 h-4 text-emerald-500" />;
    case 'Confirmed':
      return <CheckCircle2 className="w-4 h-4 text-blue-500" />;
    case 'Processed':
      return <Zap className="w-4 h-4 text-amber-500" />;
    case 'Failed':
      return <AlertTriangle className="w-4 h-4 text-red-500" />;
    default:
      return <MoreHorizontal className="w-4 h-4 text-slate-400 animate-pulse" />;
  }
}

function EventRow({ event }: { event: LifecycleEvent }) {
  const isFailed = event.status === 'Failed';
  
  return (
    <tr className="border-b border-slate-100 hover:bg-slate-50 transition-colors group">
      <td className="px-5 py-4 font-mono text-xs text-slate-500">
        {event.signature.slice(0, 12)}...
      </td>
      <td className="px-5 py-4 font-mono text-sm text-slate-700 font-medium">
        {event.tip_lamports.toLocaleString()}
      </td>
      <td className="px-5 py-4">
        <div className="flex items-center gap-2">
          <StatusIcon status={event.status} />
          <span className="text-sm font-medium text-slate-700">{event.status}</span>
        </div>
      </td>
      <td className="px-5 py-4 text-sm text-slate-600">
        {isFailed && event.failure ? (
          <span className="text-red-600 bg-red-50 px-2.5 py-1 rounded-md text-xs border border-red-100 font-medium">
            {event.failure.error_type}
          </span>
        ) : (
          <div className="flex flex-col gap-1 font-mono text-xs">
            {event.latency_to_processed_ms && (
              <span className="text-amber-600 bg-amber-50 px-1.5 py-0.5 rounded border border-amber-100 inline-block w-fit">P: {event.latency_to_processed_ms}ms</span>
            )}
            {event.latency_to_confirmed_ms && (
              <span className="text-blue-600 bg-blue-50 px-1.5 py-0.5 rounded border border-blue-100 inline-block w-fit">C: {event.latency_to_confirmed_ms}ms</span>
            )}
            {event.latency_to_finalized_ms && (
              <span className="text-emerald-600 bg-emerald-50 px-1.5 py-0.5 rounded border border-emerald-100 inline-block w-fit">F: {event.latency_to_finalized_ms}ms</span>
            )}
            {!event.latency_to_processed_ms && !event.latency_to_confirmed_ms && !event.latency_to_finalized_ms && (
              <span className="text-slate-400">—</span>
            )}
          </div>
        )}
      </td>
      <td className="px-5 py-4 text-sm text-slate-600 leading-relaxed max-w-md" title={event.tip_reasoning}>
        <div className="line-clamp-2">
          {event.tip_reasoning}
        </div>
      </td>
    </tr>
  );
}

export function Dashboard() {
  const { events, loading } = useLifecycleLogs();
  const latest = events[events.length - 1];
  const pendingCount = events.filter(e => e.status === 'Pending' || e.status === 'Processed').length;
  const successCount = events.filter(e => e.status === 'Confirmed' || e.status === 'Finalized').length;
  const failedCount = events.filter(e => e.status === 'Failed').length;

  return (
    <div className="max-w-6xl mx-auto space-y-8 py-8 px-4 sm:px-6 lg:px-8">
      <header className="flex flex-col md:flex-row justify-between items-start md:items-end gap-6 pb-6 border-b border-slate-200">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-slate-900 mb-2">
            Solana TX Stack
          </h1>
          <p className="text-slate-500 flex items-center gap-2 text-sm">
            <Activity className="w-4 h-4" />
            Live transaction lifecycle monitor & Agent visualizer
          </p>
        </div>
        
        <div className="flex gap-4">
          <div className="glass-panel px-6 py-3 flex flex-col items-center justify-center min-w-[100px]">
            <span className="text-2xl font-bold text-slate-800">{events.length}</span>
            <span className="text-[10px] font-semibold text-slate-500 uppercase tracking-wider mt-1">Total</span>
          </div>
          <div className="glass-panel px-6 py-3 flex flex-col items-center justify-center min-w-[100px]">
            <span className="text-2xl font-bold text-emerald-600">{successCount}</span>
            <span className="text-[10px] font-semibold text-slate-500 uppercase tracking-wider mt-1">Success</span>
          </div>
          <div className="glass-panel px-6 py-3 flex flex-col items-center justify-center min-w-[100px]">
            <span className="text-2xl font-bold text-red-600">{failedCount}</span>
            <span className="text-[10px] font-semibold text-slate-500 uppercase tracking-wider mt-1">Failed</span>
          </div>
        </div>
      </header>

      {latest && (
        <div className="glass-panel p-8 relative overflow-hidden bg-gradient-to-br from-white to-slate-50/50">
          <div className="relative z-10">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xs font-bold uppercase tracking-widest text-slate-500 flex items-center gap-2">
                <Zap className="w-4 h-4 text-blue-500" /> Latest Agent Decision
              </h2>
              {pendingCount > 0 && (
                <span className="px-3 py-1 rounded-full text-xs font-semibold bg-amber-50 text-amber-600 border border-amber-200 shadow-sm animate-pulse">
                  {pendingCount} Pending
                </span>
              )}
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
              <div className="col-span-1 border-r border-slate-200 pr-8">
                <div className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-2">Recommended Tip</div>
                <div className="text-4xl font-mono font-medium text-slate-800 mb-2">
                  {latest.tip_lamports.toLocaleString()} <span className="text-sm font-sans text-slate-500 font-normal">lamports</span>
                </div>
                <div className="text-xs font-mono text-slate-500 flex items-center gap-1.5 mt-4">
                  <Clock className="w-3.5 h-3.5 text-slate-400" /> Slot {latest.submitted_slot}
                </div>
              </div>
              <div className="col-span-2">
                <div className="text-[10px] font-semibold text-slate-400 uppercase tracking-wider mb-3">AI Reasoning</div>
                <p className="text-base text-slate-700 leading-relaxed font-serif italic">
                  "{latest.tip_reasoning}"
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      <div className="glass-panel overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="border-b border-slate-200 bg-slate-50/50">
                <th className="px-5 py-4 text-xs font-semibold text-slate-500 uppercase tracking-wider">Signature</th>
                <th className="px-5 py-4 text-xs font-semibold text-slate-500 uppercase tracking-wider">Tip Amount</th>
                <th className="px-5 py-4 text-xs font-semibold text-slate-500 uppercase tracking-wider">Status</th>
                <th className="px-5 py-4 text-xs font-semibold text-slate-500 uppercase tracking-wider">Latency</th>
                <th className="px-5 py-4 text-xs font-semibold text-slate-500 uppercase tracking-wider">Reasoning</th>
              </tr>
            </thead>
            <tbody>
              {loading ? (
                <tr>
                  <td colSpan={5} className="px-5 py-16 text-center">
                    <div className="flex flex-col items-center justify-center gap-3">
                      <div className="w-6 h-6 rounded-full border-2 border-blue-500 border-t-transparent animate-spin" />
                      <span className="text-sm font-medium text-slate-500">Connecting to stream...</span>
                    </div>
                  </td>
                </tr>
              ) : events.length === 0 ? (
                <tr>
                  <td colSpan={5} className="px-5 py-20 text-center text-slate-400">
                    <Activity className="w-8 h-8 mx-auto mb-4 text-slate-300" />
                    <p className="text-sm font-medium">No transactions tracked yet.</p>
                    <p className="text-xs mt-1">Waiting for backend submissions.</p>
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
