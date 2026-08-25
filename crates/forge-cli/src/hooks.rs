use anyhow::Result;
use colored::*;
use cyrce_forge_core::config::HooksConfig;
use std::path::Path;

/// Ejecuta una lista de hooks (comandos shell) con un label descriptivo.
pub async fn run_hooks(hooks: &[String], label: &str, project_dir: &Path) -> Result<()> {
    if hooks.is_empty() {
        return Ok(());
    }

    println!(
        "   {}",
        format!(
            "🪝 Ejecutando hooks {}... ({} comando(s))",
            label,
            hooks.len()
        )
        .cyan()
    );

    for cmd_str in hooks {
        println!("   {}", format!("   ▶ {}", cmd_str).dimmed());

        let status = if cfg!(target_os = "windows") {
            tokio::process::Command::new("cmd")
                .args(["/C", cmd_str])
                .current_dir(project_dir)
                .status()
                .await?
        } else {
            tokio::process::Command::new("sh")
                .args(["-c", cmd_str])
                .current_dir(project_dir)
                .status()
                .await?
        };

        if !status.success() {
            println!(
                "   {}",
                format!(
                    "❌ Hook '{}' falló (exit code: {})",
                    cmd_str,
                    status.code().unwrap_or(-1)
                )
                .red()
            );
            return Err(anyhow::anyhow!(
                "Hook '{}' falló en fase {}",
                cmd_str,
                label
            ));
        }
    }

    println!(
        "   {}",
        format!("   ✅ Hooks {} completados", label).green()
    );
    Ok(())
}

/// Ejecuta los hooks pre-build si están definidos.
pub async fn run_pre_build(hooks: &HooksConfig, project_dir: &Path) -> Result<()> {
    run_hooks(&hooks.pre_build, "pre-build", project_dir).await
}

/// Ejecuta los hooks post-build si están definidos.
pub async fn run_post_build(hooks: &HooksConfig, project_dir: &Path) -> Result<()> {
    run_hooks(&hooks.post_build, "post-build", project_dir).await
}

/// Ejecuta los hooks pre-test si están definidos.
pub async fn run_pre_test(hooks: &HooksConfig, project_dir: &Path) -> Result<()> {
    run_hooks(&hooks.pre_test, "pre-test", project_dir).await
}

/// Ejecuta los hooks post-test si están definidos.
pub async fn run_post_test(hooks: &HooksConfig, project_dir: &Path) -> Result<()> {
    run_hooks(&hooks.post_test, "post-test", project_dir).await
}
