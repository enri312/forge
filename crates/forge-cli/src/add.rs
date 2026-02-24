// =============================================================================
// 🔥 FORGE — Comando: add
// =============================================================================
// Añade dependencias dinámicamente al archivo forge.toml sin edición manual.
// Localiza la sección [dependencies] o [test-dependencies] e inserta.
// =============================================================================

use std::path::PathBuf;
use colored::Colorize;

pub async fn cmd_add(project_dir: &PathBuf, dep: &str, is_test: bool) -> anyhow::Result<()> {
    let toml_path = project_dir.join("forge.toml");
    
    if !toml_path.exists() {
        return Err(anyhow::anyhow!("No se encontró forge.toml en el directorio actual."));
    }

    let mut content = std::fs::read_to_string(&toml_path)?;
    let target_section = if is_test { "[test-dependencies]" } else { "[dependencies]" };

    // Validar formato básico para Java/Kotlin (groupId:artifactId:version)
    // Para Python, suele ser (name==version) o solo (name)
    // Asumiremos que si viene un paquete con dos ':' es groupId:artifactId:version
    let parts: Vec<&str> = dep.split(':').collect();
    let (key, val) = if parts.len() == 3 {
        (format!("{}:{}", parts[0], parts[1]), parts[2].to_string())
    } else {
        // Formato fallback genérico (Python o dependencias locales)
        (dep.to_string(), "latest".to_string())
    };

    let dep_line = format!("\"{}\" = \"{}\"", key, val);

    // Prevención simple de duplicados
    if content.contains(&format!("\"{}\"", key)) {
        println!("   {} La dependencia '{}' ya existe en forge.toml", "⚠️".yellow(), key);
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

    std::fs::write(&toml_path, content)?;

    let label = if is_test { "Dependencia de test" } else { "Dependencia" };
    println!("   {} {} añadida a forge.toml", "✅".green(), label.bold());
    println!("   {} {}", "📦".cyan(), dep_line.bright_black());

    Ok(())
}
