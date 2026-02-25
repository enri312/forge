import React from 'react';
import { DashboardCard } from '../../shared/components/DashboardCard';
import { StatBar } from '../../shared/components/StatBar';
import { HardDrive, Cloud, Zap, RefreshCw, FileCode, Box, Server } from 'lucide-react';
import { useTelemetry } from '../../shared/context/TelemetryContext';

export function CacheTab() {
  const { cacheStats } = useTelemetry();

  const totalQueries = cacheStats.localHits + cacheStats.s3Hits + cacheStats.misses;
  const localPct = totalQueries > 0 ? Math.round((cacheStats.localHits / totalQueries) * 100) : 0;
  const s3Pct = totalQueries > 0 ? Math.round((cacheStats.s3Hits / totalQueries) * 100) : 0;
  const missPct = totalQueries > 0 ? Math.round((cacheStats.misses / totalQueries) * 100) : 0;
  return (
    <div className="h-full flex flex-col pb-8">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h2 className="text-2xl font-bold text-white tracking-widest uppercase">Cache Telemetry</h2>
          <p className="text-xs text-gray-400 font-mono mt-1">Real-time object storage & hit rate analytics</p>
        </div>
        <div className="flex items-center gap-3">
          <span className="flex items-center gap-2 text-xs font-mono text-[#39FF14] bg-[#0f0f0f] border border-[#39FF14] px-3 py-1 cyber-cut shadow-[0_0_8px_rgba(57,255,20,0.2)]">
            <RefreshCw size={14} className="animate-spin" />
            SYNCING S3
          </span>
        </div>
      </div>

      <div className="grid grid-cols-12 gap-6">

        {/* Core Metrics Array */}
        <DashboardCard title="PERFORMANCE MATRIX" className="col-span-12 lg:col-span-8 border-t-2 border-t-[#00FFFF]">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
            <div className="flex flex-col relative p-5 cyber-cut bg-[#050505] border border-[#222] overflow-hidden group">
              <div className="absolute top-0 left-0 w-1 h-full bg-[#00FFFF] shadow-[0_0_10px_#00FFFF]"></div>
              <div className="flex justify-between items-start mb-2">
                <Zap className="text-[#00FFFF] w-5 h-5" />
                <span className="text-[#00FFFF] text-xs font-mono opacity-50">T-SAVED</span>
              </div>
              <span className="text-4xl font-bold text-white font-mono mt-2 drop-shadow-[0_0_8px_rgba(255,255,255,0.3)]">{cacheStats.tSavedSecs.toFixed(2)}<span className="text-lg text-gray-500">s</span></span>
              <span className="text-[10px] text-gray-400 uppercase tracking-widest mt-1">Avg Time Saved / Build</span>
            </div>

            <div className="flex flex-col relative p-5 cyber-cut bg-[#050505] border border-[#222] overflow-hidden group">
              <div className="absolute top-0 left-0 w-1 h-full bg-[#39FF14] shadow-[0_0_10px_#39FF14]"></div>
              <div className="flex justify-between items-start mb-2">
                <Server className="text-[#39FF14] w-5 h-5" />
                <span className="text-[#39FF14] text-xs font-mono opacity-50">OBJ-HIT</span>
              </div>
              <span className="text-4xl font-bold text-white font-mono mt-2 drop-shadow-[0_0_8px_rgba(255,255,255,0.3)]">{cacheStats.hitRate}<span className="text-lg text-gray-500">%</span></span>
              <span className="text-[10px] text-gray-400 uppercase tracking-widest mt-1">Overall Hit Rate</span>
            </div>

            <div className="flex flex-col relative p-5 cyber-cut bg-[#050505] border border-[#222] overflow-hidden group">
              <div className="absolute top-0 left-0 w-1 h-full bg-[#FF00FF] shadow-[0_0_10px_#FF00FF]"></div>
              <div className="flex justify-between items-start mb-2">
                <Box className="text-[#FF00FF] w-5 h-5" />
                <span className="text-[#FF00FF] text-xs font-mono opacity-50">I/O-BW</span>
              </div>
              <span className="text-4xl font-bold text-white font-mono mt-2 drop-shadow-[0_0_8px_rgba(255,255,255,0.3)]">{cacheStats.bandwidthSavedGb.toFixed(1)}<span className="text-lg text-gray-500">GB</span></span>
              <span className="text-[10px] text-gray-400 uppercase tracking-widest mt-1">Bandwidth Saved</span>
            </div>
          </div>
        </DashboardCard>

        {/* Global Distribution */}
        <DashboardCard title="STORAGE DISTRIBUTION" className="col-span-12 lg:col-span-4 border-t-2 border-t-[#FFCC00]">
          <div className="space-y-6 mt-6">
            <StatBar label={`Local Disk (Hits: ${cacheStats.localHits})`} value={`${localPct}%`} percent={localPct} color="bg-[#00FFFF]" glow="rgba(0,255,255,0.5)" />
            <StatBar label={`S3 Remote (Hits: ${cacheStats.s3Hits})`} value={`${s3Pct}%`} percent={s3Pct} color="bg-[#FF00FF]" glow="rgba(255,0,255,0.5)" />
            <StatBar label={`Misses (Hits: ${cacheStats.misses})`} value={`${missPct}%`} percent={missPct} color="bg-[#FF3300]" glow="rgba(255,51,0,0.5)" />
          </div>
        </DashboardCard>

        {/* Artifact Stream Table */}
        <DashboardCard title="RECENT ARTIFACTS" className="col-span-12 border-t-2 border-t-[#FF3300]">
          <div className="mt-4 overflow-x-auto">
            <table className="w-full text-left font-mono text-sm">
              <thead>
                <tr className="text-[#FF3300] border-b border-[#333] tracking-widest text-xs">
                  <th className="pb-3 px-4 font-normal">HASH (SHA-256)</th>
                  <th className="pb-3 px-4 font-normal">MODULE</th>
                  <th className="pb-3 px-4 font-normal">SOURCE COMPONENT</th>
                  <th className="pb-3 px-4 font-normal text-right">SIZE</th>
                  <th className="pb-3 px-4 font-normal text-right">NODE</th>
                </tr>
              </thead>
              <tbody className="text-gray-300">
                <tr className="border-b border-[#1a1a1a] hover:bg-[#111] transition-colors">
                  <td className="py-3 px-4 text-gray-500">a1b2c3d4e5f6...</td>
                  <td className="py-3 px-4 text-[#39FF14]">core</td>
                  <td className="py-3 px-4 flex items-center gap-2"><FileCode size={14} className="text-gray-500" /> src/main/java/Engine.java</td>
                  <td className="py-3 px-4 text-right">4.2 MB</td>
                  <td className="py-3 px-4 text-right flex justify-end gap-2 text-[#00FFFF]"><HardDrive size={16} /> LOCAL</td>
                </tr>
                <tr className="border-b border-[#1a1a1a] hover:bg-[#111] transition-colors">
                  <td className="py-3 px-4 text-gray-500">f8e7d6c5b4a3...</td>
                  <td className="py-3 px-4 text-[#FF00FF]">auth</td>
                  <td className="py-3 px-4 flex items-center gap-2"><FileCode size={14} className="text-gray-500" /> src/main/kotlin/JwtUtils.kt</td>
                  <td className="py-3 px-4 text-right">1.8 MB</td>
                  <td className="py-3 px-4 text-right flex justify-end gap-2 text-[#FF00FF]"><Cloud size={16} /> S3</td>
                </tr>
                <tr className="hover:bg-[#111] transition-colors">
                  <td className="py-3 px-4 text-gray-500">9c8b7a6f5e4d...</td>
                  <td className="py-3 px-4 text-[#FF3300]">api</td>
                  <td className="py-3 px-4 flex items-center gap-2"><FileCode size={14} className="text-gray-500" /> src/main/java/Router.java</td>
                  <td className="py-3 px-4 text-right">8.1 MB</td>
                  <td className="py-3 px-4 text-right flex justify-end gap-2 text-[#FF3300]">MISS</td>
                </tr>
              </tbody>
            </table>
          </div>
        </DashboardCard>

      </div>
    </div>
  );
}
