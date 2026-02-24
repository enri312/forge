// =============================================================================
// 🔥 FORGE — Motor Core: Tipos de Error
// =============================================================================
// Manejo de errores centralizado con tipos descriptivos.
// Patrón moderno: thiserror para errores tipados + anyhow para propagación.
// =============================================================================

use std::path::PathBuf;

/// Errores específicos del motor FORGE.
/// Cada variante describe un problema concreto con contexto útil para el usuario.
#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    // ── Configuración ────────────────────────────────────────────────────
    #[error("No se encontró 'forge.toml' en: {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Error al parsear 'forge.toml': {message}")]
    ConfigParseError { message: String },

    #[error("Campo requerido '{field}' no encontrado en forge.toml")]
    ConfigMissingField { field: String },

    #[error("Lenguaje no soportado: '{lang}'. Usa: java, kotlin, python")]
    UnsupportedLanguage { lang: String },

    // ── Grafo de Tareas (DAG) ────────────────────────────────────────────
    #[error("Dependencia circular detectada: {cycle}")]
    CyclicDependency { cycle: String },

    #[error("Tarea no encontrada: '{task_name}'")]
    TaskNotFound { task_name: String },

    // ── Ejecución ────────────────────────────────────────────────────────
    #[error("La tarea '{task_name}' falló con código de salida: {exit_code}")]
    TaskFailed { task_name: String, exit_code: i32 },

    #[error("Comando no encontrado: '{command}'. ¿Está instalado y en el PATH?")]
    CommandNotFound { command: String },

    #[error("Timeout al ejecutar la tarea '{task_name}' después de {seconds}s")]
    TaskTimeout { task_name: String, seconds: u64 },

    // ── Dependencias ─────────────────────────────────────────────────────
    #[error("No se pudo resolver la dependencia: '{dependency}'")]
    DependencyResolutionFailed { dependency: String },

    #[error("Error al descargar '{url}': {message}")]
    DownloadError { url: String, message: String },

    // ── Sistema de Archivos ──────────────────────────────────────────────
    #[error("Error de E/S en '{path}': {message}")]
    IoError { path: PathBuf, message: String },

    // ── Caché ────────────────────────────────────────────────────────────
    #[error("Caché corrupta en '{path}'. Ejecuta 'forge clean' para regenerar.")]
    CacheCorrupted { path: PathBuf },
}

/// Resultado tipado de FORGE usando anyhow para contexto flexible.
pub type ForgeResult<T> = anyhow::Result<T>;
