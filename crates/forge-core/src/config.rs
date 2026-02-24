// =============================================================================
// 🔥 FORGE — Motor Core: Configuración (forge.toml)
// =============================================================================
// Parser del archivo de configuración forge.toml.
// Diseño: serde + toml para deserialización automática.
// =============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{ForgeError, ForgeResult};

/// Configuración principal del proyecto, mapeada desde forge.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForgeConfig {
    /// Metadatos del proyecto
    pub project: ProjectConfig,

    /// Configuración específica de Java (opcional)
    pub java: Option<JavaConfig>,

    /// Configuración específica de Kotlin (opcional)
    pub kotlin: Option<KotlinConfig>,

    /// Configuración específica de Python (opcional)
    pub python: Option<PythonConfig>,

    /// Dependencias del proyecto (nombre = versión)
    #[serde(default)]
    pub dependencies: HashMap<String, String>,

    /// Dependencias exclusivas para testing
    #[serde(default, rename = "test-dependencies")]
    pub test_dependencies: HashMap<String, String>,

    /// Tareas personalizadas
    #[serde(default)]
    pub tasks: HashMap<String, TaskConfig>,

    /// Hooks de ciclo de vida (pre-build, post-build, pre-test, post-test)
    #[serde(default)]
    pub hooks: HooksConfig,

    /// Sub-módulos del workspace (multi-módulo)
    #[serde(default)]
    pub modules: Vec<String>,

    /// Plugins WebAssembly instalados (Fase 17)
    #[serde(default)]
    pub plugins: HashMap<String, String>,

    /// Configuración de caché distribuido (Fase 16)
    pub cache: Option<RemoteCacheConfig>,
}

/// Configuración de servidor remoto de Caché (Distribución S3/HTTP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCacheConfig {
    /// URL del bucket o servidor (ej: `http://forge-cache.local`)
    pub remote: String,
    
    /// Token opcional (Bearer) si la subida requiere autenticación
    pub token: Option<String>,
    
    /// Controla si se subirá el caché local al servidor
    #[serde(default)]
    pub push: bool,
}

/// Metadatos generales del proyecto.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    /// Nombre del proyecto
    pub name: String,

    /// Versión del proyecto
    #[serde(default = "default_version")]
    pub version: String,

    /// Lenguaje principal: "java", "kotlin", "python"
    #[serde(default = "default_lang")]
    pub lang: String,

    /// Descripción breve del proyecto
    #[serde(default)]
    pub description: String,

    /// Directorio de salida (default: "build")
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

/// Configuración para proyectos Java.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaConfig {
    /// Directorio de código fuente
    #[serde(default = "default_java_source")]
    pub source: String,

    /// Directorio de código de tests
    #[serde(default = "default_java_test_source", rename = "test-source")]
    pub test_source: String,

    /// Versión objetivo del JDK (ej: "17", "21")
    #[serde(default = "default_java_target")]
    pub target: String,

    /// Clase principal con método main
    #[serde(rename = "main-class")]
    pub main_class: Option<String>,
}

/// Configuración para proyectos Kotlin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KotlinConfig {
    /// Directorio de código fuente
    #[serde(default = "default_kotlin_source")]
    pub source: String,

    /// Directorio de código de tests
    #[serde(default = "default_kotlin_test_source", rename = "test-source")]
    pub test_source: String,

    /// Versión objetivo de la JVM
    #[serde(default = "default_java_target")]
    pub jvm_target: String,

    /// Clase principal con método main
    #[serde(rename = "main-class")]
    pub main_class: Option<String>,
}

/// Configuración para proyectos Python.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    /// Directorio de código fuente
    #[serde(default = "default_python_source")]
    pub source: String,

    /// Script de entrada principal
    #[serde(rename = "main-script")]
    pub main_script: Option<String>,

    /// Versión de Python requerida (ej: "3.12")
    pub python_version: Option<String>,
}

/// Definición de una tarea personalizada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// Comando a ejecutar
    pub command: String,

    /// Tareas de las que depende
    #[serde(default, rename = "depends-on")]
    pub depends_on: Vec<String>,

    /// Descripción de la tarea
    #[serde(default)]
    pub description: String,
}

/// Hooks de ciclo de vida del build.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HooksConfig {
    /// Comando(s) a ejecutar ANTES de compilar
    #[serde(default, rename = "pre-build")]
    pub pre_build: Vec<String>,

    /// Comando(s) a ejecutar DESPUÉS de compilar
    #[serde(default, rename = "post-build")]
    pub post_build: Vec<String>,

    /// Comando(s) a ejecutar ANTES de testear
    #[serde(default, rename = "pre-test")]
    pub pre_test: Vec<String>,

    /// Comando(s) a ejecutar DESPUÉS de testear
    #[serde(default, rename = "post-test")]
    pub post_test: Vec<String>,
}

// ── Valores por defecto ──────────────────────────────────────────────────────

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_lang() -> String {
    "java".to_string()
}

fn default_output_dir() -> String {
    "build".to_string()
}

fn default_java_source() -> String {
    "src/main/java".to_string()
}

fn default_java_test_source() -> String {
    "src/test/java".to_string()
}

fn default_kotlin_source() -> String {
    "src/main/kotlin".to_string()
}

fn default_kotlin_test_source() -> String {
    "src/test/kotlin".to_string()
}

fn default_python_source() -> String {
    "src".to_string()
}

fn default_java_target() -> String {
    "17".to_string()
}

// ── Implementación ───────────────────────────────────────────────────────────

impl ForgeConfig {
    /// Carga la configuración desde un archivo forge.toml.
    pub fn load(project_dir: &Path) -> ForgeResult<Self> {
        let config_path = project_dir.join("forge.toml");

        if !config_path.exists() {
            return Err(ForgeError::ConfigNotFound {
                path: config_path.clone(),
            }
            .into());
        }

        let content = std::fs::read_to_string(&config_path).map_err(|e| ForgeError::IoError {
            path: config_path.clone(),
            message: e.to_string(),
        })?;

        let config: ForgeConfig =
            toml::from_str(&content).map_err(|e| ForgeError::ConfigParseError {
                message: e.to_string(),
            })?;

        config.validate()?;
        Ok(config)
    }

    /// Valida que la configuración sea coherente.
    fn validate(&self) -> ForgeResult<()> {
        // Verificar que el lenguaje sea soportado
        match self.project.lang.as_str() {
            "java" | "kotlin" | "python" => {}
            other => {
                return Err(ForgeError::UnsupportedLanguage {
                    lang: other.to_string(),
                }
                .into())
            }
        }

        // Verificar coherencia: si lang=java, debe existir [java]
        if self.project.lang == "java" && self.java.is_none() {
            tracing::warn!("Lenguaje 'java' seleccionado pero no se definió [java] en forge.toml. Usando valores por defecto.");
        }

        if self.project.lang == "kotlin" && self.kotlin.is_none() {
            tracing::warn!("Lenguaje 'kotlin' seleccionado pero no se definió [kotlin] en forge.toml. Usando valores por defecto.");
        }

        if self.project.lang == "python" && self.python.is_none() {
            tracing::warn!("Lenguaje 'python' seleccionado pero no se definió [python] en forge.toml. Usando valores por defecto.");
        }

        Ok(())
    }

    /// Genera un forge.toml de ejemplo para un lenguaje dado.
    pub fn generate_template(lang: &str) -> ForgeResult<String> {
        let template = match lang {
            "java" => include_str!("../templates/forge_java.toml"),
            "kotlin" => include_str!("../templates/forge_kotlin.toml"),
            "python" => include_str!("../templates/forge_python.toml"),
            other => {
                return Err(ForgeError::UnsupportedLanguage {
                    lang: other.to_string(),
                }
                .into())
            }
        };
        Ok(template.to_string())
    }

    /// Devuelve el directorio fuente según el lenguaje.
    pub fn source_dir(&self) -> String {
        match self.project.lang.as_str() {
            "java" => self
                .java
                .as_ref()
                .map(|j| j.source.clone())
                .unwrap_or_else(default_java_source),
            "kotlin" => self
                .kotlin
                .as_ref()
                .map(|k| k.source.clone())
                .unwrap_or_else(default_kotlin_source),
            "python" => self
                .python
                .as_ref()
                .map(|p| p.source.clone())
                .unwrap_or_else(default_python_source),
            _ => "src".to_string(),
        }
    }

    /// Devuelve la clase/script principal.
    pub fn main_entry(&self) -> Option<String> {
        match self.project.lang.as_str() {
            "java" => self.java.as_ref().and_then(|j| j.main_class.clone()),
            "kotlin" => self.kotlin.as_ref().and_then(|k| k.main_class.clone()),
            "python" => self.python.as_ref().and_then(|p| p.main_script.clone()),
            _ => None,
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_java_config() {
        let toml_str = r#"
[project]
name = "mi-app"
version = "1.0.0"
lang = "java"

[java]
source = "src/main/java"
target = "21"
main-class = "com.ejemplo.Main"

[dependencies]
"com.google.guava:guava" = "33.0.0"

[test-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.10.1"
"#;

        let config: ForgeConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "mi-app");
        assert_eq!(config.project.lang, "java");
        assert_eq!(config.java.as_ref().unwrap().target, "21");
        assert_eq!(config.java.as_ref().unwrap().test_source, "src/test/java");
        assert_eq!(
            config.java.as_ref().unwrap().main_class,
            Some("com.ejemplo.Main".to_string())
        );
        assert!(config.dependencies.contains_key("com.google.guava:guava"));
        assert!(config.test_dependencies.contains_key("org.junit.jupiter:junit-jupiter-api"));
    }

    #[test]
    fn test_parse_python_config() {
        let toml_str = r#"
[project]
name = "mi-script"
lang = "python"

[python]
source = "src"
main-script = "main.py"
"#;

        let config: ForgeConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.lang, "python");
        assert_eq!(
            config.python.as_ref().unwrap().main_script,
            Some("main.py".to_string())
        );
    }

    #[test]
    fn test_unsupported_language() {
        let toml_str = r#"
[project]
name = "test"
lang = "go"
"#;

        let config: ForgeConfig = toml::from_str(toml_str).unwrap();
        assert!(config.validate().is_err());
    }
}
