// =============================================================================
// 🔥 FORGE — Módulos de Lenguaje: Punto de Entrada
// =============================================================================

pub mod java;
pub mod kotlin;
pub mod python;

/// Extensiones de archivo por lenguaje (para caché incremental).
pub fn extensions_for_lang(lang: &str) -> &[&str] {
    match lang {
        "java" => &["java"],
        "kotlin" => &["kt", "kts"],
        "python" => &["py"],
        _ => &[],
    }
}
