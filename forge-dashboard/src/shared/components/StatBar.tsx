import React from 'react';

export function StatBar({ label, value, percent, color, glow }: { label: string, value: string, percent: number, color: string, glow: string }) {
  return (
    <div>
      <div className="flex justify-between text-xs font-mono mb-1.5">
        <span className="text-gray-400">{label}</span>
        <span className="text-white drop-shadow-[0_0_2px_rgba(255,255,255,0.5)]">{value}</span>
      </div>
      <div className="h-2 w-full bg-[#1a1a1a] cyber-cut overflow-hidden border border-[#333]">
        <div className={`h-full ${color} animate-fill-bar`} style={{ width: `${percent}%`, boxShadow: `0 0 10px ${glow}` }}></div>
      </div>
    </div>
  );
}
