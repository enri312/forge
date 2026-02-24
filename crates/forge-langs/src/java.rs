// =============================================================================
// 🔥 FORGE — Módulos de Lenguaje: Java
// =============================================================================
// Compilación y ejecución de proyectos Java usando javac y java.
// Soporta compilación incremental, classpath y empaquetado JAR.
// =============================================================================

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::Context;
use colored::Colorize;
use walkdir::WalkDir;

use forge_core::config::ForgeConfig;
use forge_core::error::{ForgeError, ForgeResult};

/// Módulo de compilación Java.
pub struct JavaModule;

impl JavaModule {
    /// Compila el proyecto Java.
    pub async fn compile(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        let java_config = config.java.as_ref();
        let source_dir = project_dir.join(
            java_config
                .map(|j| j.source.as_str())
                .unwrap_or("src/main/java"),
        );
        let output_dir = project_dir.join(&config.project.output_dir).join("classes");
        let deps_dir = project_dir.join(".forge").join("deps");

        // Verificar que exista el directorio fuente
        if !source_dir.exists() {
            return Err(ForgeError::IoError {
                path: source_dir,
                message: "Directorio fuente no existe. ¿Olvidaste crear tus archivos .java?".to_string(),
            }
            .into());
        }

        // Crear directorio de salida
        std::fs::create_dir_all(&output_dir).context("No se pudo crear el directorio de salida")?;

        // Encontrar todos los archivos .java
        let java_files: Vec<PathBuf> = WalkDir::new(&source_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "java")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        if java_files.is_empty() {
            println!(
                "   {}",
                "⚠️  No se encontraron archivos .java para compilar".yellow()
            );
            return Ok(());
        }

        println!(
            "   {}",
            format!("☕ Compilando {} archivos Java...", java_files.len()).cyan()
        );

        // Construir classpath con dependencias descargadas
        let classpath = build_classpath(&deps_dir);

        // Construir comando javac
        let target = java_config
            .map(|j| j.target.as_str())
            .unwrap_or("17");

        let mut cmd = tokio::process::Command::new("javac");

        // Opciones de compilación
        cmd.arg("-d")
            .arg(&output_dir)
            .arg("--release")
            .arg(target);

        // Agregar classpath si hay dependencias
        if !classpath.is_empty() {
            cmd.arg("-cp").arg(&classpath);
        }

        // Agregar archivos fuente
        for file in &java_files {
            cmd.arg(file);
        }

        cmd.current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ForgeError::CommandNotFound {
                    command: "javac".to_string(),
                }
            } else {
                ForgeError::IoError {
                    path: project_dir.to_path_buf(),
                    message: format!("Error al ejecutar javac: {}", e),
                }
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("{}", stderr);
            return Err(ForgeError::TaskFailed {
                task_name: "javac".to_string(),
                exit_code: output.status.code().unwrap_or(-1),
            }
            .into());
        }

        println!(
            "   {}",
            format!("✅ {} archivos compilados exitosamente", java_files.len()).green()
        );

        Ok(())
    }

    /// Empaqueta las clases compiladas en un JAR.
    pub async fn package(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<PathBuf> {
        let output_dir = project_dir.join(&config.project.output_dir);
        let classes_dir = output_dir.join("classes");
        let jar_path = output_dir.join(format!("{}.jar", config.project.name));

        // Verificar que existan las clases compiladas
        if !classes_dir.exists() {
            return Err(ForgeError::IoError {
                path: classes_dir,
                message: "No hay clases compiladas. Ejecuta 'forge build' primero.".to_string(),
            }
            .into());
        }

        println!("   {}", "📦 Empaquetando JAR...".cyan());

        let mut cmd = tokio::process::Command::new("jar");
        cmd.arg("cf").arg(&jar_path);

        // Agregar manifiesto con Main-Class si está definido
        if let Some(main_class) = config.main_entry() {
            let manifest_dir = output_dir.join("META-INF");
            std::fs::create_dir_all(&manifest_dir)?;
            let manifest_path = manifest_dir.join("MANIFEST.MF");
            std::fs::write(
                &manifest_path,
                format!(
                    "Manifest-Version: 1.0\nMain-Class: {}\nBuilt-By: FORGE\n",
                    main_class
                ),
            )?;
            cmd.arg("--manifest").arg(&manifest_path);
        }

        cmd.arg("-C").arg(&classes_dir).arg(".");

        cmd.current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| ForgeError::CommandNotFound {
            command: format!("jar: {}", e),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ForgeError::TaskFailed {
                task_name: format!("jar: {}", stderr),
                exit_code: output.status.code().unwrap_or(-1),
            }
            .into());
        }

        println!(
            "   {}",
            format!("📦 JAR creado: {}", jar_path.display()).green()
        );

        Ok(jar_path)
    }

    /// Ejecuta el proyecto Java.
    pub async fn run(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        let main_class = config
            .main_entry()
            .ok_or_else(|| ForgeError::ConfigMissingField {
                field: "java.main-class".to_string(),
            })?;

        let output_dir = project_dir.join(&config.project.output_dir);
        let classes_dir = output_dir.join("classes");
        let deps_dir = project_dir.join(".forge").join("deps");

        // Construir classpath: clases compiladas + dependencias
        let mut cp_parts: Vec<String> = vec![classes_dir.to_string_lossy().to_string()];
        let deps_cp = build_classpath(&deps_dir);
        if !deps_cp.is_empty() {
            cp_parts.push(deps_cp);
        }

        let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
        let classpath = cp_parts.join(separator);

        println!(
            "   {}",
            format!("🚀 Ejecutando {}...", main_class).cyan()
        );
        println!();

        let mut cmd = tokio::process::Command::new("java");
        cmd.arg("-cp")
            .arg(&classpath)
            .arg(&main_class)
            .current_dir(project_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().await.map_err(|e| ForgeError::CommandNotFound {
            command: format!("java: {}", e),
        })?;

        if !status.success() {
            return Err(ForgeError::TaskFailed {
                task_name: "java".to_string(),
                exit_code: status.code().unwrap_or(-1),
            }
            .into());
        }

        Ok(())
    }
}

/// Construye el classpath con todos los JARs en el directorio de dependencias.
fn build_classpath(deps_dir: &Path) -> String {
    if !deps_dir.exists() {
        return String::new();
    }

    let separator = if cfg!(target_os = "windows") { ";" } else { ":" };

    WalkDir::new(deps_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "jar")
                .unwrap_or(false)
        })
        .map(|e| e.path().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(separator)
}
