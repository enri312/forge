// =============================================================================
// 🔥 FORGE — CLI: Punto de Entrada Principal
// =============================================================================
// Interfaz de línea de comandos del build system FORGE.
// Usa clap para parseo de argumentos con interfaz moderna y colorida.
// =============================================================================

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
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
    after_help = "Ejemplos:\n  forge init java      Crear proyecto Java\n  forge build           Compilar el proyecto\n  forge run             Compilar y ejecutar\n  forge test            Ejecutar tests\n  forge clean           Limpiar artefactos\n\n🌐 https://github.com/forge-build/forge"
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
    let result = match cli.command {
        Commands::Init { lang } => cmd_init(&project_dir, &lang).await,
        Commands::Build => cmd_build(&project_dir, cli.verbose).await,
        Commands::Run => cmd_run(&project_dir, cli.verbose).await,
        Commands::Test => cmd_test(&project_dir, cli.verbose).await,
        Commands::Clean => cmd_clean(&project_dir).await,
        Commands::Deps => cmd_deps(&project_dir).await,
        Commands::Info => cmd_info(&project_dir).await,
    };

    if let Err(e) = &result {
        eprintln!("\n{} {}", "❌ Error:".red().bold(), e);
        eprintln!(
            "{}",
            "   Usa 'forge --help' para ver los comandos disponibles.".dimmed()
        );
        std::process::exit(1);
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

    println!();
    Ok(())
}
