// =============================================================================
// 🔥 FORGE — Comando: tree
// =============================================================================
// Visualiza el árbol de dependencias del proyecto.
// Muestra tanto las directas como un resumen de las test-dependencies.
// =============================================================================

use std::path::PathBuf;
use colored::Colorize;
use cyrce_forge_core::config::ForgeConfig;

pub async fn cmd_tree(project_dir: &PathBuf) -> anyhow::Result<()> {
    let config = ForgeConfig::load(project_dir)?;
    
    println!("{} {}", "🌲".green(), format!("Árbol de dependencias para '{}'", config.project.name).bold());
    
    if config.dependencies.is_empty() && config.test_dependencies.is_empty() {
        println!("   {}", "No hay dependencias declaradas en este proyecto.".dimmed());
        return Ok(());
    }

    if !config.dependencies.is_empty() {
        println!("\n   {}", "[dependencies]".cyan());
        let count = config.dependencies.len();
        for (i, (key, val)) in config.dependencies.iter().enumerate() {
            let symbol = if i == count - 1 { "└──" } else { "├──" };
            println!("   {} {} {}", symbol, key.bold(), val.dimmed());
        }
    }

    if !config.test_dependencies.is_empty() {
        println!("\n   {}", "[test-dependencies]".purple());
        let count = config.test_dependencies.len();
        for (i, (key, val)) in config.test_dependencies.iter().enumerate() {
            let symbol = if i == count - 1 { "└──" } else { "├──" };
            println!("   {} {} {}", symbol, key.bold(), val.dimmed());
        }
    }

    println!("\n   {}", "Nota: El árbol completo con sub-dependencias transitivas".dimmed());
    println!("   {}", "      se visualiza resolviéndolas durante 'forge build'.".dimmed());

    Ok(())
}
