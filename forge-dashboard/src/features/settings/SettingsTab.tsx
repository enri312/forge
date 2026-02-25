import React from 'react';
import { DashboardCard } from '../../shared/components/DashboardCard';
import { Save, AlertTriangle, ShieldAlert, Database, Cpu, Terminal } from 'lucide-react';

export function SettingsTab() {
  return (
    <div className="h-full flex flex-col pb-8">
      <div className="flex justify-between items-center mb-6">
        <div>
          <h2 className="text-2xl font-bold text-white tracking-widest uppercase">System Settings</h2>
          <p className="text-xs text-gray-400 font-mono mt-1">Configure FORGE daemon and workspace preferences</p>
        </div>
        <button className="flex items-center gap-2 px-6 py-2 bg-[#0f0f0f] border border-[#FFCC00] text-[#FFCC00] font-mono text-sm hover:bg-[#FFCC00] hover:text-[#0f0f0f] transition-colors cyber-cut shadow-[0_0_10px_rgba(255,204,0,0.2)]">
          <Save size={16} />
          <span>SAVE CONFIG</span>
        </button>
      </div>

      <div className="grid grid-cols-12 gap-6">
        {/* Core Execution Settings */}
        <DashboardCard title="CORE EXECUTION" className="col-span-12 lg:col-span-6 border-t-2 border-t-[#39FF14]">
          <div className="space-y-6 mt-4">
            <div className="flex items-start gap-4">
              <div className="p-3 bg-[#1a1a1a] cyber-cut border border-[#333]">
                <Cpu className="w-5 h-5 text-[#39FF14]" />
              </div>
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-gray-200">Parallel Workers</span>
                  <div className="flex items-center bg-[#050505] border border-[#333] px-3 py-1 cyber-cut">
                    <span className="text-[#39FF14] font-mono font-bold">16</span>
                    <span className="text-[#444] font-mono ml-2">cores</span>
                  </div>
                </div>
                <p className="text-xs text-gray-500 mt-1">Number of concurrent threads for task execution. Defaults to physical CPU cores.</p>
              </div>
            </div>

            <div className="w-full h-px bg-[#222]"></div>

            <div className="flex items-start gap-4">
              <div className="p-3 bg-[#1a1a1a] cyber-cut border border-[#333]">
                <ShieldAlert className="w-5 h-5 text-[#FFCC00]" />
              </div>
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-gray-200">Daemon Mode</span>
                  <div className="w-12 h-6 bg-[#050505] border border-[#333] relative cursor-pointer cyber-cut">
                    <div className="absolute right-1 top-1 w-4 h-4 bg-[#FFCC00] shadow-[0_0_8px_#FFCC00] cyber-cut"></div>
                  </div>
                </div>
                <p className="text-xs text-gray-500 mt-1">Keep a background process alive to maintain Hot-JVMs and avoid startup overhead.</p>
              </div>
            </div>
          </div>
        </DashboardCard>

        {/* Console & Output */}
        <DashboardCard title="TERMINAL OUTPUT" className="col-span-12 lg:col-span-6 border-t-2 border-t-[#00FFFF]">
          <div className="space-y-6 mt-4">
            <div className="flex items-start gap-4">
              <div className="p-3 bg-[#1a1a1a] cyber-cut border border-[#333]">
                <Terminal className="w-5 h-5 text-[#00FFFF]" />
              </div>
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-gray-200">Log Level</span>
                  <select className="bg-[#050505] border border-[#333] text-[#00FFFF] font-mono px-3 py-1 cyber-cut outline-none focus:border-[#00FFFF]">
                    <option value="info">INFO</option>
                    <option value="debug">DEBUG</option>
                    <option value="warn">WARN</option>
                    <option value="error">ERROR</option>
                  </select>
                </div>
                <p className="text-xs text-gray-500 mt-1">Verbosity of the CLI standard output.</p>
              </div>
            </div>

            <div className="w-full h-px bg-[#222]"></div>

            <div className="flex items-start gap-4">
              <div className="p-3 bg-[#1a1a1a] cyber-cut border border-[#333] opacity-50"></div>
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-gray-200">Rich Output</span>
                  <div className="w-12 h-6 bg-[#050505] border border-[#333] relative cursor-pointer cyber-cut">
                    <div className="absolute right-1 top-1 w-4 h-4 bg-[#00FFFF] shadow-[0_0_8px_#00FFFF] cyber-cut"></div>
                  </div>
                </div>
                <p className="text-xs text-gray-500 mt-1">Use ANSI colors and interactive progress bars in TTY terminals.</p>
              </div>
            </div>
          </div>
        </DashboardCard>

        {/* Global Cache Settings */}
        <DashboardCard title="DISTRIBUTED CACHE" className="col-span-12 lg:col-span-8 border-t-2 border-t-[#FF00FF]">
          <div className="space-y-6 mt-4">
            <div className="flex items-start gap-4">
              <div className="p-3 bg-[#1a1a1a] cyber-cut border border-[#333]">
                <Database className="w-5 h-5 text-[#FF00FF]" />
              </div>
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <span className="font-bold text-gray-200">S3 Remote Cache</span>
                  <div className="w-12 h-6 bg-[#050505] border border-[#333] relative cursor-pointer cyber-cut">
                    <div className="absolute right-1 top-1 w-4 h-4 bg-[#FF00FF] shadow-[0_0_8px_#FF00FF] cyber-cut"></div>
                  </div>
                </div>
                <p className="text-xs text-gray-500 mt-1">Share build artifacts across the team via AWS S3 or MinIO.</p>

                <div className="mt-4 p-4 bg-[#050505] border border-[#222] cyber-cut relative overflow-hidden">
                  <div className="absolute top-0 left-0 w-1 h-full bg-[#FF00FF]"></div>
                  <div className="grid grid-cols-3 gap-4 items-center">
                    <span className="text-xs text-gray-400 font-mono">ENDPOINT URL</span>
                    <input type="text" readOnly value="s3://forge-cache-prod/team-alpha" className="col-span-2 bg-[#111] border border-[#333] text-[#FF00FF] font-mono px-3 py-1 outline-none" />

                    <span className="text-xs text-gray-400 font-mono">ACCESS KEY</span>
                    <input type="password" readOnly value="****************" className="col-span-2 bg-[#111] border border-[#333] text-gray-400 font-mono px-3 py-1 outline-none tracking-widest" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </DashboardCard>

        {/* Danger Zone */}
        <div className="col-span-12 lg:col-span-4 bg-[#0f0f0f] border border-[#FF3300] shadow-[0_0_20px_rgba(255,51,0,0.1)] cyber-cut p-6 relative overflow-hidden">
          <div className="absolute top-0 left-0 w-full h-1 caution-stripes opacity-80"></div>
          <div className="absolute bottom-0 left-0 w-full h-1 caution-stripes opacity-80"></div>

          <div className="flex items-center gap-2 mb-4">
            <AlertTriangle className="w-5 h-5 text-[#FF3300] animate-pulse" />
            <h3 className="text-sm font-bold text-[#FF3300] uppercase tracking-widest drop-shadow-[0_0_5px_rgba(255,51,0,0.8)]">DANGER ZONE</h3>
          </div>

          <div className="space-y-4">
            <p className="text-xs text-gray-400 leading-relaxed">
              These actions are immediate and cannot be undone. Purging the local cache will force a full recompilation of all projects on the next build.
            </p>

            <button className="w-full flex items-center justify-center gap-2 py-3 bg-[#FF3300]/10 border border-[#FF3300] text-[#FF3300] font-mono text-sm hover:bg-[#FF3300]/20 transition-all cyber-cut shadow-[0_0_15px_rgba(255,51,0,0.2)]">
              <span>PURGE LOCAL CACHE</span>
            </button>

            <button className="w-full flex items-center justify-center gap-2 py-3 bg-[#111] border border-[#555] text-gray-400 font-mono text-xs hover:border-[#FF3300] hover:text-[#FF3300] transition-colors cyber-cut">
              <span>RESTART DAEMON</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
