import React from 'react';
import { AlertTriangle } from 'lucide-react';

export function DashboardCard({ title, children, className = '' }: { title: string, children: React.ReactNode, className?: string }) {
  return (
    <div className={`bg-[#0f0f0f] border border-[#2a2a2a] cyber-cut p-6 shadow-lg relative group ${className}`}>
      {/* Subtle top highlight */}
      <div className="absolute top-0 left-0 right-0 h-px bg-gradient-to-r from-transparent via-[#404040] to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-bold text-gray-300 uppercase tracking-widest">{title}</h3>
        <AlertTriangle className="w-4 h-4 text-[#333] group-hover:text-[#FFCC00] transition-colors" />
      </div>
      {children}
    </div>
  );
}
