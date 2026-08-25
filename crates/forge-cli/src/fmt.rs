use anyhow::Result;
use colored::*;
use cyrce_forge_core::config::ForgeConfig;
use std::path::Path;

/// Formatea el código fuente del proyecto usando la herramienta nativa del lenguaje.
pub async fn cmd_fmt(project_dir: &Path) -> Result<()> {
    let config = ForgeConfig::load(project_dir)?;

    println!("   {}", "🎨 Formateando código fuente...".cyan());

    match config.project.lang.as_str() {
        "java" => fmt_java(project_dir, &config).await,
        "kotlin" => fmt_kotlin(project_dir, &config).await,
        "python" => fmt_python(project_dir, &config).await,
        other => {
            println!(
                "   {}",
                format!("⚠️  Formateo no soportado para lenguaje '{}'", other).yellow()
            );
            Ok(())
        }
    }
}

async fn fmt_java(project_dir: &Path, config: &ForgeConfig) -> Result<()> {
    let source_dir = config.source_dir();
    let source_path = project_dir.join(&source_dir);

    if !source_path.exists() {
        println!("   {}", "⚠️  No se encontró directorio fuente".yellow());
        return Ok(());
    }

    // Intentar google-java-format primero
    let status = tokio::process::Command::new("google-java-format")
        .arg("--replace")
        .arg("--glob")
        .arg(format!("{}/**/*.java", source_path.display()))
        .current_dir(project_dir)
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!(
                "   {}",
                "✅ Código Java formateado (google-java-format)".green()
            );
        }
        _ => {
            println!(
                "   {}",
                "💡 Tip: Instala 'google-java-format' para formateo automático de Java.".yellow()
            );
            println!(
                "   {}",
                "   https://github.com/google/google-java-format".dimmed()
            );
        }
    }
    Ok(())
}

async fn fmt_kotlin(project_dir: &Path, _config: &ForgeConfig) -> Result<()> {
    let status = tokio::process::Command::new("ktlint")
        .arg("--format")
        .arg("**/*.kt")
        .current_dir(project_dir)
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!("   {}", "✅ Código Kotlin formateado (ktlint)".green());
        }
        _ => {
            println!(
                "   {}",
                "💡 Tip: Instala 'ktlint' para formateo automático de Kotlin.".yellow()
            );
            println!("   {}", "   https://pinterest.github.io/ktlint/".dimmed());
        }
    }
    Ok(())
}

async fn fmt_python(project_dir: &Path, _config: &ForgeConfig) -> Result<()> {
    // Intentar black primero, luego autopep8
    let status = tokio::process::Command::new("black")
        .arg(".")
        .current_dir(project_dir)
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!("   {}", "✅ Código Python formateado (black)".green());
        }
        _ => {
            // Fallback a autopep8
            let status2 = tokio::process::Command::new("autopep8")
                .args(["--in-place", "--recursive", "."])
                .current_dir(project_dir)
                .status()
                .await;

            match status2 {
                Ok(s) if s.success() => {
                    println!("   {}", "✅ Código Python formateado (autopep8)".green());
                }
                _ => {
                    println!(
                        "   {}",
                        "💡 Tip: Instala 'black' o 'autopep8' para formateo automático de Python."
                            .yellow()
                    );
                    println!("   {}", "   pip install black".dimmed());
                }
            }
        }
    }
    Ok(())
}
