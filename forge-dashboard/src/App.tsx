import React, { useState } from 'react';
import { Sidebar } from './shared/components/Sidebar';
import { Header } from './shared/components/Header';
import { GeneralTab } from './features/general/GeneralTab';
import { GraphTab } from './features/graph/GraphTab';
import { CacheTab } from './features/cache/CacheTab';
import { SettingsTab } from './features/settings/SettingsTab';

export default function App() {
  const [activeTab, setActiveTab] = useState('general');

  return (
    <div className="flex h-screen w-full bg-[#050505] text-[#e5e5e5] font-sans overflow-hidden selection:bg-[#FF3300] selection:text-white">
      <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} />

      <main className="flex-1 flex flex-col h-full overflow-hidden relative">
        {/* Background Grid Effect */}
        <div className="absolute inset-0 pointer-events-none" 
             style={{ backgroundImage: 'linear-gradient(#1a1a1a 1px, transparent 1px), linear-gradient(90deg, #1a1a1a 1px, transparent 1px)', backgroundSize: '40px 40px', opacity: 0.3 }}>
        </div>

        <Header />

        {/* Dashboard Content Area */}
        <div className="flex-1 overflow-y-auto p-8 z-10">
          <div className="max-w-7xl mx-auto h-full">
            {activeTab === 'general' && <GeneralTab />}
            {activeTab === 'graph' && <GraphTab />}
            {activeTab === 'cache' && <CacheTab />}
            {activeTab === 'settings' && <SettingsTab />}
          </div>
        </div>
      </main>
    </div>
  );
}
