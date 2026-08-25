use anyhow::Result;
use colored::*;
use cyrce_forge_core::config::ForgeConfig;
use std::path::Path;

/// Ejecuta análisis estático (linting) sobre el código fuente.
pub async fn cmd_lint(project_dir: &Path) -> Result<()> {
    let config = ForgeConfig::load(project_dir)?;

    println!("   {}", "🔍 Ejecutando análisis estático (lint)...".cyan());

    match config.project.lang.as_str() {
        "java" => lint_java(project_dir, &config).await,
        "kotlin" => lint_kotlin(project_dir).await,
        "python" => lint_python(project_dir).await,
        other => {
            println!(
                "   {}",
                format!("⚠️  Linting no soportado para lenguaje '{}'", other).yellow()
            );
            Ok(())
        }
    }
}

async fn lint_java(project_dir: &Path, config: &ForgeConfig) -> Result<()> {
    let source_dir = config.source_dir();

    // Intentar checkstyle
    let status = tokio::process::Command::new("checkstyle")
        .arg("-c")
        .arg("/google_checks.xml")
        .arg(&source_dir)
        .current_dir(project_dir)
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!(
                "   {}",
                "✅ Análisis Java completado sin errores (checkstyle)".green()
            );
        }
        Ok(s) => {
            println!(
                "   {}",
                format!(
                    "⚠️  Checkstyle reportó problemas (exit {})",
                    s.code().unwrap_or(-1)
                )
                .yellow()
            );
        }
        _ => {
            println!(
                "   {}",
                "💡 Tip: Instala 'checkstyle' para análisis estático de Java.".yellow()
            );
            println!("   {}", "   https://checkstyle.org/".dimmed());
        }
    }
    Ok(())
}

async fn lint_kotlin(project_dir: &Path) -> Result<()> {
    let status = tokio::process::Command::new("detekt")
        .current_dir(project_dir)
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!(
                "   {}",
                "✅ Análisis Kotlin completado sin errores (detekt)".green()
            );
        }
        Ok(s) => {
            println!(
                "   {}",
                format!(
                    "⚠️  Detekt reportó problemas (exit {})",
                    s.code().unwrap_or(-1)
                )
                .yellow()
            );
        }
        _ => {
            println!(
                "   {}",
                "💡 Tip: Instala 'detekt' para análisis estático de Kotlin.".yellow()
            );
            println!("   {}", "   https://detekt.dev/".dimmed());
        }
    }
    Ok(())
}

async fn lint_python(project_dir: &Path) -> Result<()> {
    // Intentar ruff primero (moderno), luego flake8
    let status = tokio::process::Command::new("ruff")
        .arg("check")
        .arg(".")
        .current_dir(project_dir)
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!(
                "   {}",
                "✅ Análisis Python completado sin errores (ruff)".green()
            );
            return Ok(());
        }
        Ok(s) => {
            println!(
                "   {}",
                format!(
                    "⚠️  Ruff reportó problemas (exit {})",
                    s.code().unwrap_or(-1)
                )
                .yellow()
            );
            return Ok(());
        }
        _ => {}
    }

    // Fallback a flake8
    let status = tokio::process::Command::new("flake8")
        .arg(".")
        .current_dir(project_dir)
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!(
                "   {}",
                "✅ Análisis Python completado sin errores (flake8)".green()
            );
        }
        Ok(s) => {
            println!(
                "   {}",
                format!(
                    "⚠️  Flake8 reportó problemas (exit {})",
                    s.code().unwrap_or(-1)
                )
                .yellow()
            );
        }
        _ => {
            println!(
                "   {}",
                "💡 Tip: Instala 'ruff' (recomendado) o 'flake8' para análisis de Python.".yellow()
            );
            println!("   {}", "   pip install ruff".dimmed());
        }
    }
    Ok(())
}
