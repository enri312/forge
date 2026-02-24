// =============================================================================
// 🔥 FORGE — Resolución de Dependencias: PyPI
// =============================================================================
// Consulta PyPI para verificar paquetes Python.
// La instalación real se delega a pip dentro del venv.
// =============================================================================

use colored::Colorize;

use cyrce_forge_core::error::{ForgeError, ForgeResult};

/// URL base de la API JSON de PyPI.
const PYPI_API_URL: &str = "https://pypi.org/pypi";

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
            client: reqwest::Client::new(),
        }
    }

    /// Verifica que un paquete exista en PyPI.
    pub async fn verify_package(&self, name: &str, version: &str) -> ForgeResult<PypiPackageInfo> {
        let url = if version == "*" || version.is_empty() {
            format!("{}/{}/json", PYPI_API_URL, name)
        } else {
            format!("{}/{}/{}/json", PYPI_API_URL, name, version)
        };

        let response = self
            .client
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

        let info: PypiPackageInfo =
            response
                .json()
                .await
                .map_err(|e| ForgeError::DownloadError {
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
            format!(
                "🐍 Verificando {} paquetes en PyPI...",
                dependencies.len()
            )
            .cyan()
        );

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
                    println!(
                        "   {}",
                        format!("   ⚠️  {}: {}", name, e).yellow()
                    );
                }
            }
        }

        Ok(())
    }
}

impl Default for PypiResolver {
    fn default() -> Self {
        Self::new()
    }
}
