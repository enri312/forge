import React from 'react';
import { Box, Clock, Server } from 'lucide-react';

export function Header() {
  return (
    <header className="h-16 bg-[#0f0f0f]/90 backdrop-blur-md border-b border-[#2a2a2a] flex items-center justify-between px-8 z-10 shadow-md">
      <div className="flex items-center gap-4">
        <h2 className="text-lg font-semibold text-white flex items-center gap-2">
          <Box className="w-5 h-5 text-[#FFCC00] drop-shadow-[0_0_5px_rgba(255,204,0,0.5)]" />
          My_Service_API
        </h2>
        <div className="p-[1px] cyber-cut bg-[#00FFFF]/50 shadow-[0_0_10px_rgba(0,255,255,0.2)]">
          <div className="px-3 py-1 cyber-cut bg-[#0f0f0f] text-xs font-mono text-[#00FFFF]">
            master #a1b2c3d
          </div>
        </div>
      </div>
      
      <div className="flex items-center gap-6">
        <div className="p-[1px] cyber-cut bg-[#333]">
          <div className="flex items-center gap-2 text-sm font-mono bg-[#0f0f0f] px-3 py-1 cyber-cut">
            <Clock className="w-4 h-4 text-gray-400" />
            <span className="text-gray-400">Build Time:</span>
            <span className="text-[#00FFFF] font-bold drop-shadow-[0_0_5px_rgba(0,255,255,0.5)]">0.84s</span>
          </div>
        </div>
        <div className="p-[1px] cyber-cut bg-[#333]">
          <div className="flex items-center gap-2 text-sm font-mono bg-[#0f0f0f] px-3 py-1 cyber-cut">
            <Server className="w-4 h-4 text-gray-400" />
            <span className="text-gray-400">Remote Cache:</span>
            <span className="flex items-center gap-1.5 text-white">
              Connected
              <span className="w-2 h-2 bg-[#39FF14] shadow-[0_0_8px_#39FF14] animate-pulse"></span>
            </span>
          </div>
        </div>
      </div>
    </header>
  );
}
