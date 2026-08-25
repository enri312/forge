import React from 'react';
import { Activity, HardDrive, Server, Zap } from 'lucide-react';
import { DashboardCard } from '../../shared/components/DashboardCard';
import { StatBar } from '../../shared/components/StatBar';
import { useTelemetry } from '../../shared/context/TelemetryContext';

export function CacheTab() {
  const { cacheStats, connected, tasks } = useTelemetry();
  const totalQueries = cacheStats.localHits + cacheStats.remoteHits + cacheStats.misses;
  const localPct = totalQueries > 0 ? Math.round((cacheStats.localHits / totalQueries) * 100) : 0;
  const remotePct = totalQueries > 0 ? Math.round((cacheStats.remoteHits / totalQueries) * 100) : 0;
  const missPct = totalQueries > 0 ? Math.round((cacheStats.misses / totalQueries) * 100) : 0;
  const cacheHits = cacheStats.localHits + cacheStats.remoteHits;
  const taskList = Object.values(tasks).slice().reverse();

  return (
    <div className="h-full flex flex-col pb-8">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h2 className="text-2xl font-bold text-white tracking-widest uppercase">Cache Telemetry</h2>
          <p className="text-xs text-gray-400 font-mono mt-1">Metrics observed from the current SSE session</p>
        </div>
        <span className={`flex items-center gap-2 text-xs font-mono bg-[#0f0f0f] border px-3 py-1 cyber-cut ${connected ? 'text-[#39FF14] border-[#39FF14]' : 'text-[#FF3300] border-[#FF3300]'}`}>
          <Activity size={14} />
          {connected ? 'LIVE' : 'OFFLINE'}
        </span>
      </div>

      <div className="grid grid-cols-12 gap-6">
        <DashboardCard title="OBSERVED METRICS" className="col-span-12 lg:col-span-8 border-t-2 border-t-[#00FFFF]">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
            <Metric icon={<Activity className="text-[#00FFFF] w-5 h-5" />} label="BUILDS" value={String(totalQueries)} />
            <Metric icon={<Server className="text-[#39FF14] w-5 h-5" />} label="HIT RATE" value={`${cacheStats.hitRate}%`} />
            <Metric icon={<Zap className="text-[#FF00FF] w-5 h-5" />} label="CACHE HITS" value={String(cacheHits)} />
          </div>
        </DashboardCard>

        <DashboardCard title="CACHE SOURCES" className="col-span-12 lg:col-span-4 border-t-2 border-t-[#FFCC00]">
          <div className="space-y-6 mt-6">
            <StatBar label={`Local (${cacheStats.localHits})`} value={`${localPct}%`} percent={localPct} color="bg-[#00FFFF]" glow="rgba(0,255,255,0.5)" />
            <StatBar label={`Remote HTTP (${cacheStats.remoteHits})`} value={`${remotePct}%`} percent={remotePct} color="bg-[#FF00FF]" glow="rgba(255,0,255,0.5)" />
            <StatBar label={`Compiled (${cacheStats.misses})`} value={`${missPct}%`} percent={missPct} color="bg-[#FF3300]" glow="rgba(255,51,0,0.5)" />
          </div>
        </DashboardCard>

        <DashboardCard title="RECENT TASK EVENTS" className="col-span-12 border-t-2 border-t-[#FF3300]">
          <div className="mt-4 overflow-x-auto">
            {taskList.length === 0 ? (
              <p className="text-center text-gray-500 font-mono text-sm py-10">No task events received yet.</p>
            ) : (
              <table className="w-full text-left font-mono text-sm">
                <thead>
                  <tr className="text-[#FF3300] border-b border-[#333] tracking-widest text-xs">
                    <th className="pb-3 px-4 font-normal">TASK</th>
                    <th className="pb-3 px-4 font-normal">STATE</th>
                    <th className="pb-3 px-4 font-normal">CACHE SOURCE</th>
                    <th className="pb-3 px-4 font-normal text-right">DURATION</th>
                  </tr>
                </thead>
                <tbody className="text-gray-300">
                  {taskList.map(task => (
                    <tr key={task.name} className="border-b border-[#1a1a1a] hover:bg-[#111] transition-colors">
                      <td className="py-3 px-4 flex items-center gap-2"><HardDrive size={14} className="text-gray-500" /> {task.name}</td>
                      <td className="py-3 px-4 text-[#39FF14] uppercase">{task.state}</td>
                      <td className="py-3 px-4">{task.cache_source ?? '—'}</td>
                      <td className="py-3 px-4 text-right">{task.time_ms == null ? '—' : `${task.time_ms} ms`}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        </DashboardCard>
      </div>
    </div>
  );
}

function Metric({ icon, label, value }: { icon: React.ReactNode; label: string; value: string }) {
  return (
    <div className="flex flex-col relative p-5 cyber-cut bg-[#050505] border border-[#222] overflow-hidden">
      <div className="flex justify-between items-start mb-2">
        {icon}
        <span className="text-gray-500 text-xs font-mono">{label}</span>
      </div>
      <span className="text-4xl font-bold text-white font-mono mt-2">{value}</span>
    </div>
  );
}
