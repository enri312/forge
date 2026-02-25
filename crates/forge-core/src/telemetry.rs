use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

// =============================================================================
// 🔥 FORGE — Telemetría en Tiempo Real (Server-Sent Events)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ForgeEvent {
    TaskStarted {
        name: String,
    },
    TaskFinished {
        name: String,
        time_ms: u64,
        cached: bool,
        cache_source: Option<String>,
    },
    LogMessage {
        level: String,
        text: String,
    },
}

#[derive(Clone)]
pub struct EventBus {
    pub sender: broadcast::Sender<ForgeEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        // Canal de transmisión (broadcast) con capacidad de 1024 mensajes
        let (sender, _) = broadcast::channel(1024);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ForgeEvent> {
        self.sender.subscribe()
    }

    pub fn send(&self, event: ForgeEvent) {
        // Ignoramos el error si no hay suscriptores vivos escuchando
        let _ = self.sender.send(event);
    }
}

use std::sync::OnceLock;

// Inicialización de un bus global usando OnceLock (Standard Library >= 1.70)
pub fn global_event_bus() -> &'static EventBus {
    static BUS: OnceLock<EventBus> = OnceLock::new();
    BUS.get_or_init(EventBus::new)
}
