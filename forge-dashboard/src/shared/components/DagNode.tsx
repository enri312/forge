import React from 'react';
import { Check } from 'lucide-react';

export function DagNode({ label, status }: { label: string, status: 'done' | 'active' | 'magenta' | 'pending' }) {
  const isDone = status === 'done';
  const isActive = status === 'active';
  const isMagenta = status === 'magenta';

  return (
    <div className={`relative px-4 py-2 cyber-cut border-2 font-mono text-sm z-10 transition-all duration-300 min-w-[100px] text-center ${isDone ? 'bg-[#0f0f0f] border-[#39FF14] text-[#39FF14] shadow-[0_0_15px_rgba(57,255,20,0.2)]' :
        isMagenta ? 'bg-[#0f0f0f] border-[#FF00FF] text-[#FF00FF] shadow-[0_0_15px_rgba(255,0,255,0.2)]' :
          isActive ? 'bg-[#0f0f0f] border-[#FF3300] text-[#FF3300] shadow-[0_0_20px_rgba(255,51,0,0.3)] animate-pulse-glow-lava' :
            'bg-[#0f0f0f] border-[#333] text-gray-400'
      }`}>
      {label}
      {isDone && (
        <div className="absolute -top-2 -right-2 w-5 h-5 bg-[#0f0f0f] border border-[#39FF14] rounded-full flex items-center justify-center">
          <Check className="w-3 h-3 text-[#39FF14]" strokeWidth={3} />
        </div>
      )}
      {isActive && <div className="absolute -top-1 -right-1 w-2 h-2 bg-[#FF3300] shadow-[0_0_8px_#FF3300] animate-ping"></div>}
    </div>
  );
}
