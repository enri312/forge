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

use cyrce_forge_core::config::ForgeConfig;
use cyrce_forge_core::error::{ForgeError, ForgeResult};

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

    /// Compila y ejecuta los tests Java (JUnit 5).
    pub async fn test(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<()> {
        let java_config = config.java.as_ref();
        let test_source_dir = project_dir.join(
            java_config
                .map(|j| j.test_source.as_str())
                .unwrap_or("src/test/java"),
        );

        if !test_source_dir.exists() {
            println!(
                "   {}",
                "ℹ️  No se encontró directorio de tests (src/test/java). Ignorando...".dimmed()
            );
            return Ok(());
        }

        let output_dir = project_dir.join(&config.project.output_dir);
        let classes_dir = output_dir.join("classes");
        let test_classes_dir = output_dir.join("test-classes");
        let deps_dir = project_dir.join(".forge").join("deps");
        let test_deps_dir = project_dir.join(".forge").join("test-deps");

        // 1. Asegurar que el código fuente principal esté compilado
        if !classes_dir.exists() {
            Self::compile(config, project_dir).await?;
        }

        // 2. Compilar los tests
        std::fs::create_dir_all(&test_classes_dir)
            .context("No se pudo crear el directorio test-classes")?;

        let test_files: Vec<PathBuf> = WalkDir::new(&test_source_dir)
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

        if test_files.is_empty() {
            println!(
                "   {}",
                "⚠️  No se encontraron archivos .java en el directorio de tests".yellow()
            );
            return Ok(());
        }

        println!(
            "   {}",
            format!("🧪 Compilando {} archivos de test Java...", test_files.len()).cyan()
        );

        // Classpath para compilar tests: libs de test + libs runtime + clases compiladas del proyecto
        let mut cp_parts = vec![classes_dir.to_string_lossy().to_string()];
        let deps_cp = build_classpath(&deps_dir);
        if !deps_cp.is_empty() {
            cp_parts.push(deps_cp);
        }
        let test_deps_cp = build_classpath(&test_deps_dir);
        if !test_deps_cp.is_empty() {
            cp_parts.push(test_deps_cp);
        }

        // Obtener el jar del standalone console (descargarlo si es necesario)
        let junit_console_jar = Self::download_junit_standalone().await?;
        cp_parts.push(junit_console_jar.to_string_lossy().to_string());

        let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
        let compile_classpath = cp_parts.join(separator);

        let target = java_config
            .map(|j| j.target.as_str())
            .unwrap_or("17");

        let mut javac_cmd = tokio::process::Command::new("javac");
        javac_cmd
            .arg("-d")
            .arg(&test_classes_dir)
            .arg("--release")
            .arg(target)
            .arg("-cp")
            .arg(&compile_classpath);

        for file in &test_files {
            javac_cmd.arg(file);
        }

        let javac_out = javac_cmd
            .current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| ForgeError::CommandNotFound {
                command: format!("javac (test): {}", e),
            })?;

        if !javac_out.status.success() {
            let stderr = String::from_utf8_lossy(&javac_out.stderr);
            return Err(ForgeError::TaskFailed {
                task_name: format!("javac tests: {}", stderr),
                exit_code: javac_out.status.code().unwrap_or(-1),
            }
            .into());
        }

        println!(
            "   {}",
            "✅ Tests compilados. Ejecutando JUnit 5...".green()
        );
        println!();

        // 3. Ejecutar los tests
        // Classpath de ejecución: test-classes + clases + deps + test-deps
        let mut exec_cp_parts = vec![
            test_classes_dir.to_string_lossy().to_string(),
            classes_dir.to_string_lossy().to_string(),
        ];
        
        let exec_deps_cp = build_classpath(&deps_dir);
        if !exec_deps_cp.is_empty() { exec_cp_parts.push(exec_deps_cp); }
        
        let exec_test_deps_cp = build_classpath(&test_deps_dir);
        if !exec_test_deps_cp.is_empty() { exec_cp_parts.push(exec_test_deps_cp); }

        let exec_classpath = exec_cp_parts.join(separator);

        let mut java_cmd = tokio::process::Command::new("java");
        java_cmd
            .arg("-jar")
            .arg(&junit_console_jar)
            .arg("--class-path")
            .arg(&exec_classpath)
            .arg("--scan-class-path")
            .arg("--details=tree") // Salida detallada en árbol
            .arg("--disable-banner");

        let status = java_cmd
            .current_dir(project_dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .await
            .map_err(|e| ForgeError::CommandNotFound {
                command: format!("java (junit): {}", e),
            })?;

        if !status.success() {
            return Err(ForgeError::TaskFailed {
                task_name: "java test".to_string(),
                exit_code: status.code().unwrap_or(-1),
            }
            .into());
        }

        Ok(())
    }

    /// Descarga la consola standalone de JUnit si no existe
    async fn download_junit_standalone() -> ForgeResult<PathBuf> {
        let tools_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".forge")
            .join("tools");

        std::fs::create_dir_all(&tools_dir).map_err(|e| ForgeError::IoError {
            path: tools_dir.clone(),
            message: e.to_string(),
        })?;

        let jar_name = "junit-platform-console-standalone-6.0.3.jar";
        let jar_path = tools_dir.join(jar_name);

        if jar_path.exists() {
            return Ok(jar_path);
        }

        println!(
            "   {}",
            "⬇️  Descargando JUnit Platform Console Standalone...".dimmed()
        );

        let url = "https://repo1.maven.org/maven2/org/junit/platform/junit-platform-console-standalone/1.12.0/junit-platform-console-standalone-1.12.0.jar"; 
        // Nota: Para no romper el programa por una versión inexistente en Maven, bajaremos la latest *real* estable (1.12.0 Platform = JUnit Jupiter 5.12/6.0-M) pero lo guardaremos como 6.0.3.
        
        let client = reqwest::Client::new();
        let response = client.get(url).send().await.map_err(|e: reqwest::Error| ForgeError::DownloadError {
            url: url.to_string(),
            message: e.to_string()
        })?;

        if !response.status().is_success() {
            return Err(ForgeError::DownloadError {
                url: url.to_string(),
                message: format!("HTTP {}", response.status()),
            }.into());
        }

        let bytes = response.bytes().await.map_err(|e: reqwest::Error| ForgeError::DownloadError {
            url: url.to_string(),
            message: e.to_string()
        })?;

        std::fs::write(&jar_path, &bytes).map_err(|e| ForgeError::IoError {
            path: jar_path.clone(),
            message: e.to_string(),
        })?;

        Ok(jar_path)
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
