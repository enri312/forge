// =============================================================================
// 🔥 FORGE — Resolución de Dependencias: Maven Central
// =============================================================================
// Descarga JARs y resuelve dependencias transitivas desde Maven Central.
// =============================================================================

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use colored::Colorize;

use forge_core::error::{ForgeError, ForgeResult};

/// URL base de Maven Central.
const MAVEN_CENTRAL_URL: &str = "https://repo1.maven.org/maven2";

/// Coordenadas Maven (groupId:artifactId:version).
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct MavenCoordinate {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
}

impl MavenCoordinate {
    /// Parsea una coordenada en formato "groupId:artifactId".
    pub fn parse(key: &str, version: &str) -> ForgeResult<Self> {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() != 2 {
            return Err(ForgeError::DependencyResolutionFailed {
                dependency: format!(
                    "'{}' — Formato esperado: 'groupId:artifactId'",
                    key
                ),
            }
            .into());
        }

        Ok(Self {
            group_id: parts[0].to_string(),
            artifact_id: parts[1].to_string(),
            version: version.to_string(),
        })
    }

    /// Genera la URL del JAR en Maven Central.
    pub fn jar_url(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}-{}.jar",
            MAVEN_CENTRAL_URL,
            self.group_id.replace('.', "/"),
            self.artifact_id,
            self.version,
            self.artifact_id,
            self.version
        )
    }

    /// Genera la URL del POM en Maven Central.
    pub fn pom_url(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}-{}.pom",
            MAVEN_CENTRAL_URL,
            self.group_id.replace('.', "/"),
            self.artifact_id,
            self.version,
            self.artifact_id,
            self.version
        )
    }

    /// Nombre del archivo JAR.
    pub fn jar_filename(&self) -> String {
        format!("{}-{}.jar", self.artifact_id, self.version)
    }

    /// Representación legible.
    pub fn display(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }
}

/// Resuelve y descarga dependencias Maven.
pub struct MavenResolver {
    /// Cliente HTTP reutilizable
    client: reqwest::Client,
    /// Directorio donde se cachean JARs
    cache_dir: PathBuf,
    /// Dependencias ya resueltas (evitar ciclos)
    resolved: HashSet<String>,
}

impl MavenResolver {
    /// Crea un nuevo resolver.
    pub fn new(project_dir: &Path) -> Self {
        let cache_dir = project_dir.join(".forge").join("deps");
        Self {
            client: reqwest::Client::new(),
            cache_dir,
            resolved: HashSet::new(),
        }
    }

    /// Resuelve y descarga todas las dependencias a .forge/deps/.
    pub async fn resolve_all(
        &mut self,
        dependencies: &std::collections::HashMap<String, String>,
    ) -> ForgeResult<Vec<PathBuf>> {
        std::fs::create_dir_all(&self.cache_dir).map_err(|e| ForgeError::IoError {
            path: self.cache_dir.clone(),
            message: e.to_string(),
        })?;

        let mut downloaded = Vec::new();

        println!(
            "   {}",
            format!(
                "📦 Resolviendo {} dependencias de Maven Central...",
                dependencies.len()
            )
            .cyan()
        );

        for (key, version) in dependencies {
            let coord = MavenCoordinate::parse(key, version)?;
            let jar_path = self.download_dependency(&coord).await?;
            downloaded.push(jar_path);
        }

        println!(
            "   {}",
            format!("✅ {} dependencias descargadas", downloaded.len()).green()
        );

        Ok(downloaded)
    }

    /// Descarga un JAR individual si no está en caché.
    async fn download_dependency(&mut self, coord: &MavenCoordinate) -> ForgeResult<PathBuf> {
        let key = coord.display();

        // Evitar resolver la misma dependencia dos veces
        if self.resolved.contains(&key) {
            return Ok(self.cache_dir.join(coord.jar_filename()));
        }

        let jar_path = self.cache_dir.join(coord.jar_filename());

        // Si ya existe en caché, no descargar
        if jar_path.exists() {
            self.resolved.insert(key);
            println!(
                "   {}",
                format!("   ⚡ {} (caché)", coord.display()).dimmed()
            );
            return Ok(jar_path);
        }

        println!(
            "   {}",
            format!("   ⬇️  Descargando {}...", coord.display()).dimmed()
        );

        let url = coord.jar_url();
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
            return Err(ForgeError::DownloadError {
                url,
                message: format!("HTTP {}", response.status()),
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

        std::fs::write(&jar_path, &bytes).map_err(|e| ForgeError::IoError {
            path: jar_path.clone(),
            message: e.to_string(),
        })?;

        self.resolved.insert(key);
        Ok(jar_path)
    }
}
