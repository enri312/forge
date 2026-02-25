import React from 'react';
import { Zap } from 'lucide-react';
import { GeneralIcon, GraphIcon, CacheIcon, SettingsIcon } from '../icons';

function NavItem({ icon, label, active = false, onClick }: { icon: React.ReactNode, label: string, active?: boolean, onClick?: () => void }) {
  return (
    <button
      onClick={onClick}
      className={`relative w-full flex items-center gap-3 px-4 py-3 transition-all duration-300 ${
        active
          ? 'bg-gradient-to-r from-[#8B2500] to-[#3A0D00] text-[#FF4500] cyber-cut shadow-[0_0_15px_rgba(255,69,0,0.3)]'
          : 'bg-transparent text-gray-400 hover:bg-[#1a1a1a] hover:text-gray-200 rounded-md'
      }`}
    >
      {active && (
        <div className="absolute left-0 top-0 bottom-0 w-1.5 bg-gradient-to-b from-[#FFCC00] to-[#FF3300] shadow-[0_0_10px_#FF3300]"></div>
      )}
      <div className={`w-5 h-5 ${active ? 'drop-shadow-[0_0_8px_rgba(255,69,0,0.8)]' : ''}`}>
        {icon}
      </div>
      <span className="font-medium text-sm tracking-wide">{label}</span>
    </button>
  );
}

export function Sidebar({ activeTab, setActiveTab }: { activeTab: string, setActiveTab: (tab: string) => void }) {
  return (
    <aside className="w-64 bg-[#0f0f0f] border-r border-[#2a2a2a] flex flex-col relative z-20 shadow-[4px_0_24px_rgba(0,0,0,0.5)]">
      {/* SVG Gradients Definition */}
      <svg width="0" height="0" className="absolute">
        <defs>
          <linearGradient id="neon-orange" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#FFCC00" />
            <stop offset="50%" stopColor="#FF3300" />
            <stop offset="100%" stopColor="#FF00FF" />
          </linearGradient>
        </defs>
      </svg>

      <div className="h-1 w-full caution-stripes-lava"></div>
      <div className="p-6 flex items-center gap-3 border-b border-[#2a2a2a]">
        <div className="relative flex items-center justify-center w-10 h-10 bg-[#FF3300]/10 cyber-cut border border-[#FF3300]/50 animate-pulse-glow-lava">
          <Zap className="w-6 h-6 text-[#FF3300]" />
        </div>
        <div>
          <h1 className="text-xl font-bold tracking-wider text-white drop-shadow-[0_0_8px_rgba(255,51,0,0.5)]">FORGE</h1>
          <p className="text-[10px] text-[#FF3300] uppercase tracking-widest font-mono">Build System</p>
        </div>
      </div>
      
      <nav className="flex-1 py-6 px-4 space-y-2">
        <NavItem icon={<GeneralIcon active={activeTab === 'general'} />} label="General" active={activeTab === 'general'} onClick={() => setActiveTab('general')} />
        <NavItem icon={<GraphIcon active={activeTab === 'graph'} />} label="Graph (DAG)" active={activeTab === 'graph'} onClick={() => setActiveTab('graph')} />
        <NavItem icon={<CacheIcon active={activeTab === 'cache'} />} label="Cache Stats" active={activeTab === 'cache'} onClick={() => setActiveTab('cache')} />
        <NavItem icon={<SettingsIcon active={activeTab === 'settings'} />} label="Settings" active={activeTab === 'settings'} onClick={() => setActiveTab('settings')} />
      </nav>
      
      <div className="p-4 border-t border-[#2a2a2a] bg-[#050505]">
        <div className="flex items-center gap-3 px-3 py-2 cyber-cut bg-[#0f0f0f] border border-[#333]">
          <div className="w-2 h-2 rounded-none bg-[#39FF14] shadow-[0_0_8px_#39FF14] animate-pulse"></div>
          <span className="text-xs font-mono text-gray-400">Daemon: v2.4.1</span>
        </div>
      </div>
    </aside>
  );
}
