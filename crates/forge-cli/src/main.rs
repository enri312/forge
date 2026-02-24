// =============================================================================
// 🔥 FORGE — CLI: Punto de Entrada Principal
// =============================================================================
// Interfaz de línea de comandos del build system FORGE.
// Usa clap para parseo de argumentos con interfaz moderna y colorida.
// =============================================================================

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Context;
use clap::{CommandFactory, Parser, Subcommand};
use colored::Colorize;

use forge_core::cache::BuildCache;
use forge_core::config::ForgeConfig;

use forge_deps::maven::MavenResolver;
use forge_deps::pypi::PypiResolver;

use forge_langs::java::JavaModule;
use forge_langs::kotlin::KotlinModule;
use forge_langs::python::PythonModule;

/// 🔥 FORGE — Build system de nueva generación.
/// Rápido, simple y multi-lenguaje.
#[derive(Parser)]
#[command(
    name = "forge",
    version,
    about = "🔥 FORGE — Build system de nueva generación",
    long_about = "FORGE es un build system moderno escrito en Rust.\nSoporta Java, Kotlin y Python con compilación incremental,\nejecución paralela y una configuración simple en TOML.",
    after_help = "Ejemplos:\n  forge init java      Crear proyecto Java\n  forge build           Compilar el proyecto\n  forge run             Compilar y ejecutar\n  forge test            Ejecutar tests\n  forge clean           Limpiar artefactos\n\n🌐 https://github.com/enri312/forge"
)]
struct Cli {
    /// Comando a ejecutar
    #[command(subcommand)]
    command: Commands,

    /// Directorio del proyecto (por defecto: directorio actual)
    #[arg(short = 'p', long = "project-dir", global = true)]
    project_dir: Option<PathBuf>,

    /// Modo verboso (muestra más detalles)
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 🆕 Inicializar un nuevo proyecto FORGE
    Init {
        /// Lenguaje del proyecto: java, kotlin, python
        #[arg(default_value = "java")]
        lang: String,
    },

    /// 📁 Crear un nuevo proyecto en una carpeta nueva
    New {
        /// Nombre del proyecto (se crea como carpeta)
        name: String,

        /// Lenguaje del proyecto: java, kotlin, python
        #[arg(short, long, default_value = "java")]
        lang: String,
    },

    /// 🔨 Compilar el proyecto
    Build,

    /// 🚀 Compilar y ejecutar el proyecto
    Run,

    /// 🧪 Ejecutar tests
    Test,

    /// 🧹 Limpiar artefactos de build y caché
    Clean,

    /// 📦 Descargar y resolver dependencias
    Deps,

    /// ℹ️  Mostrar información del proyecto
    Info,

    /// 👁️ Vigilar cambios y recompilar automáticamente
    Watch,

    /// ⚙️ Ejecutar una tarea personalizada del forge.toml
    Task {
        /// Nombre de la tarea a ejecutar
        name: String,
    },

    /// 🐚 Generar autocompletado para tu shell
    Completions {
        /// Shell objetivo: bash, zsh, fish, powershell
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Inicializar logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .without_time()
        .init();

    let cli = Cli::parse();

    // Determinar directorio del proyecto (convertir a ruta absoluta)
    let project_dir = cli
        .project_dir
        .unwrap_or_else(|| std::env::current_dir().expect("No se puede obtener el directorio actual"));
    let project_dir = std::fs::canonicalize(&project_dir)
        .unwrap_or_else(|_| {
            // Si no existe aún (ej: forge init), usar la ruta tal cual
            if project_dir.is_relative() {
                std::env::current_dir()
                    .expect("No se puede obtener el directorio actual")
                    .join(&project_dir)
            } else {
                project_dir
            }
        });

    // Banner
    print_banner();

    // Ejecutar comando
    let start = Instant::now();
    let result = match cli.command {
        Commands::Init { lang } => cmd_init(&project_dir, &lang).await,
        Commands::New { name, lang } => cmd_new(&project_dir, &name, &lang).await,
        Commands::Build => cmd_build(&project_dir, cli.verbose).await,
        Commands::Run => cmd_run(&project_dir, cli.verbose).await,
        Commands::Test => cmd_test(&project_dir, cli.verbose).await,
        Commands::Clean => cmd_clean(&project_dir).await,
        Commands::Deps => cmd_deps(&project_dir).await,
        Commands::Info => cmd_info(&project_dir).await,
        Commands::Watch => cmd_watch(&project_dir).await,
        Commands::Task { name } => cmd_task(&project_dir, &name).await,
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "forge", &mut std::io::stdout());
            Ok(())
        }
    };

    if let Err(e) = &result {
        eprintln!("\n{} {}", "❌ Error:".red().bold(), e);
        eprintln!(
            "{}",
            "   Usa 'forge --help' para ver los comandos disponibles.".dimmed()
        );
        std::process::exit(1);
    }

    let elapsed = start.elapsed();
    if elapsed.as_millis() > 100 {
        println!(
            "{}",
            format!("⏱️  Completado en {:.2}s", elapsed.as_secs_f64()).dimmed()
        );
    }

    Ok(())
}

/// Muestra el banner de FORGE.
fn print_banner() {
    println!(
        "{}",
        r#"
   ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
   ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
   █████╗  ██║   ██║██████╔╝██║  ███╗█████╗  
   ██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝  
   ██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
   ╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
"#
        .cyan()
        .bold()
    );
}

/// Comando: forge init <lang>
async fn cmd_init(project_dir: &PathBuf, lang: &str) -> anyhow::Result<()> {
    println!(
        "{}",
        format!("🆕 Inicializando proyecto {} en {:?}...", lang, project_dir).bold()
    );

    let forge_toml = project_dir.join("forge.toml");

    if forge_toml.exists() {
        println!(
            "{}",
            "⚠️  Ya existe un forge.toml en este directorio".yellow()
        );
        return Ok(());
    }

    // Crear directorio del proyecto si no existe
    std::fs::create_dir_all(project_dir)?;

    // Generar forge.toml desde plantilla
    let template = ForgeConfig::generate_template(lang)?;
    std::fs::write(&forge_toml, &template)?;
    println!("   {} forge.toml", "✅ Creado:".green());

    // Crear estructura de directorios según el lenguaje
    let source_dir = match lang {
        "java" => "src/main/java",
        "kotlin" => "src/main/kotlin",
        "python" => "src",
        _ => "src",
    };

    let full_source_dir = project_dir.join(source_dir);
    std::fs::create_dir_all(&full_source_dir)?;
    println!(
        "   {} {}",
        "✅ Creado:".green(),
        source_dir
    );

    // Crear archivo de ejemplo
    create_example_file(lang, &full_source_dir)?;

    // Crear .gitignore
    let gitignore = project_dir.join(".gitignore");
    if !gitignore.exists() {
        std::fs::write(
            &gitignore,
            "# FORGE\nbuild/\n.forge/\n\n# IDE\n.idea/\n.vscode/\n*.iml\n\n# OS\n.DS_Store\nThumbs.db\n",
        )?;
        println!("   {} .gitignore", "✅ Creado:".green());
    }

    println!();
    println!(
        "{}",
        "🎉 ¡Proyecto inicializado! Próximos pasos:".green().bold()
    );
    println!("   1. Edita {} para configurar tu proyecto", "forge.toml".cyan());
    println!("   2. Escribe tu código en {}", source_dir.cyan());
    println!("   3. Ejecuta {} para compilar", "forge build".cyan());
    println!("   4. Ejecuta {} para correr tu programa", "forge run".cyan());
    println!();

    Ok(())
}

/// Crea un archivo de ejemplo según el lenguaje.
fn create_example_file(lang: &str, source_dir: &PathBuf) -> anyhow::Result<()> {
    match lang {
        "java" => {
            let file = source_dir.join("Main.java");
            if !file.exists() {
                std::fs::write(
                    &file,
                    r#"public class Main {
    public static void main(String[] args) {
        System.out.println("🔥 ¡Hola desde FORGE! — Proyecto Java");
        System.out.println("   Build system de nueva generación");
    }
}
"#,
                )?;
                println!("   {} Main.java (ejemplo)", "✅ Creado:".green());
            }
        }
        "kotlin" => {
            let file = source_dir.join("Main.kt");
            if !file.exists() {
                std::fs::write(
                    &file,
                    r#"fun main() {
    println("🔥 ¡Hola desde FORGE! — Proyecto Kotlin")
    println("   Build system de nueva generación")
}
"#,
                )?;
                println!("   {} Main.kt (ejemplo)", "✅ Creado:".green());
            }
        }
        "python" => {
            let file = source_dir.join("main.py");
            if !file.exists() {
                std::fs::write(
                    &file,
                    r#"#!/usr/bin/env python3
"""🔥 Proyecto de ejemplo FORGE — Python"""


def main():
    print("🔥 ¡Hola desde FORGE! — Proyecto Python")
    print("   Build system de nueva generación")


if __name__ == "__main__":
    main()
"#,
                )?;
                println!("   {} main.py (ejemplo)", "✅ Creado:".green());
            }
        }
        _ => {}
    }

    Ok(())
}

/// Comando: forge build
async fn cmd_build(project_dir: &PathBuf, _verbose: bool) -> anyhow::Result<()> {
    let config = ForgeConfig::load(project_dir)?;

    // Verificar caché: ¿necesitamos recompilar?
    let source_dir = project_dir.join(config.source_dir());
    let extensions = forge_langs::extensions_for_lang(&config.project.lang);
    let mut cache = BuildCache::load(project_dir)?;

    if !cache.has_changes(&source_dir, extensions)? {
        println!(
            "{}",
            "⚡ Sin cambios detectados — usando caché".dimmed()
        );
        return Ok(());
    }

    // Resolver dependencias si hay
    if !config.dependencies.is_empty() {
        resolve_dependencies(&config, project_dir).await?;
    }

    // Compilar según el lenguaje
    match config.project.lang.as_str() {
        "java" => JavaModule::compile(&config, project_dir).await?,
        "kotlin" => KotlinModule::compile(&config, project_dir).await?,
        "python" => PythonModule::compile(&config, project_dir).await?,
        _ => {}
    }

    // Actualizar caché
    cache.update_hashes(&source_dir, extensions)?;
    cache.save(project_dir)?;

    Ok(())
}

/// Comando: forge run
async fn cmd_run(project_dir: &PathBuf, verbose: bool) -> anyhow::Result<()> {
    // Primero compilar
    cmd_build(project_dir, verbose).await?;

    let config = ForgeConfig::load(project_dir)?;

    // Ejecutar según el lenguaje
    match config.project.lang.as_str() {
        "java" => JavaModule::run(&config, project_dir).await?,
        "kotlin" => KotlinModule::run(&config, project_dir).await?,
        "python" => PythonModule::run(&config, project_dir).await?,
        _ => {}
    }

    Ok(())
}

/// Comando: forge test
async fn cmd_test(project_dir: &PathBuf, verbose: bool) -> anyhow::Result<()> {
    let config = ForgeConfig::load(project_dir)?;

    println!("{}", "🧪 Ejecutando tests...".bold());

    match config.project.lang.as_str() {
        "java" => {
            // Compilar primero
            cmd_build(project_dir, verbose).await?;
            println!("   {}", "⚠️  Tests Java: Próximamente (JUnit runner)".yellow());
        }
        "kotlin" => {
            cmd_build(project_dir, verbose).await?;
            println!(
                "   {}",
                "⚠️  Tests Kotlin: Próximamente (JUnit runner)".yellow()
            );
        }
        "python" => PythonModule::test(&config, project_dir).await?,
        _ => {}
    }

    Ok(())
}

/// Comando: forge clean
async fn cmd_clean(project_dir: &PathBuf) -> anyhow::Result<()> {
    println!("{}", "🧹 Limpiando artefactos...".bold());

    let build_dir = project_dir.join("build");
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir)?;
        println!("   {} build/", "🗑️  Eliminado:".green());
    }

    BuildCache::clean(project_dir)?;
    println!("   {} .forge/", "🗑️  Eliminado:".green());

    println!("\n{}", "✅ Limpieza completada".green().bold());
    Ok(())
}

/// Comando: forge deps
async fn cmd_deps(project_dir: &PathBuf) -> anyhow::Result<()> {
    let config = ForgeConfig::load(project_dir)?;

    if config.dependencies.is_empty() {
        println!("{}", "📦 No hay dependencias definidas en forge.toml".dimmed());
        return Ok(());
    }

    resolve_dependencies(&config, project_dir).await
}

/// Resuelve dependencias según el lenguaje.
async fn resolve_dependencies(config: &ForgeConfig, project_dir: &PathBuf) -> anyhow::Result<()> {
    match config.project.lang.as_str() {
        "java" | "kotlin" => {
            let mut resolver = MavenResolver::new(project_dir);
            resolver.resolve_all(&config.dependencies).await?;
        }
        "python" => {
            let resolver = PypiResolver::new();
            resolver.verify_all(&config.dependencies).await?;
        }
        _ => {}
    }

    Ok(())
}

/// Comando: forge info
async fn cmd_info(project_dir: &PathBuf) -> anyhow::Result<()> {
    let config = ForgeConfig::load(project_dir)
        .context("No se encontró forge.toml. ¿Estás en un proyecto FORGE?")?;

    println!("{}", "ℹ️  Información del Proyecto".bold());
    println!("   {} {}", "Nombre:".cyan(), config.project.name);
    println!("   {} {}", "Versión:".cyan(), config.project.version);
    println!("   {} {}", "Lenguaje:".cyan(), config.project.lang);
    println!("   {} {}", "Fuente:".cyan(), config.source_dir());
    println!("   {} {}", "Salida:".cyan(), config.project.output_dir);

    if let Some(main) = config.main_entry() {
        println!("   {} {}", "Entrada:".cyan(), main);
    }

    if !config.dependencies.is_empty() {
        println!("\n   {} ({}):", "Dependencias".cyan(), config.dependencies.len());
        for (name, version) in &config.dependencies {
            println!("      • {} = {}", name, version);
        }
    }

    if !config.tasks.is_empty() {
        println!("\n   {} ({}):", "Tareas".cyan(), config.tasks.len());
        for (name, task) in &config.tasks {
            println!("      • {} — {}", name, task.command);
        }
    }

    // Mostrar herramientas del sistema
    println!("\n{}", "🔧 Herramientas del Sistema".bold());
    print_tool_version("Rust", "rustc", &["--version"]);
    match config.project.lang.as_str() {
        "java" => {
            print_tool_version("Java", "javac", &["--version"]);
            print_tool_version("JVM", "java", &["--version"]);
        }
        "kotlin" => {
            print_tool_version("Kotlin", "kotlinc", &["-version"]);
            print_tool_version("JVM", "java", &["--version"]);
        }
        "python" => {
            print_tool_version("Python", "python", &["--version"]);
            print_tool_version("Pip", "pip", &["--version"]);
        }
        _ => {}
    }

    println!();
    Ok(())
}

/// Imprime la versión de una herramienta del sistema.
fn print_tool_version(name: &str, cmd: &str, args: &[&str]) {
    match std::process::Command::new(cmd).args(args).output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            let version = version.trim();
            if version.is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let version = stderr.lines().next().unwrap_or("").trim();
                println!("   {} {}", format!("{}:", name).cyan(), version);
            } else {
                let first_line = version.lines().next().unwrap_or(version);
                println!("   {} {}", format!("{}:", name).cyan(), first_line);
            }
        }
        Err(_) => {
            println!(
                "   {} {}",
                format!("{}:", name).cyan(),
                "No encontrado ❌".red()
            );
        }
    }
}

/// Comando: forge new <nombre>
async fn cmd_new(parent_dir: &PathBuf, name: &str, lang: &str) -> anyhow::Result<()> {
    let project_dir = parent_dir.join(name);

    if project_dir.exists() {
        return Err(anyhow::anyhow!(
            "El directorio '{}' ya existe",
            project_dir.display()
        ));
    }

    println!(
        "{}",
        format!("📁 Creando proyecto '{}' ({})...", name, lang).bold()
    );

    std::fs::create_dir_all(&project_dir)?;
    cmd_init(&project_dir, lang).await?;

    println!(
        "\n{}",
        format!("💡 Para empezar: cd {} && forge build", name)
            .cyan()
            .bold()
    );

    Ok(())
}

/// Comando: forge watch
async fn cmd_watch(project_dir: &PathBuf) -> anyhow::Result<()> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
    use std::sync::mpsc;

    let config = ForgeConfig::load(project_dir)?;
    let source_dir = project_dir.join(config.source_dir());

    if !source_dir.exists() {
        return Err(anyhow::anyhow!(
            "Directorio de código fuente no encontrado: {}",
            source_dir.display()
        ));
    }

    println!(
        "{}",
        format!(
            "👁️ Vigilando cambios en {} (Ctrl+C para detener)...",
            config.source_dir()
        )
        .cyan()
        .bold()
    );

    // Build inicial
    println!("{}", "\n── Build inicial ──".dimmed());
    if let Err(e) = cmd_build(project_dir, false).await {
        eprintln!("   {} {}", "⚠️  Error en build:".yellow(), e);
    }

    // Configurar watcher
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(&source_dir, RecursiveMode::Recursive)?;

    // Configurar Ctrl+C
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;

    println!(
        "{}",
        "✅ Watcher activo — editá tu código y FORGE recompilará automáticamente\n".green()
    );

    let extensions = forge_langs::extensions_for_lang(&config.project.lang);

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        match rx.recv_timeout(std::time::Duration::from_millis(500)) {
            Ok(Ok(event)) => {
                // Solo recompilar si son archivos relevantes
                let is_relevant = event.paths.iter().any(|p| {
                    if let Some(ext) = p.extension() {
                        extensions.iter().any(|e| ext == *e)
                    } else {
                        false
                    }
                });

                if is_relevant && matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    let changed_files: Vec<String> = event
                        .paths
                        .iter()
                        .filter_map(|p| p.file_name())
                        .map(|f| f.to_string_lossy().to_string())
                        .collect();

                    println!(
                        "\n{}",
                        format!(
                            "🔄 Cambios detectados: {} — Recompilando...",
                            changed_files.join(", ")
                        )
                        .yellow()
                        .bold()
                    );

                    let start = Instant::now();
                    match cmd_build(project_dir, false).await {
                        Ok(_) => {
                            println!(
                                "{}",
                                format!(
                                    "✅ Build exitoso en {:.2}s — Esperando más cambios...\n",
                                    start.elapsed().as_secs_f64()
                                )
                                .green()
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "{}",
                                format!("❌ Error: {} — Corrige y guarda de nuevo\n", e).red()
                            );
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("   {} {}", "⚠️  Error del watcher:".yellow(), e);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    println!("\n{}", "👋 Watch mode detenido".dimmed());
    Ok(())
}

/// Comando: forge task <nombre>
async fn cmd_task(project_dir: &PathBuf, task_name: &str) -> anyhow::Result<()> {
    let config = ForgeConfig::load(project_dir)?;

    let task = config
        .tasks
        .get(task_name)
        .ok_or_else(|| {
            let available: Vec<&String> = config.tasks.keys().collect();
            if available.is_empty() {
                anyhow::anyhow!(
                    "No hay tareas definidas en forge.toml. Agrega una sección [tasks.{}]",
                    task_name
                )
            } else {
                anyhow::anyhow!(
                    "Tarea '{}' no encontrada. Disponibles: {}",
                    task_name,
                    available
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        })?;

    println!(
        "{}",
        format!("⚙️  Ejecutando tarea: {}", task_name).bold()
    );
    println!("   {} {}", "Comando:".dimmed(), task.command);

    // Ejecutar el comando
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", &task.command])
            .current_dir(project_dir)
            .output()?
    } else {
        std::process::Command::new("sh")
            .args(["-c", &task.command])
            .current_dir(project_dir)
            .output()?
    };

    // Mostrar salida
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.is_empty() {
        println!("\n{}", stdout.trim());
    }
    if !stderr.is_empty() {
        eprintln!("{}", stderr.trim());
    }

    if output.status.success() {
        println!(
            "\n{}",
            format!("✅ Tarea '{}' completada", task_name).green().bold()
        );
    } else {
        return Err(anyhow::anyhow!(
            "La tarea '{}' falló con código {}",
            task_name,
            output.status.code().unwrap_or(-1)
        ));
    }

    Ok(())
}

