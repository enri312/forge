import React from 'react';
import { DashboardCard } from '../../shared/components/DashboardCard';
import { DagNode } from '../../shared/components/DagNode';
import { useTelemetry } from '../../shared/context/TelemetryContext';

export function GraphTab() {
  const { tasks } = useTelemetry();
  const taskList = Object.values(tasks);

  return (
    <div className="h-full flex flex-col">
      <DashboardCard title="Observed Tasks" className="flex-1 border-t-2 border-t-[#FF00FF]">
        <div className="relative h-full flex flex-col items-center justify-center mt-2 min-h-[400px]">
          <div
            className="absolute top-0 left-0 w-full h-full opacity-10 pointer-events-none"
            style={{ backgroundImage: 'radial-gradient(#FF00FF 1px, transparent 1px)', backgroundSize: '20px 20px' }}
          ></div>
          <div className="w-full max-w-4xl relative z-10 py-8 px-8">
            {taskList.length === 0 ? (
              <p className="text-center text-gray-500 font-mono text-sm">No task events received yet.</p>
            ) : (
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-8">
                {taskList.map(task => (
                  <DagNode key={task.name} label={task.name} status={task.state} />
                ))}
              </div>
            )}
            <p className="text-center text-[10px] text-gray-600 font-mono mt-8">
              Dependency edges require topology metadata that the backend does not emit yet.
            </p>
          </div>
        </div>
      </DashboardCard>
    </div>
  );
}
