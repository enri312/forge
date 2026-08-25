// =============================================================================
// 🔥 FORGE — Motor Core: Configuración (forge.toml)
// =============================================================================
// Parser del archivo de configuración forge.toml.
// Diseño: serde + toml para deserialización automática.
// =============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Component, Path};

use crate::error::{ForgeError, ForgeResult};

/// Configuración principal del proyecto, mapeada desde forge.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
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

    /// Configuración de caché distribuido (Fase 16)
    pub cache: Option<RemoteCacheConfig>,
}

/// Configuración de servidor remoto de Caché (Distribución S3/HTTP)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

    /// Versión objetivo de Java/JDK (ej: "17", "21", "25")
    #[serde(default, rename = "java-version")]
    pub java_version: Option<String>,

    /// Directorio de salida (default: "build")
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

/// Configuración para proyectos Java.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
    /// Parsea y valida una configuración desde texto TOML.
    ///
    /// Esta ruta compartida evita que herramientas como el LSP acepten una
    /// configuración que luego sería rechazada por la CLI.
    pub fn parse(content: &str) -> ForgeResult<Self> {
        let config: ForgeConfig =
            toml::from_str(content).map_err(|e| ForgeError::ConfigParseError {
                message: e.to_string(),
            })?;

        config.validate()?;
        Ok(config)
    }

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

        Self::parse(&content)
    }

    /// Obtiene el classpath compilado de los sub-módulos locales definidos con `path:`
    pub fn get_local_classpath(&self, project_dir: &Path) -> String {
        let mut cp = Vec::new();
        let sep = if cfg!(target_os = "windows") {
            ";"
        } else {
            ":"
        };

        let all_deps = self
            .dependencies
            .values()
            .chain(self.test_dependencies.values());

        for val in all_deps {
            if val.starts_with("path:") {
                let rel_path = val.trim_start_matches("path:");
                let dep_dir = project_dir.join(rel_path);

                // 1. Rutina NATIVA: Si el subproyecto es de Forge, derivamos el classpath desde su `forge.toml`
                if let Ok(dep_config) = ForgeConfig::load(&dep_dir) {
                    let output_dir = dep_dir.join(&dep_config.project.output_dir);
                    let jar_path = output_dir.join(format!("{}.jar", dep_config.project.name));

                    // 1a. JAR empaquetado por Forge
                    if jar_path.exists() {
                        cp.push(jar_path.to_string_lossy().to_string());
                        continue;
                    }

                    // 1b. JARs empaquetados por Gradle en build/libs/ (JPMS-compatible)
                    let gradle_libs = output_dir.join("libs");
                    if gradle_libs.exists() {
                        let mut found_jar = false;
                        if let Ok(entries) = std::fs::read_dir(&gradle_libs) {
                            for res in entries.flatten() {
                                let f = res.path();
                                if f.is_file()
                                    && f.extension().and_then(|s| s.to_str()) == Some("jar")
                                    && !f.to_string_lossy().contains("-sources")
                                    && !f.to_string_lossy().contains("-javadoc")
                                {
                                    cp.push(f.to_string_lossy().to_string());
                                    found_jar = true;
                                }
                            }
                        }
                        if found_jar {
                            continue;
                        }
                    }

                    // 1c. Clases sueltas: Gradle usa build/classes/java/main, Forge usa build/classes
                    let gradle_classes = output_dir.join("classes").join("java").join("main");
                    let forge_classes = output_dir.join("classes");
                    if gradle_classes.exists() {
                        cp.push(gradle_classes.to_string_lossy().to_string());
                        continue;
                    } else if forge_classes.exists() {
                        cp.push(forge_classes.to_string_lossy().to_string());
                        continue;
                    }
                    // Si nada existe, caemos al bloque híbrido
                }

                // 2. Rutina HÍBRIDA 2026: Prioridad → JARs empaquetados (necesarios para JPMS automatic modules)
                let mut injected = false;

                // 2a. Buscar JARs en build/libs/ (Gradle) o target/ (Maven) — PRIORIDAD MÁXIMA para JPMS
                let jar_search_dirs =
                    vec![dep_dir.join("build").join("libs"), dep_dir.join("target")];
                for jar_dir in &jar_search_dirs {
                    if let Ok(entries) = std::fs::read_dir(jar_dir) {
                        for res in entries.flatten() {
                            let f_path = res.path();
                            if f_path.is_file()
                                && f_path.extension().and_then(|s| s.to_str()) == Some("jar")
                                && !f_path.to_string_lossy().contains("-sources")
                                && !f_path.to_string_lossy().contains("-javadoc")
                            {
                                cp.push(f_path.to_string_lossy().to_string());
                                injected = true;
                            }
                        }
                    }
                }

                // 2b. Fallback: carpetas de clases sueltas (solo si no encontramos JARs)
                if !injected {
                    let possible_outputs = vec![
                        dep_dir
                            .join("build")
                            .join("classes")
                            .join("java")
                            .join("main"),
                        dep_dir
                            .join("build")
                            .join("classes")
                            .join("kotlin")
                            .join("main"),
                        dep_dir.join("target").join("classes"),
                        dep_dir.join("out").join("production").join("classes"),
                    ];
                    for p_out in possible_outputs {
                        if p_out.exists() {
                            cp.push(p_out.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        cp.join(sep)
    }

    /// Valida que la configuración sea coherente.
    pub fn validate(&self) -> ForgeResult<()> {
        if self.project.name.trim().is_empty() {
            return Err(ForgeError::ConfigMissingField {
                field: "project.name".to_string(),
            }
            .into());
        }

        validate_file_component("project.name", &self.project.name)?;
        validate_project_relative_path("project.output_dir", &self.project.output_dir)?;

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

        for (name, value) in self
            .dependencies
            .iter()
            .chain(self.test_dependencies.iter())
        {
            if name.trim().is_empty() {
                return Err(config_error(
                    "Las dependencias no pueden tener un nombre vacío",
                ));
            }
            if value.trim().is_empty() {
                return Err(config_error(format!(
                    "La dependencia '{}' no puede tener una versión vacía",
                    name
                )));
            }
            if value.starts_with("path:") && value.trim_start_matches("path:").trim().is_empty() {
                return Err(config_error(format!(
                    "La dependencia local '{}' debe indicar una ruta después de 'path:'",
                    name
                )));
            }
        }

        const BUILTIN_TASKS: &[&str] = &[
            "build", "run", "test", "clean", "deps", "fmt", "lint", "package",
        ];
        let mut task_graph = crate::dag::TaskGraph::new();
        for builtin in BUILTIN_TASKS {
            task_graph.add_task(crate::dag::Task {
                name: (*builtin).to_string(),
                description: format!("Tarea interna {}", builtin),
                depends_on: Vec::new(),
                action: crate::dag::TaskAction::Composite,
            })?;
        }
        for (name, task) in &self.tasks {
            if name.trim().is_empty() || name.chars().any(char::is_control) {
                return Err(config_error("Los nombres de tareas no pueden estar vacíos ni contener caracteres de control"));
            }
            if BUILTIN_TASKS.contains(&name.as_str()) {
                return Err(config_error(format!(
                    "La tarea '{}' usa un nombre reservado por FORGE",
                    name
                )));
            }
            if task.command.trim().is_empty() {
                return Err(config_error(format!(
                    "La tarea '{}' debe contener un comando",
                    name
                )));
            }
            task_graph.add_task(crate::dag::Task {
                name: name.clone(),
                description: task.description.clone(),
                depends_on: task.depends_on.clone(),
                action: crate::dag::TaskAction::Composite,
            })?;
        }
        task_graph.validate()?;

        if let Some(cache) = &self.cache {
            let remote = cache.remote.trim();
            if !(remote.starts_with("https://") || remote.starts_with("http://")) {
                return Err(config_error(
                    "cache.remote debe ser una URL HTTP o HTTPS válida",
                ));
            }

            let is_local_http = remote.starts_with("http://localhost")
                || remote.starts_with("http://127.0.0.1")
                || remote.starts_with("http://[::1]");
            if cache.token.is_some() && remote.starts_with("http://") && !is_local_http {
                return Err(config_error(
                    "cache.token no puede enviarse por HTTP sin cifrar; usa HTTPS",
                ));
            }
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

fn config_error(message: impl Into<String>) -> anyhow::Error {
    ForgeError::ConfigParseError {
        message: message.into(),
    }
    .into()
}

fn validate_file_component(field: &str, value: &str) -> ForgeResult<()> {
    let path = Path::new(value);
    let mut components = path.components();
    let is_single_normal =
        matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
    let has_forbidden_windows_char = value.chars().any(|c| {
        c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
    });

    if !is_single_normal || value == "." || value == ".." || has_forbidden_windows_char {
        return Err(config_error(format!(
            "{} debe ser un nombre seguro, no una ruta: '{}'",
            field, value
        )));
    }

    Ok(())
}

fn validate_project_relative_path(field: &str, value: &str) -> ForgeResult<()> {
    let path = Path::new(value);
    if value.trim().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(config_error(format!(
            "{} debe ser una ruta relativa dentro del proyecto: '{}'",
            field, value
        )));
    }

    Ok(())
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
        assert!(config
            .test_dependencies
            .contains_key("org.junit.jupiter:junit-jupiter-api"));
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

    #[test]
    fn rejects_unknown_fields_instead_of_ignoring_typos() {
        let toml_str = r#"
[project]
name = "test"
lang = "java"
output-dir = "dist"
"#;

        assert!(ForgeConfig::parse(toml_str).is_err());
    }

    #[test]
    fn rejects_output_paths_outside_the_project() {
        let toml_str = r#"
[project]
name = "test"
lang = "java"
output_dir = "../outside"
"#;

        assert!(ForgeConfig::parse(toml_str).is_err());
    }

    #[test]
    fn rejects_insecure_remote_cache_tokens() {
        let toml_str = r#"
[project]
name = "test"
lang = "java"

[cache]
remote = "http://cache.example.com"
token = "secret"
"#;

        assert!(ForgeConfig::parse(toml_str).is_err());
    }

    #[test]
    fn rejects_missing_and_cyclic_task_dependencies() {
        let missing = r#"
[project]
name = "test"
lang = "java"

[tasks.deploy]
command = "echo deploy"
depends-on = ["does-not-exist"]
"#;
        assert!(ForgeConfig::parse(missing).is_err());

        let cyclic = r#"
[project]
name = "test"
lang = "java"

[tasks.a]
command = "echo a"
depends-on = ["b"]

[tasks.b]
command = "echo b"
depends-on = ["a"]
"#;
        assert!(ForgeConfig::parse(cyclic).is_err());
    }
}
