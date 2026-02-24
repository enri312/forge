// =============================================================================
// 🔥 FORGE — Motor Core: Ejecutor Paralelo de Tareas
// =============================================================================
// Ejecuta tareas respetando dependencias y paralelizando cuando es posible.
// Patrón moderno: async/await con tokio, ejecución por niveles del DAG.
// =============================================================================

use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::process::Command;

use crate::cache::BuildCache;
use crate::dag::{TaskAction, TaskGraph};
use crate::error::{ForgeError, ForgeResult};

/// Resultado de la ejecución de una tarea individual.
#[derive(Debug)]
pub struct TaskResult {
    /// Nombre de la tarea
    pub name: String,
    /// Si se ejecutó correctamente
    pub success: bool,
    /// Tiempo de ejecución
    pub duration: Duration,
    /// Salida estándar capturada
    pub stdout: String,
    /// Salida de errores capturada
    pub stderr: String,
    /// Si se usó caché (no se re-ejecutó)
    pub cached: bool,
}

/// Resultado general de un build.
#[derive(Debug)]
pub struct BuildResult {
    /// Resultados de cada tarea individual
    pub tasks: Vec<TaskResult>,
    /// Tiempo total
    pub total_duration: Duration,
    /// Si el build fue exitoso
    pub success: bool,
}

/// Ejecutor de tareas del build system.
pub struct Executor {
    /// Directorio raíz del proyecto
    project_dir: std::path::PathBuf,
    /// Sistema de caché incremental
    cache: BuildCache,
    /// Si se debe mostrar salida verbosa
    verbose: bool,
}

impl Executor {
    /// Crea un nuevo ejecutor.
    pub fn new(project_dir: &Path, verbose: bool) -> ForgeResult<Self> {
        let cache = BuildCache::load(project_dir)?;
        Ok(Self {
            project_dir: project_dir.to_path_buf(),
            cache,
            verbose,
        })
    }

    /// Ejecuta todas las tareas del grafo respetando dependencias.
    /// Las tareas sin dependencias entre sí se ejecutan en paralelo.
    pub async fn execute(&mut self, graph: &TaskGraph) -> ForgeResult<BuildResult> {
        let start = Instant::now();
        let levels = graph.parallel_levels()?;
        let mut all_results: Vec<TaskResult> = Vec::new();
        let mut success = true;

        let multi = MultiProgress::new();

        println!(
            "\n{}",
            format!("🔥 FORGE v{} — Iniciando build...", env!("CARGO_PKG_VERSION"))
                .bold()
                .cyan()
        );
        println!(
            "{}",
            format!(
                "   📋 {} tareas en {} niveles de ejecución\n",
                graph.len(),
                levels.len()
            )
            .dimmed()
        );

        for (level_idx, level) in levels.iter().enumerate() {
            if !success {
                break;
            }

            if level.len() > 1 {
                println!(
                    "{}",
                    format!("   ⚡ Nivel {} — {} tareas en paralelo", level_idx + 1, level.len())
                        .yellow()
                );
            }

            // Ejecutar tareas del mismo nivel en paralelo
            let mut handles = Vec::new();

            for task_name in level {
                let task = graph
                    .get_task(task_name)
                    .ok_or_else(|| ForgeError::TaskNotFound {
                        task_name: task_name.clone(),
                    })?
                    .clone();

                let project_dir = self.project_dir.clone();
                let verbose = self.verbose;

                let pb = multi.add(ProgressBar::new_spinner());
                pb.set_style(
                    ProgressStyle::default_spinner()
                        .template("   {spinner:.cyan} {msg}")
                        .unwrap(),
                );
                pb.set_message(task.name.to_string());

                handles.push(tokio::spawn(async move {
                    let result = execute_single_task(&task, &project_dir, verbose, &pb).await;
                    pb.finish_and_clear();
                    result
                }));
            }

            // Esperar que todas las tareas del nivel terminen
            for handle in handles {
                match handle.await {
                    Ok(Ok(result)) => {
                        let status = if result.cached {
                            "⚡ CACHÉ".dimmed().to_string()
                        } else if result.success {
                            "✅ OK".green().to_string()
                        } else {
                            success = false;
                            "❌ FALLÓ".red().to_string()
                        };

                        let duration_str =
                            format!("({:.1}ms)", result.duration.as_secs_f64() * 1000.0).dimmed();

                        println!(
                            "   {} {} {}",
                            status,
                            result.name.bold(),
                            duration_str
                        );

                        if !result.success && !result.stderr.is_empty() {
                            println!("\n{}", "   ── Error ──".red().bold());
                            for line in result.stderr.lines().take(20) {
                                println!("      {}", line.red());
                            }
                            println!();
                        }

                        if !result.success {
                            success = false;
                        }

                        all_results.push(result);
                    }
                    Ok(Err(e)) => {
                        success = false;
                        println!("   {} {}", "❌ Error:".red().bold(), e);
                    }
                    Err(e) => {
                        success = false;
                        println!("   {} Tarea panicked: {}", "💀".red(), e);
                    }
                }
            }
        }

        let total_duration = start.elapsed();

        // Resumen final
        println!();
        if success {
            println!(
                "{}",
                format!(
                    "🔥 BUILD EXITOSO en {:.2}s ({} tareas)",
                    total_duration.as_secs_f64(),
                    all_results.len()
                )
                .green()
                .bold()
            );
        } else {
            println!(
                "{}",
                format!(
                    "💀 BUILD FALLIDO en {:.2}s",
                    total_duration.as_secs_f64()
                )
                .red()
                .bold()
            );
        }
        println!();

        // Guardar caché actualizado
        self.cache.save(&self.project_dir)?;

        Ok(BuildResult {
            tasks: all_results,
            total_duration,
            success,
        })
    }

    /// Devuelve referencia mutable al caché para actualizaciones externas.
    pub fn cache_mut(&mut self) -> &mut BuildCache {
        &mut self.cache
    }
}

/// Ejecuta una tarea individual.
async fn execute_single_task(
    task: &crate::dag::Task,
    project_dir: &Path,
    verbose: bool,
    pb: &ProgressBar,
) -> ForgeResult<TaskResult> {
    let start = Instant::now();

    pb.set_message(format!("Ejecutando: {}", task.name));

    let (success, stdout, stderr) = match &task.action {
        TaskAction::Command(cmd) => {
            run_external_command(cmd, project_dir, verbose).await?
        }
        TaskAction::Internal(_internal) => {
            // Las tareas internas serán manejadas por los módulos de lenguaje
            // Por ahora, simplemente se marcan como exitosas
            (true, String::new(), String::new())
        }
        TaskAction::Composite => {
            // Las tareas compuestas no ejecutan nada, solo agrupan dependencias
            (true, String::new(), String::new())
        }
    };

    Ok(TaskResult {
        name: task.name.clone(),
        success,
        duration: start.elapsed(),
        stdout,
        stderr,
        cached: false,
    })
}

/// Ejecuta un comando externo del sistema.
async fn run_external_command(
    command: &str,
    working_dir: &Path,
    _verbose: bool,
) -> ForgeResult<(bool, String, String)> {
    // En Windows usamos cmd /C, en Unix usamos sh -c
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command])
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
    } else {
        Command::new("sh")
            .args(["-c", command])
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok((output.status.success(), stdout, stderr))
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Err(ForgeError::CommandNotFound {
                    command: command.to_string(),
                }
                .into())
            } else {
                Err(ForgeError::IoError {
                    path: working_dir.to_path_buf(),
                    message: format!("Error al ejecutar '{}': {}", command, e),
                }
                .into())
            }
        }
    }
}
