// =============================================================================
// 🔥 FORGE — CLI: Punto de Entrada Principal
// =============================================================================
// Interfaz de línea de comandos del build system FORGE.
// Usa clap para parseo de argumentos con interfaz moderna y colorida.
// =============================================================================

mod ide;
mod hooks;
mod fmt;
mod lint;
mod add;
mod upgrade;
mod tree;
mod dashboard;

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Context;
use clap::{CommandFactory, Parser, Subcommand};
use colored::Colorize;

use cyrce_forge_core::cache::BuildCache;
use cyrce_forge_core::config::ForgeConfig;

use cyrce_forge_deps::maven::MavenResolver;
use cyrce_forge_deps::pypi::PypiResolver;

use cyrce_forge_langs::java::JavaModule;
use cyrce_forge_langs::kotlin::KotlinModule;
use cyrce_forge_langs::python::PythonModule;

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
    Build {
        /// Compilar en modo optimizado para producción
        #[arg(long)]
        release: bool,
        
        /// Lanzar el dashboard web interactivo durante la compilación
        #[arg(short, long)]
        dashboard: bool,
    },

    /// 🚀 Compilar y ejecutar el proyecto
    Run,

    /// 🧪 Ejecutar tests
    Test,

    /// 🧹 Limpiar artefactos de build y caché
    Clean,

    /// 📦 Descargar y resolver dependencias
    Deps,

    /// ➕ Añadir una dependencia a forge.toml
    Add {
        /// Coordenada u offset de paquete (ej: com.google.gson:gson:2.11.0 o flask)
        dep: String,
        /// Añadir como dependencia de test
        #[arg(short, long)]
        test: bool,
    },

    /// ⬆️  Actualizar dependencias a versiones más recientes (beta/PyPI only por ahora)
    Upgrade,

    /// 🌲 Visualizar el árbol de dependencias resueltas
    Tree,

    /// ℹ️  Mostrar información del proyecto
    Info,

    /// 👁️ Vigilar cambios y recompilar automáticamente
    Watch {
        /// Lanzar el dashboard web interactivo
        #[arg(short, long)]
        dashboard: bool,
    },

    /// ⚙️ Ejecutar una tarea personalizada del forge.toml
    Task {
        /// Nombre de la tarea a ejecutar
        name: String,
    },

    /// 🩺 Diagnosticar el sistema (verificar herramientas instaladas)
    Doctor,

    /// 📊 Mostrar estadísticas del proyecto (archivos, líneas, tamaño)
    Stats,

    /// ⏱️  Medir tiempo de compilación (benchmark)
    Bench,

    /// 📦 Empaquetar proyecto para distribución
    Package,

    /// 🐚 Generar autocompletado para tu shell
    Completions {
        /// Shell objetivo: bash, zsh, fish, powershell
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// 🛠️ Configurar integración con IDE (VS Code, IntelliJ)
    Ide {
        /// Editor objetivo: vscode, intellij
        target: String,
    },

    /// 🎨 Formatear código fuente (google-java-format, ktlint, black)
    Fmt,

    /// 🔍 Análisis estático del código (checkstyle, detekt, ruff)
    Lint,

    /// 🌐 Iniciar el Dashboard Web Interactivo
    Dashboard {
        /// Puerto para iniciar el servidor (por defecto: 3000)
        #[arg(long, default_value = "3000")]
        port: u16,
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
        Commands::Build { release, dashboard } => {
            if dashboard {
                let p = project_dir.clone();
                tokio::spawn(async move {
                    let _ = dashboard::cmd_dashboard(&p, 3000).await;
                });
                // Darle tiempo al servidor Axum para iniciar
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            let res = cmd_build(project_dir.clone(), cli.verbose, release).await;
            if dashboard {
                println!("\n{} {}", "🚀".cyan(), "Dashboard corriendo en segundo plano.".bold());
                println!("{}", "Presiona Ctrl+C para finalizar, o visita http://localhost:3000".dimmed());
                // Mantener el proceso vivo para que el dashboard siga funcionando
                let _ = tokio::signal::ctrl_c().await; 
            }
            res
        },
        Commands::Run => cmd_run(&project_dir, cli.verbose).await,
        Commands::Test => cmd_test(&project_dir, cli.verbose).await,
        Commands::Clean => cmd_clean(&project_dir).await,
        Commands::Deps => cmd_deps(&project_dir).await,
        Commands::Add { dep, test } => add::cmd_add(&project_dir, &dep, test).await,
        Commands::Upgrade => upgrade::cmd_upgrade(&project_dir).await,
        Commands::Tree => tree::cmd_tree(&project_dir).await,
        Commands::Info => cmd_info(&project_dir).await,
        Commands::Watch { dashboard } => cmd_watch(&project_dir, dashboard).await,
        Commands::Task { name } => cmd_task(&project_dir, &name).await,
        Commands::Doctor => cmd_doctor().await,
        Commands::Stats => cmd_stats(&project_dir).await,
        Commands::Bench => cmd_bench(&project_dir, cli.verbose).await,
        Commands::Package => cmd_package(&project_dir).await,
        Commands::Ide { target } => ide::cmd_ide(&project_dir, &target).await,
        Commands::Fmt => fmt::cmd_fmt(&project_dir).await,
        Commands::Lint => lint::cmd_lint(&project_dir).await,
        Commands::Dashboard { port } => dashboard::cmd_dashboard(&project_dir, port).await,
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            clap_complete::generate(shell, &mut cmd, "forge", &mut std::io::stdout());
            Ok(())
        }
    };

    if let Err(e) = &result {
        eprintln!("\n{} {}", "❌ Error:".red().bold(), e);

        // Intentar extraer sugerencia contextual si es un ForgeError
        if let Some(forge_err) = e.downcast_ref::<cyrce_forge_core::error::ForgeError>() {
            eprintln!("{}", forge_err.suggestion().yellow());
        } else {
            eprintln!(
                "{}",
                "   Usa 'forge --help' para ver los comandos disponibles.".dimmed()
            );
        }

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

    let test_dir = match lang {
        "java" => "src/test/java",
        "kotlin" => "src/test/kotlin",
        "python" => "tests",
        _ => "tests",
    };

    let full_source_dir = project_dir.join(source_dir);
    std::fs::create_dir_all(&full_source_dir)?;
    println!(
        "   {} {}",
        "✅ Creado:".green(),
        source_dir
    );

    let full_test_dir = project_dir.join(test_dir);
    std::fs::create_dir_all(&full_test_dir)?;
    println!(
        "   {} {}",
        "✅ Creado:".green(),
        test_dir
    );

    // Crear archivo de ejemplo y test
    create_example_file(lang, &full_source_dir)?;
    create_test_file(lang, &full_test_dir)?;

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
    println!("   3. Ejecuta {} para compilar tu programa", "forge build".cyan());
    println!("   4. Ejecuta {} para validar los tests", "forge test".cyan());
    println!("   5. Ejecuta {} para correr tu programa", "forge run".cyan());
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

/// Crea un archivo de test de ejemplo según el lenguaje.
fn create_test_file(lang: &str, test_dir: &PathBuf) -> anyhow::Result<()> {
    match lang {
        "java" => {
            let file = test_dir.join("MainTest.java");
            if !file.exists() {
                std::fs::write(
                    &file,
                    r#"import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class MainTest {
    @Test
    void forgeTestWorks() {
        assertEquals(2, 1 + 1, "FORGE Test Runner debería funcionar correctamente");
    }
}
"#,
                )?;
                println!("   {} MainTest.java (ejemplo de test)", "✅ Creado:".green());
            }
        }
        "kotlin" => {
            let file = test_dir.join("MainTest.kt");
            if !file.exists() {
                std::fs::write(
                    &file,
                    r#"import org.junit.jupiter.api.Test
import org.junit.jupiter.api.Assertions.assertEquals

class MainTest {
    @Test
    fun `forge test works`() {
        assertEquals(2, 1 + 1, "FORGE Test Runner debería funcionar correctamente")
    }
}
"#,
                )?;
                println!("   {} MainTest.kt (ejemplo de test)", "✅ Creado:".green());
            }
        }
        "python" => {
            let file = test_dir.join("test_main.py");
            if !file.exists() {
                std::fs::write(
                    &file,
                    r#"def test_forge_works():
    assert 1 + 1 == 2, "FORGE Test Runner debería funcionar correctamente"
"#,
                )?;
                println!("   {} test_main.py (ejemplo de test)", "✅ Creado:".green());
            }
        }
        _ => {}
    }

    Ok(())
}

/// Función auxiliar para romper el ciclo de recursión infinito en el compilador
/// y asegurar el type-bound `Send + 'static` al usar concurrencia.
fn cmd_build_boxed(
    project_dir: PathBuf,
    verbose: bool,
    release: bool,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'static>> {
    Box::pin(async move { cmd_build(project_dir, verbose, release).await })
}

/// Comando: forge build
async fn cmd_build(project_dir: PathBuf, _verbose: bool, release: bool) -> anyhow::Result<()> {
    let config = ForgeConfig::load(&project_dir)?;

    // 📦 Multi-módulo: compilar sub-módulos con DAG inter-proyecto
    if !config.modules.is_empty() {
        println!(
            "{}",
            format!("📦 Workspace detectado: {} sub-módulos", config.modules.len()).cyan().bold()
        );

        use cyrce_forge_core::dag::{Task, TaskAction};
        let mut modules_map = std::collections::HashMap::new(); // name -> path
        let mut dep_map = std::collections::HashMap::new(); // name -> deps

        for module_path in &config.modules {
            let module_dir = project_dir.join(module_path);
            if !module_dir.join("forge.toml").exists() {
                println!(
                    "   {}",
                    format!("⚠️  Módulo '{}' no tiene forge.toml, saltando...", module_path).yellow()
                );
                continue;
            }
            
            let mod_config = ForgeConfig::load(&module_dir)?;
            let mod_name = mod_config.project.name.clone();
            modules_map.insert(mod_name.clone(), module_path.clone());
            
            let mut local_deps = Vec::new();
            for val in mod_config.dependencies.values().chain(mod_config.test_dependencies.values()) {
                if val.starts_with("path:") {
                    let rel_path = val.trim_start_matches("path:");
                    let dep_dir = module_dir.join(rel_path);
                    if let Ok(dep_config) = ForgeConfig::load(&dep_dir) {
                        local_deps.push(dep_config.project.name);
                    }
                }
            }
            dep_map.insert(mod_name, local_deps);
        }

        let mut graph = cyrce_forge_core::dag::TaskGraph::new();
        for (name, deps) in &dep_map {
            graph.add_task(Task {
                name: name.clone(),
                description: format!("Build módulo {}", name),
                depends_on: deps.clone(),
                action: TaskAction::Composite,
            })?;
        }

        graph.validate().context("Se detectó un ciclo de dependencias entre los proyectos del workspace")?;
        
        let levels = graph.parallel_levels()?;
        for (i, level) in levels.iter().enumerate() {
            if level.len() > 1 {
                println!(
                    "   {}",
                    format!("⚡ Nivel {} — {} módulos en paralelo", i + 1, level.len()).yellow()
                );
            }
            
            let mut handles = Vec::new();
            for mod_name in level {
                let module_path = modules_map.get(mod_name).unwrap().clone();
                let module_dir = project_dir.join(&module_path);
                
                println!(
                    "   {}",
                    format!("🔨 Compilando módulo: {}", module_path).cyan()
                );
                
                let release_clone = release;
                let verbose_clone = _verbose;
                
                handles.push(tokio::spawn(async move {
                    let res = cmd_build_boxed(module_dir.clone(), verbose_clone, release_clone).await;
                    (module_path, res)
                }));
            }
            
            for handle in handles {
                let (m_path, res) = handle.await?;
                if let Err(e) = res {
                    return Err(anyhow::anyhow!("Error compilando módulo '{}': {}", m_path, e));
                }
            }
        }

        println!(
            "   {}",
            "✅ Todos los sub-módulos compilados".green()
        );
        
        // Si el directorio actual sólo es un workspace root (sin src), no seguimos
        let source_dir = project_dir.join(config.source_dir());
        if !source_dir.exists() {
            return Ok(());
        }
    }

    // 1. Verificación Caché Local
    let source_dir = project_dir.join(config.source_dir());
    let extensions = cyrce_forge_langs::extensions_for_lang(&config.project.lang);
    let mut cache = BuildCache::load(&project_dir)?;

    if !cache.has_changes(&source_dir, extensions)? {
        println!(
            "{}",
            "⚡ Sin cambios detectados — usando caché local".dimmed()
        );
        return Ok(());
    }

    // 2. Verificación Caché Remoto (Si está configurado)
    let output_dir_name = &config.project.output_dir;
    let mut used_remote = false;
    
    if let Some(remote_cfg) = &config.cache {
        // Intenta descargar el output compilado remotamente para este master_hash
        cache.update_hashes(&source_dir, extensions)?;
        if cache.download_from_remote(&project_dir, output_dir_name, remote_cfg).await? {
            used_remote = true;
            cache.save(&project_dir)?;
        }
    }

    // 3. Compilación o Skipping
    if !used_remote {

    // 🪝 Hooks pre-build
    hooks::run_pre_build(&config.hooks, &project_dir).await?;

    // Resolver dependencias si hay
    if !config.dependencies.is_empty() {
        resolve_dependencies(&config, &project_dir).await?;
    }

        // Compilar según el lenguaje
        match config.project.lang.as_str() {
            "java" => JavaModule::compile(&config, &project_dir).await?,
            "kotlin" => KotlinModule::compile(&config, &project_dir).await?,
            "python" => PythonModule::compile(&config, &project_dir).await?,
            _ => {}
        }

        // Actualizar caché
        cache.update_hashes(&source_dir, extensions)?;
        cache.save(&project_dir)?;

        // Si la compilación fue local y tenemos push habilitado, subir artefactos
        if let Some(remote_cfg) = &config.cache {
            cache.upload_to_remote(&project_dir, output_dir_name, remote_cfg).await?;
        }
    }

    // 🪝 Hooks post-build
    hooks::run_post_build(&config.hooks, &project_dir).await?;

    Ok(())
}

/// Comando: forge run
async fn cmd_run(project_dir: &PathBuf, verbose: bool) -> anyhow::Result<()> {
    // Primero compilar (en modo por defecto / no-release para run)
    cmd_build(project_dir.clone(), verbose, false).await?;

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

    // 🪝 Hooks pre-test
    hooks::run_pre_test(&config.hooks, project_dir).await?;

    match config.project.lang.as_str() {
        "java" => {
            cmd_build(project_dir.clone(), verbose, false).await?;
            JavaModule::test(&config, project_dir).await?;
        }
        "kotlin" => {
            cmd_build(project_dir.clone(), verbose, false).await?;
            KotlinModule::test(&config, project_dir).await?;
        }
        "python" => PythonModule::test(&config, project_dir).await?,
        _ => {}
    }

    // 🪝 Hooks post-test
    hooks::run_post_test(&config.hooks, project_dir).await?;

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

/// Resuelve dependencias según el lenguaje, excluyendo las locales (path:).
async fn resolve_dependencies(config: &ForgeConfig, project_dir: &PathBuf) -> anyhow::Result<()> {
    match config.project.lang.as_str() {
        "java" | "kotlin" => {
            let mut resolver = cyrce_forge_deps::maven::MavenResolver::new(project_dir);
            
            let remote_deps: std::collections::HashMap<String, String> = config.dependencies.clone().into_iter().filter(|(_, v)| !v.starts_with("path:")).collect();
            if !remote_deps.is_empty() {
                resolver.resolve_all(&remote_deps).await?;
            }
            
            let remote_test_deps: std::collections::HashMap<String, String> = config.test_dependencies.clone().into_iter().filter(|(_, v)| !v.starts_with("path:")).collect();
            if !remote_test_deps.is_empty() {
                resolver.resolve_test_deps(&remote_test_deps).await?;
            }
        }
        "python" => {
            let resolver = cyrce_forge_deps::pypi::PypiResolver::new();
            let remote_deps: std::collections::HashMap<String, String> = config.dependencies.clone().into_iter().filter(|(_, v)| !v.starts_with("path:")).collect();
            if !remote_deps.is_empty() {
                resolver.verify_all(&remote_deps).await?;
            }
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
async fn cmd_watch(project_dir: &PathBuf, dashboard: bool) -> anyhow::Result<()> {
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

    if dashboard {
        let p = project_dir.clone();
        tokio::spawn(async move {
            let _ = dashboard::cmd_dashboard(&p, 3000).await;
        });
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
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
    if let Err(e) = cmd_build(project_dir.clone(), false, false).await {
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

    let extensions = cyrce_forge_langs::extensions_for_lang(&config.project.lang);

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
                    match cmd_build(project_dir.clone(), false, false).await {
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

/// Comando: forge doctor
async fn cmd_doctor() -> anyhow::Result<()> {
    println!("{}", "🩺 Diagnóstico del Sistema FORGE".bold());
    println!("{}", "─".repeat(50).dimmed());

    let mut all_ok = true;
    let mut missing_tools: Vec<(&str, &str)> = Vec::new();

    // Verificar herramientas esenciales
    // (label, cmd, args, required, install_hint)
    let checks: Vec<(&str, &str, Vec<&str>, bool, &str)> = vec![
        ("Rust (rustc)", "rustc", vec!["--version"], true,
         "https://rustup.rs"),
        ("Cargo", "cargo", vec!["--version"], true,
         "Se instala con Rust: https://rustup.rs"),
        ("Git", "git", vec!["--version"], true,
         "https://git-scm.com/downloads"),
        ("Java (javac)", "javac", vec!["--version"], false,
         "https://adoptium.net (Temurin JDK 21+)"),
        ("JVM (java)", "java", vec!["--version"], false,
         "Se instala con el JDK"),
        ("Kotlin (kotlinc)", "kotlinc", vec!["-version"], false,
         "Descargar de: https://github.com/JetBrains/kotlin/releases\n              Extraer y agregar kotlinc/bin al PATH del sistema"),
        ("Python", "python", vec!["--version"], false,
         "https://python.org/downloads"),
        ("Pip", "pip", vec!["--version"], false,
         "Se instala con Python (python -m ensurepip)"),
    ];

    for (label, cmd, args, required, hint) in &checks {
        // En Windows, algunos tools como kotlinc son .bat — ejecutar via cmd /C
        let result = if cfg!(target_os = "windows") && *cmd == "kotlinc" {
            std::process::Command::new("cmd")
                .arg("/C")
                .arg(cmd)
                .args(args)
                .output()
        } else {
            std::process::Command::new(cmd).args(args).output()
        };
        match result {
            Ok(output) if output.status.success() => {
                let ver = String::from_utf8_lossy(&output.stdout);
                let ver = ver.trim();
                let ver = if ver.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    stderr.lines().next().unwrap_or("OK").trim().to_string()
                } else {
                    ver.lines().next().unwrap_or("OK").to_string()
                };
                println!("   {} {} — {}", "✅".green(), label, ver.dimmed());
            }
            _ => {
                if *required {
                    println!("   {} {} — {}", "❌".red(), label, "No encontrado (REQUERIDO)".red());
                    all_ok = false;
                } else {
                    println!("   {} {} — {}", "⚠️ ".yellow(), label, "No encontrado (opcional)".yellow());
                }
                missing_tools.push((label, hint));
            }
        }
    }

    // Verificar FORGE
    println!("\n{}", "📦 FORGE".bold());
    println!("   {} {} — {}", "✅".green(), "Versión", env!("CARGO_PKG_VERSION").dimmed());
    println!("   {} {} — {}", "✅".green(), "Ubicación", std::env::current_exe().unwrap_or_default().display().to_string().dimmed());

    // Verificar caché global
    if let Some(home) = dirs::home_dir() {
        let forge_cache = home.join(".forge");
        let repo_cache = home.join(".forge").join("repository");
        if forge_cache.exists() {
            let size = dir_size(&forge_cache);
            println!("   {} {} — {}", "✅".green(), "Caché global",
                format!("{} ({})", forge_cache.display(), format_bytes(size)).dimmed());
        } else {
            println!("   {} {} — {}", "ℹ️ ".cyan(), "Caché global", "No creada aún".dimmed());
        }
        if repo_cache.exists() {
            let count = std::fs::read_dir(&repo_cache).map(|r| r.count()).unwrap_or(0);
            println!("   {} {} — {}", "✅".green(), "Dependencias", format!("{} en caché", count).dimmed());
        }
    }

    // Mostrar sugerencias de instalación
    if !missing_tools.is_empty() {
        println!("\n{}", "💡 Sugerencias de instalación:".yellow().bold());
        for (tool, hint) in &missing_tools {
            println!("   {} {}", format!("{}:", tool).cyan(), hint);
        }
    }

    println!("\n{}", "─".repeat(50).dimmed());
    if all_ok {
        println!("{}", "🎉 ¡Sistema listo para FORGE!".green().bold());
    } else {
        println!("{}", "⚠️  Algunas herramientas requeridas no se encontraron.".yellow().bold());
    }
    println!();

    Ok(())
}

/// Comando: forge stats
async fn cmd_stats(project_dir: &PathBuf) -> anyhow::Result<()> {
    let config = ForgeConfig::load(project_dir)
        .context("No se encontró forge.toml. ¿Estás en un proyecto FORGE?")?;

    let source_dir = project_dir.join(config.source_dir());
    let extensions = cyrce_forge_langs::extensions_for_lang(&config.project.lang);

    println!("{}", "📊 Estadísticas del Proyecto".bold());
    println!("{}", "─".repeat(45).dimmed());
    println!("   {} {}", "Proyecto:".cyan(), config.project.name);
    println!("   {} {}", "Lenguaje:".cyan(), config.project.lang);

    if !source_dir.exists() {
        println!("\n   {}", "⚠️  Directorio fuente no encontrado".yellow());
        return Ok(());
    }

    let mut total_files = 0u64;
    let mut total_lines = 0u64;
    let mut total_bytes = 0u64;
    let mut files_by_ext: std::collections::HashMap<String, (u64, u64)> = std::collections::HashMap::new();

    for entry in walkdir::WalkDir::new(&source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_string();
            let is_relevant = extensions.iter().any(|e| ext_str == *e)
                || matches!(ext_str.as_str(), "toml" | "xml" | "json" | "yaml" | "yml" | "md" | "txt");

            if is_relevant {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let lines = std::fs::read_to_string(path)
                    .map(|content| content.lines().count() as u64)
                    .unwrap_or(0);

                total_files += 1;
                total_lines += lines;
                total_bytes += size;

                let entry = files_by_ext.entry(ext_str).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += lines;
            }
        }
    }

    println!("\n   {}", "Código Fuente:".cyan().bold());
    println!("   {} {} archivos", "Archivos:".cyan(), total_files);
    println!("   {} {} líneas", "Líneas:".cyan(), total_lines);
    println!("   {} {}", "Tamaño:".cyan(), format_bytes(total_bytes));

    if !files_by_ext.is_empty() {
        println!("\n   {}", "Por extensión:".cyan().bold());
        let mut sorted: Vec<_> = files_by_ext.into_iter().collect();
        sorted.sort_by(|a, b| b.1.1.cmp(&a.1.1));
        for (ext, (count, lines)) in &sorted {
            println!("      .{:<8} {} archivos, {} líneas", ext, count, lines);
        }
    }

    // Info del build
    let build_dir = project_dir.join(&config.project.output_dir);
    if build_dir.exists() {
        let build_size = dir_size(&build_dir);
        println!("\n   {}", "Build:".cyan().bold());
        println!("   {} {}", "Artefactos:".cyan(), format_bytes(build_size));
    }

    // Dependencias
    if !config.dependencies.is_empty() {
        println!("\n   {}", "Dependencias:".cyan().bold());
        println!("   {} {} definidas", "Total:".cyan(), config.dependencies.len());
    }

    // Tareas
    if !config.tasks.is_empty() {
        println!("\n   {}", "Tareas:".cyan().bold());
        for (name, task) in &config.tasks {
            println!("      ⚙️  {} → {}", name.bold(), task.command.dimmed());
        }
    }

    println!("\n{}", "─".repeat(45).dimmed());
    println!();

    Ok(())
}

/// Calcula el tamaño de un directorio recursivamente.
fn dir_size(path: &std::path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .sum()
}

/// Formatea bytes en formato legible.
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Comando: forge bench
async fn cmd_bench(project_dir: &PathBuf, verbose: bool) -> anyhow::Result<()> {
    let config = ForgeConfig::load(project_dir)?;

    println!("{}", "⏱️  Benchmark de Compilación".bold());
    println!("{}", "─".repeat(50).dimmed());
    println!("   {} {}", "Proyecto:".cyan(), config.project.name);
    println!("   {} {}\n", "Lenguaje:".cyan(), config.project.lang);

    let runs = 3;
    let mut times: Vec<f64> = Vec::new();

    for i in 1..=runs {
        // Limpiar primero
        let _ = cmd_clean(project_dir).await;

        println!(
            "{}",
            format!("   🔄 Ejecución {}/{}...", i, runs).dimmed()
        );

        let start = Instant::now();
        cmd_build(project_dir.clone(), verbose, false).await?;
        let elapsed = start.elapsed().as_secs_f64();
        times.push(elapsed);

        println!(
            "      {} {:.3}s\n",
            "Tiempo:".cyan(),
            elapsed
        );
    }

    // Calcular estadísticas
    let avg = times.iter().sum::<f64>() / times.len() as f64;
    let min = times.iter().cloned().fold(f64::MAX, f64::min);
    let max = times.iter().cloned().fold(f64::MIN, f64::max);

    println!("{}", "─".repeat(50).dimmed());
    println!("{}", "📊 Resultados".bold());
    println!("   {} {:.3}s", "Promedio:".cyan().bold(), avg);
    println!("   {} {:.3}s", "Mínimo: ".green(), min);
    println!("   {} {:.3}s", "Máximo: ".red(), max);
    println!("   {} {}", "Ejecuciones:".dimmed(), runs);

    // Comparar con benchmarks conocidos
    if avg < 1.0 {
        println!("\n   {}", "🚀 ¡Velocidad increíble! Sub-segundo.".green().bold());
    } else if avg < 5.0 {
        println!("\n   {}", "⚡ Compilación rápida.".green());
    } else if avg < 15.0 {
        println!("\n   {}", "🔨 Compilación normal.".yellow());
    } else {
        println!("\n   {}", "🐢 Compilación lenta — considera optimizar dependencias.".red());
    }

    println!();
    Ok(())
}

/// Comando: forge package
async fn cmd_package(project_dir: &PathBuf) -> anyhow::Result<()> {
    let config = ForgeConfig::load(project_dir)?;

    println!(
        "{}",
        format!("📦 Empaquetando {} v{}...", config.project.name, config.project.version).bold()
    );

    // Compilar primero
    cmd_build(project_dir.clone(), false, false).await?;

    // Crear directorio dist
    let dist_dir = project_dir.join("dist");
    std::fs::create_dir_all(&dist_dir)?;

    let package_name = format!(
        "{}-{}-{}",
        config.project.name,
        config.project.version,
        config.project.lang
    );

    match config.project.lang.as_str() {
        "java" | "kotlin" => {
            // Para Java/Kotlin: el JAR ya está en build/
            let build_dir = project_dir.join(&config.project.output_dir);
            let jar_name = format!("{}.jar", config.project.name);
            let jar_src = build_dir.join(&jar_name);
            let jar_dst = dist_dir.join(format!("{}.jar", package_name));

            if jar_src.exists() {
                std::fs::copy(&jar_src, &jar_dst)?;
                let size = std::fs::metadata(&jar_dst)?.len();
                println!("   {} {} ({})", "✅ JAR:".green(), jar_dst.display(), format_bytes(size));
            } else {
                // Copiar archivos .class si no hay JAR
                let classes_dir = build_dir.join("classes");
                if classes_dir.exists() {
                    let dest = dist_dir.join(format!("{}-classes", package_name));
                    copy_dir_recursive(&classes_dir, &dest)?;
                    println!("   {} {}", "✅ Classes:".green(), dest.display());
                } else {
                    println!("   {}", "⚠️  No se encontraron artefactos compilados".yellow());
                    return Ok(());
                }
            }
        }
        "python" => {
            // Para Python: copiar el source dir
            let source_dir = project_dir.join(config.source_dir());
            let dest = dist_dir.join(&package_name);
            std::fs::create_dir_all(&dest)?;

            // Copiar fuente
            copy_dir_recursive(&source_dir, &dest.join("src"))?;

            // Copiar forge.toml
            let forge_toml = project_dir.join("forge.toml");
            if forge_toml.exists() {
                std::fs::copy(&forge_toml, dest.join("forge.toml"))?;
            }

            // Crear requirements.txt
            if !config.dependencies.is_empty() {
                let reqs: Vec<String> = config
                    .dependencies
                    .iter()
                    .map(|(name, ver)| format!("{}=={}", name, ver))
                    .collect();
                std::fs::write(dest.join("requirements.txt"), reqs.join("\n"))?;
                println!("   {} requirements.txt", "✅ Creado:".green());
            }

            let size = dir_size(&dest);
            println!("   {} {} ({})", "✅ Paquete:".green(), dest.display(), format_bytes(size));
        }
        _ => {}
    }

    // Resumen
    let dist_size = dir_size(&dist_dir);
    println!(
        "\n{}",
        format!("📦 Empaquetado completado en dist/ ({})", format_bytes(dist_size))
            .green()
            .bold()
    );
    println!();

    Ok(())
}

/// Copia un directorio recursivamente.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
