import { useState, useEffect } from 'react';
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
        <a href={`https://explorer.solana.com/tx/${event.signature}`} target="_blank" rel="noreferrer" className="hover:text-indigo-600 hover:underline">
          {event.signature.slice(0, 12)}...
        </a>
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

function InteractiveDemo({ latestEvent }: { latestEvent?: LifecycleEvent }) {
  const [isSwapping, setIsSwapping] = useState(false);
  const [successSig, setSuccessSig] = useState<string | null>(null);
  const [amount, setAmount] = useState('0.1');
  const [recipient, setRecipient] = useState('');

  const handleSwap = async () => {
    setIsSwapping(true);
    setSuccessSig(null);
    try {
      const response = await fetch('http://localhost:3000/api/relay', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action: 'transfer', amount_in: parseFloat(amount) || 0, recipient: recipient.trim() || undefined }),
      });
      const data = await response.json();
      if (data.success && data.signature) {
        setSuccessSig(data.signature);
      } else {
        alert("Failed to relay transaction: " + data.error);
      }
    } catch (e) {
      console.error(e);
      alert("Error reaching relayer API");
    } finally {
      setIsSwapping(false);
    }
  };

  return (
    <div className="glass-panel p-6 mb-8 border-indigo-100 bg-gradient-to-br from-indigo-50/50 to-white">
      <div className="flex flex-col md:flex-row items-center justify-between gap-6">
        <div className="flex-1">
          <h2 className="text-lg font-bold text-slate-800 flex items-center gap-2 mb-2">
            <span className="bg-indigo-100 text-indigo-700 px-2 py-0.5 rounded text-xs">Live Submission</span>
            Vortex Transaction Sender
          </h2>
          <p className="text-sm text-slate-500 max-w-md">
            Execute a real transfer using the Vortex stack. The backend autonomously evaluates network conditions to set the optimal Jito tip and guarantees landing.
          </p>
        </div>
        
        <div className="bg-white p-5 rounded-xl border border-slate-200 shadow-sm w-full md:w-96 relative overflow-hidden">
          {isSwapping && (
             <div className="absolute inset-0 bg-white/80 backdrop-blur-sm z-10 flex flex-col items-center justify-center">
               <div className="w-8 h-8 rounded-full border-2 border-indigo-500 border-t-transparent animate-spin mb-3" />
               <span className="text-xs font-bold text-indigo-700 uppercase tracking-wider animate-pulse">Relaying to Vortex...</span>
             </div>
          )}
          
          <div className="space-y-4 mb-6">
            <div>
              <label className="block text-xs font-medium text-slate-500 mb-1">Amount (SOL)</label>
              <input 
                type="number" 
                value={amount} 
                onChange={(e) => setAmount(e.target.value)} 
                className="w-full bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 text-sm font-bold text-slate-800 focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                placeholder="0.1" 
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-slate-500 mb-1">Recipient Address</label>
              <input 
                type="text" 
                value={recipient} 
                onChange={(e) => setRecipient(e.target.value)} 
                className="w-full bg-slate-50 border border-slate-200 rounded-lg px-3 py-2 text-sm text-slate-800 focus:outline-none focus:ring-2 focus:ring-indigo-500 font-mono" 
                placeholder="Leave blank to send to self" 
              />
            </div>
          </div>
          
          <button 
            onClick={handleSwap}
            disabled={isSwapping}
            className="w-full py-3 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-semibold shadow-md shadow-indigo-200 transition-all text-sm flex items-center justify-center gap-2"
          >
            <Zap className="w-4 h-4" /> Send Transaction
          </button>
          
          {successSig && !isSwapping && (
            <div className="mt-4 p-3 bg-emerald-50 border border-emerald-100 rounded-lg text-xs text-emerald-700">
              <div className="flex items-center gap-1.5 font-bold mb-1">
                <CheckCircle2 className="w-4 h-4" /> Transaction Landed!
              </div>
              <a href={`https://explorer.solana.com/tx/${successSig}`} target="_blank" rel="noreferrer" className="underline truncate block">
                {successSig.slice(0, 24)}...
              </a>
            </div>
          )}
        </div>
      </div>
    </div>
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
          <div className="flex items-center gap-3 mb-2">
            <h1 className="text-3xl font-bold tracking-tight bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
              Vortex Transaction Stack
            </h1>
            <div className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-50 border border-emerald-200 shadow-sm">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
              </span>
              <span className="text-[10px] font-bold text-emerald-600 uppercase tracking-wider">Engine Live</span>
            </div>
          </div>
          <p className="text-slate-500 flex items-center gap-2 text-sm">
            <Activity className="w-4 h-4 text-blue-500" />
            Live network conditions & AI autonomous routing
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

      <InteractiveDemo latestEvent={latest} />

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
