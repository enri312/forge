// =============================================================================
// 🔥 FORGE — Módulos de Lenguaje: Kotlin
// =============================================================================
// Compilación y ejecución de proyectos Kotlin usando kotlinc.
// =============================================================================

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::Context;
use colored::Colorize;
use walkdir::WalkDir;

use forge_core::config::ForgeConfig;
use forge_core::error::{ForgeError, ForgeResult};

/// Módulo de compilación Kotlin.
pub struct KotlinModule;

impl KotlinModule {
    /// Compila el proyecto Kotlin.
    pub async fn compile(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        let kotlin_config = config.kotlin.as_ref();
        let source_dir = project_dir.join(
            kotlin_config
                .map(|k| k.source.as_str())
                .unwrap_or("src/main/kotlin"),
        );
        let output_dir = project_dir.join(&config.project.output_dir).join("classes");
        let deps_dir = project_dir.join(".forge").join("deps");

        // Verificar que exista el directorio fuente
        if !source_dir.exists() {
            return Err(ForgeError::IoError {
                path: source_dir,
                message: "Directorio fuente no existe. ¿Olvidaste crear tus archivos .kt?"
                    .to_string(),
            }
            .into());
        }

        // Crear directorio de salida
        std::fs::create_dir_all(&output_dir).context("No se pudo crear el directorio de salida")?;

        // Encontrar todos los archivos .kt
        let kt_files: Vec<PathBuf> = WalkDir::new(&source_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "kt")
                    .unwrap_or(false)
            })
            .map(|e| e.path().to_path_buf())
            .collect();

        if kt_files.is_empty() {
            println!(
                "   {}",
                "⚠️  No se encontraron archivos .kt para compilar".yellow()
            );
            return Ok(());
        }

        println!(
            "   {}",
            format!("🟣 Compilando {} archivos Kotlin...", kt_files.len()).cyan()
        );

        // Construir classpath con dependencias
        let classpath = build_kotlin_classpath(&deps_dir);

        let jvm_target = kotlin_config
            .map(|k| k.jvm_target.as_str())
            .unwrap_or("17");

        let mut cmd = tokio::process::Command::new("kotlinc");

        cmd.arg("-d").arg(&output_dir);
        cmd.arg("-jvm-target").arg(jvm_target);

        // Agregar classpath si hay dependencias
        if !classpath.is_empty() {
            cmd.arg("-cp").arg(&classpath);
        }

        // Agregar archivos fuente
        for file in &kt_files {
            cmd.arg(file);
        }

        cmd.current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ForgeError::CommandNotFound {
                    command: "kotlinc".to_string(),
                }
            } else {
                ForgeError::IoError {
                    path: project_dir.to_path_buf(),
                    message: format!("Error al ejecutar kotlinc: {}", e),
                }
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("{}", stderr);
            return Err(ForgeError::TaskFailed {
                task_name: "kotlinc".to_string(),
                exit_code: output.status.code().unwrap_or(-1),
            }
            .into());
        }

        println!(
            "   {}",
            format!("✅ {} archivos Kotlin compilados exitosamente", kt_files.len()).green()
        );

        Ok(())
    }

    /// Empaqueta en un JAR ejecutable.
    pub async fn package(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<PathBuf> {
        let output_dir = project_dir.join(&config.project.output_dir);
        let classes_dir = output_dir.join("classes");
        let jar_path = output_dir.join(format!("{}.jar", config.project.name));

        if !classes_dir.exists() {
            return Err(ForgeError::IoError {
                path: classes_dir,
                message: "No hay clases compiladas. Ejecuta 'forge build' primero.".to_string(),
            }
            .into());
        }

        println!("   {}", "📦 Empaquetando JAR de Kotlin...".cyan());

        let mut cmd = tokio::process::Command::new("jar");
        cmd.arg("cf").arg(&jar_path);

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
                task_name: format!("jar (kotlin): {}", stderr),
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

    /// Ejecuta el proyecto Kotlin.
    pub async fn run(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        let main_class = config
            .main_entry()
            .ok_or_else(|| ForgeError::ConfigMissingField {
                field: "kotlin.main-class".to_string(),
            })?;

        let output_dir = project_dir.join(&config.project.output_dir);
        let classes_dir = output_dir.join("classes");
        let deps_dir = project_dir.join(".forge").join("deps");

        let mut cp_parts: Vec<String> = vec![classes_dir.to_string_lossy().to_string()];
        let deps_cp = build_kotlin_classpath(&deps_dir);
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
                task_name: "kotlin (java)".to_string(),
                exit_code: status.code().unwrap_or(-1),
            }
            .into());
        }

        Ok(())
    }
}

/// Construye classpath con JARs de dependencias.
fn build_kotlin_classpath(deps_dir: &Path) -> String {
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
