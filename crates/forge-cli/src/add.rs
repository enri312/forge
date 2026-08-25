// =============================================================================
// 🔥 FORGE — Comando: add
// =============================================================================
// Añade dependencias dinámicamente al archivo forge.toml sin edición manual.
// Localiza la sección [dependencies] o [test-dependencies] e inserta.
// =============================================================================

use colored::Colorize;
use cyrce_forge_core::config::ForgeConfig;
use cyrce_forge_deps::maven::MavenCoordinate;
use std::path::Path;

pub async fn cmd_add(project_dir: &Path, dep: &str, is_test: bool) -> anyhow::Result<()> {
    let toml_path = project_dir.join("forge.toml");

    if !toml_path.exists() {
        return Err(anyhow::anyhow!(
            "No se encontró forge.toml en el directorio actual."
        ));
    }

    let config = ForgeConfig::load(project_dir)?;
    let mut content = std::fs::read_to_string(&toml_path)?;
    let target_section = if is_test {
        "[test-dependencies]"
    } else {
        "[dependencies]"
    };

    let (key, val) = match config.project.lang.as_str() {
        "java" | "kotlin" => {
            let parts: Vec<&str> = dep.split(':').collect();
            let pair = match parts.as_slice() {
                [group, artifact, version] => {
                    (format!("{}:{}", group, artifact), (*version).to_string())
                }
                [group, artifact, version, classifier] => (
                    format!("{}:{}:{}", group, artifact, classifier),
                    (*version).to_string(),
                ),
                _ => {
                    return Err(anyhow::anyhow!(
                        "Formato Maven inválido. Usa groupId:artifactId:version[:classifier]"
                    ));
                }
            };
            MavenCoordinate::parse(&pair.0, &pair.1)?;
            pair
        }
        "python" => {
            let (name, version) = dep.split_once("==").unwrap_or((dep, "*"));
            if name.is_empty()
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
                || version.is_empty()
                || version
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '"' | '\''))
            {
                return Err(anyhow::anyhow!(
                    "Formato PyPI inválido. Usa paquete o paquete==versión"
                ));
            }
            (name.to_string(), version.to_string())
        }
        _ => unreachable!("ForgeConfig ya valida el lenguaje"),
    };

    let dep_line = format!("\"{}\" = \"{}\"", key, val);

    // Prevención simple de duplicados
    if content.contains(&format!("\"{}\"", key)) {
        println!(
            "   {} La dependencia '{}' ya existe en forge.toml",
            "⚠️".yellow(),
            key
        );
        return Ok(());
    }

    // Buscar sección
    if let Some(pos) = content.find(target_section) {
        // Encontrar el siguiente salto de línea y agregar allí
        let insert_pos = content[pos..].find('\n').unwrap_or(0) + pos + 1;
        content.insert_str(insert_pos, &format!("{}\n", dep_line));
    } else {
        // Crear sección al final si no existe
        content.push_str(&format!("\n{}\n{}\n", target_section, dep_line));
    }

    // No persistir nunca una edición que deje forge.toml inválido.
    ForgeConfig::parse(&content)?;
    let temp_path = toml_path.with_extension("toml.tmp");
    let backup_path = toml_path.with_extension(format!("toml.bak-{}", std::process::id()));
    std::fs::write(&temp_path, content)?;
    std::fs::rename(&toml_path, &backup_path)?;
    if let Err(error) = std::fs::rename(&temp_path, &toml_path) {
        let _ = std::fs::rename(&backup_path, &toml_path);
        let _ = std::fs::remove_file(&temp_path);
        return Err(error.into());
    }
    let _ = std::fs::remove_file(&backup_path);

    let label = if is_test {
        "Dependencia de test"
    } else {
        "Dependencia"
    };
    println!("   {} {} añadida a forge.toml", "✅".green(), label.bold());
    println!("   {} {}", "📦".cyan(), dep_line.bright_black());

    Ok(())
}
