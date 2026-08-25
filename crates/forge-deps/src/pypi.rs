// =============================================================================
// 🔥 FORGE — Resolución de Dependencias: PyPI
// =============================================================================
// Consulta PyPI para verificar paquetes Python.
// La instalación real se delega a pip dentro del venv.
// =============================================================================

use colored::Colorize;
use std::time::Duration;

use cyrce_forge_core::error::{ForgeError, ForgeResult};

/// URL base de la API JSON de PyPI.
const PYPI_API_URL: &str = "https://pypi.org/pypi";
const MAX_PYPI_RESPONSE_BYTES: u64 = 5 * 1024 * 1024;

/// Información de un paquete PyPI.
#[derive(Debug, serde::Deserialize)]
pub struct PypiPackageInfo {
    pub info: PypiInfo,
}

#[derive(Debug, serde::Deserialize)]
pub struct PypiInfo {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
}

/// Resolver de dependencias PyPI.
pub struct PypiResolver {
    client: reqwest::Client,
}

impl PypiResolver {
    /// Crea un nuevo resolver.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    /// Verifica que un paquete exista en PyPI.
    pub async fn verify_package(&self, name: &str, version: &str) -> ForgeResult<PypiPackageInfo> {
        validate_package_input(name, version)?;
        let url = if version == "*" || version.is_empty() {
            format!("{}/{}/json", PYPI_API_URL, name)
        } else {
            format!("{}/{}/{}/json", PYPI_API_URL, name, version)
        };

        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| ForgeError::DownloadError {
                    url: url.clone(),
                    message: e.to_string(),
                })?;

        if !response.status().is_success() {
            return Err(ForgeError::DependencyResolutionFailed {
                dependency: format!(
                    "{} {} — No encontrado en PyPI (HTTP {})",
                    name,
                    version,
                    response.status()
                ),
            }
            .into());
        }

        if response
            .content_length()
            .is_some_and(|length| length > MAX_PYPI_RESPONSE_BYTES)
        {
            return Err(ForgeError::DownloadError {
                url,
                message: "La respuesta de PyPI excede el límite de 5 MiB".to_string(),
            }
            .into());
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ForgeError::DownloadError {
                url: url.clone(),
                message: e.to_string(),
            })?;
        if bytes.len() as u64 > MAX_PYPI_RESPONSE_BYTES {
            return Err(ForgeError::DownloadError {
                url,
                message: "La respuesta de PyPI excede el límite de 5 MiB".to_string(),
            }
            .into());
        }
        let info: PypiPackageInfo =
            serde_json::from_slice(&bytes).map_err(|e| ForgeError::DownloadError {
                url,
                message: format!("Error al parsear respuesta de PyPI: {}", e),
            })?;

        Ok(info)
    }

    /// Verifica todas las dependencias Python del proyecto.
    pub async fn verify_all(
        &self,
        dependencies: &std::collections::HashMap<String, String>,
    ) -> ForgeResult<()> {
        println!(
            "   {}",
            format!("🐍 Verificando {} paquetes en PyPI...", dependencies.len()).cyan()
        );

        let mut failures = Vec::new();
        for (name, version) in dependencies {
            match self.verify_package(name, version).await {
                Ok(info) => {
                    println!(
                        "   {}",
                        format!(
                            "   ✅ {} v{} — {}",
                            info.info.name,
                            info.info.version,
                            info.info.summary.as_deref().unwrap_or("Sin descripción")
                        )
                        .dimmed()
                    );
                }
                Err(e) => {
                    println!("   {}", format!("   ⚠️  {}: {}", name, e).yellow());
                    failures.push(name.clone());
                }
            }
        }

        if !failures.is_empty() {
            return Err(ForgeError::DependencyResolutionFailed {
                dependency: format!(
                    "Paquetes PyPI inválidos o no disponibles: {}",
                    failures.join(", ")
                ),
            }
            .into());
        }

        Ok(())
    }
}

fn validate_package_input(name: &str, version: &str) -> ForgeResult<()> {
    let valid_name = !name.is_empty()
        && name.len() <= 214
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    let valid_version = version == "*"
        || (!version.is_empty()
            && version.len() <= 128
            && version
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+')));

    if !valid_name || !valid_version {
        return Err(ForgeError::DependencyResolutionFailed {
            dependency: format!("Paquete PyPI inválido: '{} {}'", name, version),
        }
        .into());
    }
    Ok(())
}

impl Default for PypiResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_pypi_names_and_versions() {
        assert!(validate_package_input("requests", "2.32.0").is_ok());
        assert!(validate_package_input("../escape", "1.0").is_err());
        assert!(validate_package_input("requests", "../../1.0").is_err());
    }
}
