import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';

// === TypeScript Interfaces para los eventos disparados desde Rust ===

export interface TaskStatus {
    name: string;
    state: 'pending' | 'running' | 'success' | 'cached' | 'failed';
    time_ms?: number;
    cache_source?: string;
}

export interface LogEntry {
    id: number;
    level: string;
    text: string;
    timestamp: string;
}

export interface TelemetryState {
    connected: boolean;
    tasks: Record<string, TaskStatus>;
    logs: LogEntry[];
    cacheStats: {
        hitRate: number;
        localHits: number;
        remoteHits: number;
        misses: number;
    };
}

const TelemetryContext = createContext<TelemetryState | undefined>(undefined);

export function TelemetryProvider({ children }: { children: ReactNode }) {
    const [state, setState] = useState<TelemetryState>({
        connected: false,
        tasks: {},
        logs: [],
        cacheStats: {
            hitRate: 0,
            localHits: 0,
            remoteHits: 0,
            misses: 0,
        }
    });

    useEffect(() => {
        // Apuntamos al endpoint SSE expuesto por Axum en Rust
        const eventSource = new EventSource('/api/events');
        let logCounter = 0;

        eventSource.onopen = () => {
            console.log('🔗 [Telemetry] SSE Connected to Rust EventBus');
            setState(prev => ({ ...prev, connected: true }));
        };

        eventSource.onerror = (error) => {
            console.error('❌ [Telemetry] SSE Error:', error);
            setState(prev => ({ ...prev, connected: false }));
        };

        eventSource.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);

                setState(prev => {
                    if (data.type === 'TaskStarted') {
                        return {
                            ...prev,
                            tasks: {
                                ...prev.tasks,
                                [data.name]: { name: data.name, state: 'running' }
                            }
                        };
                    }
                    else if (data.type === 'TaskFinished') {
                        let nextState: 'success' | 'cached' | 'failed' = data.success === false
                            ? 'failed'
                            : data.cached ? 'cached' : 'success';
                        const cacheStats = { ...prev.cacheStats };
                        if (data.success !== false && data.cached) {
                            if (data.cache_source === 'remote') cacheStats.remoteHits++;
                            else cacheStats.localHits++;
                        } else if (data.success !== false) {
                            cacheStats.misses++;
                        }

                        const total = cacheStats.misses + cacheStats.localHits + cacheStats.remoteHits;
                        const hits = cacheStats.localHits + cacheStats.remoteHits;
                        if (total > 0) cacheStats.hitRate = Math.round((hits / total) * 100);

                        return {
                            ...prev,
                            cacheStats,
                            tasks: {
                                ...prev.tasks,
                                [data.name]: {
                                    name: data.name,
                                    state: nextState,
                                    time_ms: data.time_ms,
                                    cache_source: data.cache_source
                                }
                            }
                        };
                    }
                    else if (data.type === 'LogMessage') {
                        logCounter++;
                        return {
                            ...prev,
                            logs: [...prev.logs, {
                                id: logCounter,
                                level: data.level,
                                text: data.text,
                                timestamp: new Date().toISOString()
                            }]
                        };
                    }

                    return prev;
                });
            } catch (err) {
                console.error('Error parsing SSE event', err);
            }
        };

        return () => {
            eventSource.close();
            console.log('🔌 [Telemetry] SSE Disconnected');
        };
    }, []);

    return (
        <TelemetryContext.Provider value={state}>
            {children}
        </TelemetryContext.Provider>
    );
}

export function useTelemetry(): TelemetryState {
    const context = useContext(TelemetryContext);
    if (context === undefined) {
        throw new Error('useTelemetry must be used within a TelemetryProvider');
    }
    return context;
}
