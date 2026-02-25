import React from 'react';
import { DashboardCard } from '../../shared/components/DashboardCard';
import { DagNode } from '../../shared/components/DagNode';
import { ArrowDown } from 'lucide-react';
import { useTelemetry } from '../../shared/context/TelemetryContext';

export function GraphTab() {
  const { tasks } = useTelemetry();
  return (
    <div className="h-full flex flex-col">
      <DashboardCard title="Full Dependency Graph (DAG)" className="flex-1 border-t-2 border-t-[#FF00FF]">
        <div className="relative h-full flex flex-col items-center justify-center mt-2 min-h-[400px]">
          <div className="absolute top-0 left-0 w-full h-full opacity-10 pointer-events-none"
            style={{ backgroundImage: 'radial-gradient(#FF00FF 1px, transparent 1px)', backgroundSize: '20px 20px' }}></div>
          <div className="flex flex-col items-center w-full max-w-3xl relative z-10 py-8 px-8">
            {/* Layer 1: Core */}
            <DagNode label="core" status={tasks['core']?.state || 'pending'} />

            {/* Connect Layer 1 to Layer 2 */}
            <div className="flex flex-col items-center w-full">
              <div className="w-0.5 h-8 bg-gradient-to-b from-[#39FF14] to-[#00FFFF] relative shadow-[0_0_8px_rgba(0,255,255,0.5)]"></div>
              <div className="w-[256px] h-0.5 bg-[#00FFFF] relative shadow-[0_0_8px_rgba(0,255,255,0.5)]"></div>
              <div className="flex justify-between w-[256px]">
                <div className="w-0.5 h-8 bg-gradient-to-b from-[#00FFFF] to-[#FF00FF] relative">
                  <ArrowDown className="absolute -bottom-4 -left-[11px] w-6 h-6 text-[#FF00FF]" />
                </div>
                <div className="w-0.5 h-8 bg-gradient-to-b from-[#00FFFF] to-[#39FF14] relative">
                  <ArrowDown className="absolute -bottom-4 -left-[11px] w-6 h-6 text-[#39FF14]" />
                </div>
              </div>
            </div>

            {/* Layer 2: Auth and Database */}
            <div className="flex justify-center gap-[156px] w-full mt-2">
              <DagNode label="auth" status={tasks['auth']?.state || 'pending'} />
              <DagNode label="database" status={tasks['database']?.state || 'pending'} />
            </div>

            {/* Connect Layer 2 to Layer 3 */}
            <div className="flex flex-col items-center w-full mt-2">
              <div className="flex justify-between w-[256px]">
                <div className="w-0.5 h-8 bg-gradient-to-b from-[#FF00FF] to-[#FF3300] relative"></div>
                <div className="w-0.5 h-8 bg-gradient-to-b from-[#39FF14] to-[#FF3300] relative"></div>
              </div>
              <div className="w-[256px] h-0.5 bg-[#FF3300] relative shadow-[0_0_8px_rgba(255,51,0,0.5)]"></div>
              <div className="w-0.5 h-8 bg-[#FF3300] relative shadow-[0_0_8px_rgba(255,51,0,0.5)]">
                <ArrowDown className="absolute -bottom-4 -left-[11px] w-6 h-6 text-[#FF3300]" />
              </div>
            </div>

            {/* Layer 3: API */}
            <div className="mt-2">
              <DagNode label="api" status={tasks['api']?.state || 'pending'} />
            </div>
          </div>
        </div>
      </DashboardCard>
    </div>
  );
}
