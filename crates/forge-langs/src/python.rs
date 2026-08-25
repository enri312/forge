// =============================================================================
// 🔥 FORGE — Módulos de Lenguaje: Python
// =============================================================================
// Gestión de proyectos Python: entornos virtuales, dependencias, ejecución.
// =============================================================================

use std::path::Path;
use std::process::Stdio;

use colored::Colorize;

use cyrce_forge_core::config::ForgeConfig;
use cyrce_forge_core::error::{ForgeError, ForgeResult};

/// Módulo de gestión Python.
pub struct PythonModule;

impl PythonModule {
    /// Prepara el entorno Python (crea venv si no existe, instala deps).
    pub async fn setup(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        let venv_dir = project_dir.join(".forge").join("venv");

        // Crear entorno virtual si no existe
        if !venv_dir.exists() {
            println!("   {}", "🐍 Creando entorno virtual Python...".cyan());

            let python_cmd = Self::find_python().await?;

            let output = tokio::process::Command::new(&python_cmd)
                .args(["-m", "venv"])
                .arg(&venv_dir)
                .current_dir(project_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| ForgeError::CommandNotFound {
                    command: format!("{}: {}", python_cmd, e),
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ForgeError::TaskFailed {
                    task_name: format!("python venv: {}", stderr),
                    exit_code: output.status.code().unwrap_or(-1),
                }
                .into());
            }

            println!("   {}", "✅ Entorno virtual creado".green());
        }

        // Instalar dependencias si hay alguna
        if !config.dependencies.is_empty() {
            Self::install_deps(config, project_dir).await?;
        }

        Ok(())
    }

    /// Instala dependencias Python con pip.
    async fn install_deps(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        let pip = Self::pip_path(project_dir);

        println!(
            "   {}",
            format!(
                "📦 Instalando {} dependencias Python...",
                config.dependencies.len()
            )
            .cyan()
        );

        // Construir lista de dependencias con versiones
        let deps: Vec<String> = config
            .dependencies
            .iter()
            .map(|(name, version)| {
                if version == "*" || version.is_empty() {
                    name.clone()
                } else {
                    format!("{}=={}", name, version)
                }
            })
            .collect();

        let mut cmd = tokio::process::Command::new(&pip);
        cmd.arg("install").args(&deps);
        cmd.current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| ForgeError::CommandNotFound {
                command: format!("pip: {}", e),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ForgeError::TaskFailed {
                task_name: format!("pip install: {}", stderr),
                exit_code: output.status.code().unwrap_or(-1),
            }
            .into());
        }

        println!(
            "   {}",
            format!("✅ {} dependencias instaladas", deps.len()).green()
        );

        Ok(())
    }

    /// "Compila" un proyecto Python (verifica sintaxis).
    pub async fn compile(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        Self::setup(config, project_dir).await?;

        let python_config = config.python.as_ref();
        let source_dir =
            project_dir.join(python_config.map(|p| p.source.as_str()).unwrap_or("src"));

        if !source_dir.exists() {
            return Err(ForgeError::IoError {
                path: source_dir,
                message: "Directorio fuente no existe. ¿Olvidaste crear tus archivos .py?"
                    .to_string(),
            }
            .into());
        }

        println!("   {}", "🐍 Verificando sintaxis Python...".cyan());

        let python = Self::python_path(project_dir);

        let output = tokio::process::Command::new(&python)
            .args(["-m", "py_compile"])
            .arg(
                config
                    .main_entry()
                    .map(|s| source_dir.join(s).to_string_lossy().to_string())
                    .unwrap_or_else(|| source_dir.to_string_lossy().to_string()),
            )
            .current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                println!("   {}", "✅ Sintaxis Python válida".green());
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Err(ForgeError::TaskFailed {
                    task_name: format!("python syntax: {}", stderr.trim()),
                    exit_code: out.status.code().unwrap_or(-1),
                }
                .into());
            }
            Err(e) => {
                return Err(ForgeError::CommandNotFound {
                    command: format!("python del entorno virtual: {}", e),
                }
                .into());
            }
        }

        Ok(())
    }

    /// Ejecuta el proyecto Python.
    pub async fn run(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        let main_script = config
            .main_entry()
            .ok_or_else(|| ForgeError::ConfigMissingField {
                field: "python.main-script".to_string(),
            })?;

        let python_config = config.python.as_ref();
        let source_dir =
            project_dir.join(python_config.map(|p| p.source.as_str()).unwrap_or("src"));

        let script_path = source_dir.join(&main_script);

        if !script_path.exists() {
            return Err(ForgeError::IoError {
                path: script_path,
                message: format!("Script principal '{}' no encontrado", main_script),
            }
            .into());
        }

        // Preparar entorno si es necesario
        Self::setup(config, project_dir).await?;

        let python = Self::python_path(project_dir);

        println!("   {}", format!("🚀 Ejecutando {}...", main_script).cyan());
        println!();

        let mut cmd = tokio::process::Command::new(&python);
        cmd.arg(&script_path)
            .current_dir(project_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd
            .status()
            .await
            .map_err(|e| ForgeError::CommandNotFound {
                command: format!("python: {}", e),
            })?;

        if !status.success() {
            return Err(ForgeError::TaskFailed {
                task_name: "python".to_string(),
                exit_code: status.code().unwrap_or(-1),
            }
            .into());
        }

        Ok(())
    }

    /// Ejecuta tests Python.
    pub async fn test(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        Self::setup(config, project_dir).await?;

        let python = Self::python_path(project_dir);

        // pytest es el runner documentado por FORGE. Instalarlo de forma
        // explícita evita que un módulo ausente se confunda con tests fallidos
        // o, peor aún, con una ejecución de unittest que descubre cero tests.
        let pytest_available = tokio::process::Command::new(&python)
            .args(["-c", "import pytest"])
            .current_dir(project_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false);

        if !pytest_available {
            println!("   {}", "📦 Instalando runner pytest...".cyan());
            let pip = Self::pip_path(project_dir);
            let install = tokio::process::Command::new(&pip)
                .args(["install", "pytest"])
                .current_dir(project_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| ForgeError::CommandNotFound {
                    command: format!("pip: {}", e),
                })?;
            if !install.status.success() {
                return Err(ForgeError::TaskFailed {
                    task_name: format!(
                        "instalar pytest: {}",
                        String::from_utf8_lossy(&install.stderr).trim()
                    ),
                    exit_code: install.status.code().unwrap_or(-1),
                }
                .into());
            }
        }

        println!("   {}", "🧪 Ejecutando tests Python...".cyan());

        let mut cmd = tokio::process::Command::new(&python);
        cmd.args(["-m", "pytest", "tests/"]) // Opcional pero recomendada
            .current_dir(project_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        match cmd.status().await {
            Ok(status) if status.success() => {
                println!("   {}", "✅ Todos los tests pasaron exitosamente!".green());
            }
            Ok(status) => {
                return Err(ForgeError::TaskFailed {
                    task_name: "python test".to_string(),
                    exit_code: status.code().unwrap_or(-1),
                }
                .into());
            }
            Err(e) => {
                return Err(ForgeError::CommandNotFound {
                    command: format!("python: {}", e),
                }
                .into());
            }
        }

        Ok(())
    }

    /// Encuentra el ejecutable de Python en el sistema.
    async fn find_python() -> ForgeResult<String> {
        // Intentar python3 primero, luego python
        for cmd in &["python3", "python", "py"] {
            let result = tokio::process::Command::new(cmd)
                .arg("--version")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await;

            if let Ok(output) = result {
                if output.status.success() {
                    return Ok(cmd.to_string());
                }
            }
        }

        Err(ForgeError::CommandNotFound {
            command: "python/python3".to_string(),
        }
        .into())
    }

    /// Ruta al Python del entorno virtual.
    fn python_path(project_dir: &Path) -> String {
        let venv = project_dir.join(".forge").join("venv");
        if cfg!(target_os = "windows") {
            venv.join("Scripts").join("python.exe")
        } else {
            venv.join("bin").join("python")
        }
        .to_string_lossy()
        .to_string()
    }

    /// Ruta al pip del entorno virtual.
    fn pip_path(project_dir: &Path) -> String {
        let venv = project_dir.join(".forge").join("venv");
        if cfg!(target_os = "windows") {
            venv.join("Scripts").join("pip.exe")
        } else {
            venv.join("bin").join("pip")
        }
        .to_string_lossy()
        .to_string()
    }
}
