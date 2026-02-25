import React from 'react';
import { Terminal } from 'lucide-react';
import { DashboardCard } from '../../shared/components/DashboardCard';
import { StatBar } from '../../shared/components/StatBar';
import { DagNode } from '../../shared/components/DagNode';
import { useTelemetry } from '../../shared/context/TelemetryContext';

export function GeneralTab() {
  const { logs, cacheStats, tasks } = useTelemetry();

  return (
    <div className="grid grid-cols-12 gap-6">
      {/* Card 1: Build Stats */}
      <DashboardCard title="Compilation Stats" className="col-span-12 lg:col-span-4 border-t-2 border-t-[#FFCC00]">
        <div className="space-y-5 mt-2">
          <StatBar label="Dependency Resolution" value="140ms" percent={16} color="bg-[#FFCC00]" glow="rgba(255,204,0,0.5)" />
          <StatBar label="Kotlin Compilation" value="450ms" percent={53} color="bg-[#FF3300]" glow="rgba(255,51,0,0.5)" />
          <StatBar label="JUnit Tasks" value="250ms" percent={31} color="bg-[#00FFFF]" glow="rgba(0,255,255,0.5)" />
        </div>
      </DashboardCard>

      {/* Card 2: Cache Hit Rate */}
      <DashboardCard title="Cache Hit Rate" className="col-span-12 lg:col-span-4 flex flex-col items-center justify-center border-t-2 border-t-[#00FFFF]">
        <div className="relative w-40 h-40 flex items-center justify-center mt-4">
          <svg className="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
            <circle cx="50" cy="50" r="40" fill="transparent" stroke="#1a1a1a" strokeWidth="8" />
            <circle cx="50" cy="50" r="40" fill="transparent" stroke="#00FFFF" strokeWidth="8"
              strokeDasharray="251.2" strokeDashoffset={251.2 - (251.2 * (cacheStats.hitRate || 0)) / 100}
              className="drop-shadow-[0_0_12px_rgba(0,255,255,0.8)] transition-all duration-1000 ease-out" />
          </svg>
          <div className="absolute flex flex-col items-center">
            <span className="text-4xl font-bold text-white font-mono drop-shadow-[0_0_8px_rgba(0,255,255,0.5)]">{cacheStats.hitRate || 0}%</span>
            <span className="text-[10px] text-[#00FFFF] uppercase tracking-widest mt-1">Hit Rate</span>
          </div>
        </div>
        <div className="flex gap-4 mt-6 text-xs font-mono text-gray-400">
          <div className="flex items-center gap-1.5">
            <span className="w-2 h-2 bg-[#00FFFF] shadow-[0_0_5px_#00FFFF]"></span> Local
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-2 h-2 bg-[#FFCC00] shadow-[0_0_5px_#FFCC00]"></span> S3 Remote
          </div>
        </div>
      </DashboardCard>

      {/* Card 3: DAG - Dependency Graph */}
      <DashboardCard title="Dependency Graph (DAG)" className="col-span-12 lg:col-span-4 border-t-2 border-t-[#FF00FF]">
        <div className="relative h-48 flex items-center justify-center mt-2">
          <div className="flex items-center justify-center w-full">
            <DagNode label="core" status={tasks['core']?.state || 'pending'} />
            <div className="w-6 h-0.5 bg-gradient-to-r from-[#39FF14] to-[#FF00FF] relative shadow-[0_0_8px_rgba(255,0,255,0.5)] mx-1 flex-shrink-0">
              <div className="absolute right-0 top-1/2 -translate-y-1/2 w-2 h-2 border-t-2 border-r-2 border-[#FF00FF] transform rotate-45"></div>
            </div>
            <DagNode label="auth" status={tasks['auth']?.state || 'pending'} />
            <div className="w-6 h-0.5 bg-gradient-to-r from-[#FF00FF] to-[#FF3300] relative shadow-[0_0_8px_rgba(255,51,0,0.5)] mx-1 flex-shrink-0">
              <div className="absolute right-0 top-1/2 -translate-y-1/2 w-2 h-2 border-t-2 border-r-2 border-[#FF3300] transform rotate-45"></div>
            </div>
            <DagNode label="api" status={tasks['api']?.state || 'pending'} />
          </div>
        </div>
      </DashboardCard>

      {/* Card 4: Terminal / Logs */}
      <div className="col-span-12 bg-[#050505] border border-[#333] cyber-cut overflow-hidden shadow-[0_8px_32px_rgba(0,0,0,0.8)] flex flex-col h-80 relative">
        <div className="absolute top-0 left-0 w-full h-1 caution-stripes opacity-50"></div>
        <div className="h-10 bg-[#0f0f0f] border-b border-[#333] flex items-center px-4 gap-2 mt-1">
          <Terminal className="w-4 h-4 text-[#FFCC00]" />
          <span className="text-xs font-mono text-gray-400">forge-executor-tty</span>
          <div className="ml-auto flex gap-2">
            <div className="w-3 h-3 bg-[#333] cyber-cut"></div>
            <div className="w-3 h-3 bg-[#333] cyber-cut"></div>
            <div className="w-3 h-3 bg-[#FF3300] cyber-cut shadow-[0_0_5px_#FF3300]"></div>
          </div>
        </div>
        <div className="flex-1 p-4 font-mono text-sm overflow-y-auto bg-[#050505] flex flex-col-reverse">
          <div className="flex items-center mt-1">
            <span className="text-[#FF3300] mr-2 drop-shadow-[0_0_5px_rgba(255,51,0,0.8)]">❯</span>
            <span className="w-2 h-4 bg-[#FFCC00] animate-blink shadow-[0_0_5px_#FFCC00]"></span>
          </div>
          {logs.slice().reverse().map((log) => (
            <div key={log.id} className="mb-1 leading-relaxed">
              <span className="text-[#333] mr-2">{`[${log.timestamp.split('T')[1].slice(0, 8)}]`}</span>
              <span className={
                log.level === 'INFO' ? 'text-[#00FFFF] drop-shadow-[0_0_2px_rgba(0,255,255,0.8)]' :
                  log.level === 'WARN' ? 'text-[#FF00FF] drop-shadow-[0_0_2px_rgba(255,0,255,0.8)]' :
                    log.level === 'SUCCESS' ? 'text-[#39FF14] drop-shadow-[0_0_2px_rgba(57,255,20,0.8)]' :
                      log.level === 'ERROR' ? 'text-[#FF3300] drop-shadow-[0_0_2px_rgba(255,51,0,0.8)]' :
                        'text-gray-300'
              }>
                {log.text}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
