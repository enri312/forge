// =============================================================================
// 🔥 FORGE — Motor Core: Tipos de Error (v0.4.0)
// =============================================================================
// Manejo de errores centralizado con tipos descriptivos y sugerencias
// contextuales para resolver cada problema.
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

impl ForgeError {
    /// Devuelve una sugerencia contextual de resolución para el error.
    pub fn suggestion(&self) -> &'static str {
        match self {
            Self::ConfigNotFound { .. } => {
                "💡 Ejecuta 'forge init <lang>' para crear un forge.toml, o verifica que estás en el directorio correcto."
            }
            Self::ConfigParseError { .. } => {
                "💡 Verifica la sintaxis TOML de tu forge.toml. Usa un validador como https://www.toml-lint.com/"
            }
            Self::ConfigMissingField { field, .. } => {
                match field.as_str() {
                    "name" => "💡 Añade 'name = \"mi-proyecto\"' en la sección [project] de forge.toml",
                    "lang" => "💡 Añade 'lang = \"java\"' (o kotlin/python) en la sección [project] de forge.toml",
                    _ => "💡 Revisa la documentación: https://github.com/enri312/forge#configuración",
                }
            }
            Self::UnsupportedLanguage { .. } => {
                "💡 FORGE soporta: java, kotlin, python. Verifica el campo 'lang' en [project]"
            }
            Self::CyclicDependency { .. } => {
                "💡 Revisa las secciones [tasks.*.depends-on] en tu forge.toml para romper el ciclo"
            }
            Self::TaskNotFound { .. } => {
                "💡 Lista las tareas disponibles con 'forge info' o revisa la sección [tasks] de forge.toml"
            }
            Self::TaskFailed { .. } => {
                "💡 Revisa la salida del compilador arriba. Usa 'forge build --verbose' para más detalle"
            }
            Self::CommandNotFound { command, .. } => {
                match command.as_str() {
                    "javac" | "java" => "💡 Instala JDK 17+: https://adoptium.net/ y asegúrate que 'javac' está en PATH",
                    "kotlinc" => "💡 Instala Kotlin: https://kotlinlang.org/docs/command-line.html",
                    "python" | "python3" | "pip" => "💡 Instala Python 3.12+: https://www.python.org/downloads/",
                    "pytest" => "💡 Instala pytest: pip install pytest",
                    _ => "💡 Verifica que el comando está instalado y accesible en tu PATH del sistema",
                }
            }
            Self::TaskTimeout { .. } => {
                "💡 Considera aumentar el timeout o dividir la tarea en sub-tareas más pequeñas"
            }
            Self::DependencyResolutionFailed { .. } => {
                "💡 Verifica el formato en [dependencies]: \"groupId:artifactId\" = \"versión\". Ejemplo: \"com.google.gson:gson\" = \"2.11.0\""
            }
            Self::DownloadError { .. } => {
                "💡 Verifica tu conexión a internet y que la dependencia exista en Maven Central / PyPI"
            }
            Self::IoError { .. } => {
                "💡 Verifica permisos de escritura en el directorio del proyecto y espacio disponible en disco"
            }
            Self::CacheCorrupted { .. } => {
                "💡 Ejecuta 'forge clean' para eliminar la caché y reconstruir desde cero"
            }
        }
    }
}

/// Resultado tipado de FORGE usando anyhow para contexto flexible.
pub type ForgeResult<T> = anyhow::Result<T>;
