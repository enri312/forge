import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';

// === TypeScript Interfaces para los eventos disparados desde Rust ===

export interface TaskStatus {
    name: String;
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
        tSavedSecs: number;
        hitRate: number;
        bandwidthSavedGb: number;
        localHits: number;
        s3Hits: number;
        misses: number;
    };
}

const TelemetryContext = createContext<TelemetryState | undefined>(undefined);

export function TelemetryProvider({ children }: { children: ReactNode }) {
    const [state, setState] = useState<TelemetryState>({
        connected: false,
        tasks: {
            'core': { name: 'core', state: 'pending' },
            'auth': { name: 'auth', state: 'pending' },
            'database': { name: 'database', state: 'pending' },
            'api': { name: 'api', state: 'pending' },
        },
        logs: [],
        cacheStats: {
            tSavedSecs: 0,
            hitRate: 0,
            bandwidthSavedGb: 0,
            localHits: 0,
            s3Hits: 0,
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
                    const newState = { ...prev };

                    if (data.type === 'TaskStarted') {
                        newState.tasks = {
                            ...prev.tasks,
                            [data.name]: { name: data.name, state: 'running' }
                        };
                    }
                    else if (data.type === 'TaskFinished') {
                        let nextState: 'success' | 'cached' = 'success';
                        if (data.cached) {
                            nextState = 'cached';
                            newState.cacheStats.tSavedSecs += (data.time_ms / 1000.0);
                            if (data.cache_source === 's3') newState.cacheStats.s3Hits++;
                            else newState.cacheStats.localHits++;
                        } else {
                            newState.cacheStats.misses++;
                        }

                        // A recalculator naive del hitRate general
                        let total = newState.cacheStats.misses + newState.cacheStats.localHits + newState.cacheStats.s3Hits;
                        let hits = newState.cacheStats.localHits + newState.cacheStats.s3Hits;
                        if (total > 0) newState.cacheStats.hitRate = Math.round((hits / total) * 100);

                        newState.tasks = {
                            ...prev.tasks,
                            [data.name]: {
                                name: data.name,
                                state: nextState,
                                time_ms: data.time_ms,
                                cache_source: data.cache_source
                            }
                        };
                    }
                    else if (data.type === 'LogMessage') {
                        logCounter++;
                        newState.logs = [...prev.logs, {
                            id: logCounter,
                            level: data.level,
                            text: data.text,
                            timestamp: new Date().toISOString()
                        }];
                    }

                    return newState;
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

export function useTelemetry() {
    const context = useContext(TelemetryContext);
    if (context === undefined) {
        throw new Error('useTelemetry must be used within a TelemetryProvider');
    }
    return context;
}
