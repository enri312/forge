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

    /// Carga el directorio en caché para dependencias de prueba
    pub fn test_cache_dir(&self) -> PathBuf {
        self.cache_dir.parent().unwrap().join("test-deps")
    }

    /// Resuelve y descarga todas las dependencias runtime a .forge/deps/.
    pub async fn resolve_all(
        &mut self,
        dependencies: &std::collections::HashMap<String, String>,
    ) -> ForgeResult<Vec<PathBuf>> {
        self.resolve_internal(dependencies, &self.cache_dir.clone()).await
    }

    /// Resuelve y descarga dependencias de prueba a .forge/test-deps/.
    pub async fn resolve_test_deps(
        &mut self,
        dependencies: &std::collections::HashMap<String, String>,
    ) -> ForgeResult<Vec<PathBuf>> {
        self.resolve_internal(dependencies, &self.test_cache_dir()).await
    }

    /// Implementación interna de resolución a un directorio específico.
    /// Soporta resolución TRANSITIVA: descarga cada JAR, lee su POM y resuelve sub-dependencias.
    async fn resolve_internal(
        &mut self,
        dependencies: &std::collections::HashMap<String, String>,
        target_dir: &Path,
    ) -> ForgeResult<Vec<PathBuf>> {
        std::fs::create_dir_all(target_dir).map_err(|e| ForgeError::IoError {
            path: target_dir.to_path_buf(),
            message: e.to_string(),
        })?;

        let mut downloaded = Vec::new();

        println!(
            "   {}",
            format!(
                "📦 Resolviendo {} dependencias en Maven Central...",
                dependencies.len()
            )
            .cyan()
        );

        for (key, version) in dependencies {
            let coord = MavenCoordinate::parse(key, version)?;
            self.resolve_recursive(&coord, target_dir, &mut downloaded, 0).await?;
        }

        println!(
            "   {}",
            format!("✅ {} dependencias resueltas (incluyendo transitivas)", downloaded.len()).green()
        );

        Ok(downloaded)
    }

    /// Resolución recursiva: descarga JAR + lee POM + resuelve sub-dependencias.
    /// `depth` limita la profundidad para evitar ciclos infinitos.
    fn resolve_recursive<'a>(
        &'a mut self,
        coord: &'a MavenCoordinate,
        target_dir: &'a Path,
        downloaded: &'a mut Vec<PathBuf>,
        depth: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ForgeResult<()>> + 'a>> {
        Box::pin(async move {
            // Límite de profundidad para evitar ciclos
            if depth > 5 {
                return Ok(());
            }

            let key = format!("{}:{}", target_dir.to_string_lossy(), coord.display());

            // Evitar resolver la misma dependencia dos veces
            if self.resolved.contains(&key) {
                return Ok(());
            }

            // Descargar el JAR principal
            let jar_path = self.download_dependency(coord, target_dir).await?;
            downloaded.push(jar_path);

            // Intentar leer el POM para dependencias transitivas
            if let Ok(transitive_deps) = self.fetch_transitive_deps(coord).await {
                for dep_coord in transitive_deps {
                    self.resolve_recursive(&dep_coord, target_dir, downloaded, depth + 1).await?;
                }
            }

            Ok(())
        })
    }

    /// Descarga y parsea el POM de una coordenada Maven para extraer dependencias transitivas.
    /// Solo extrae dependencias con scope "compile" o sin scope (default=compile).
    /// Ignora dependencias con scope "test", "provided" o "system".
    async fn fetch_transitive_deps(
        &self,
        coord: &MavenCoordinate,
    ) -> ForgeResult<Vec<MavenCoordinate>> {
        let pom_url = coord.pom_url();

        let response = self
            .client
            .get(&pom_url)
            .send()
            .await
            .map_err(|e| ForgeError::DownloadError {
                url: pom_url.clone(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Ok(Vec::new()); // POM no encontrado, no es error fatal
        }

        let pom_text = response
            .text()
            .await
            .map_err(|e| ForgeError::DownloadError {
                url: pom_url.clone(),
                message: e.to_string(),
            })?;

        Ok(Self::parse_pom_dependencies(&pom_text))
    }

    /// Parsea un POM XML y extrae las dependencias con scope compile.
    fn parse_pom_dependencies(pom_xml: &str) -> Vec<MavenCoordinate> {
        let mut deps = Vec::new();
        let mut reader = quick_xml::Reader::from_str(pom_xml);
        reader.config_mut().trim_text(true);

        let mut in_dependencies = false;
        let mut in_dependency = false;
        let mut in_dep_mgmt = false;
        let mut current_group = String::new();
        let mut current_artifact = String::new();
        let mut current_version = String::new();
        let mut current_scope = String::new();
        let mut current_tag = String::new();

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "dependencyManagement" => in_dep_mgmt = true,
                        "dependencies" if !in_dep_mgmt => in_dependencies = true,
                        "dependency" if in_dependencies && !in_dep_mgmt => {
                            in_dependency = true;
                            current_group.clear();
                            current_artifact.clear();
                            current_version.clear();
                            current_scope.clear();
                        }
                        _ if in_dependency => {
                            current_tag = tag_name;
                        }
                        _ => {}
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "dependencyManagement" => in_dep_mgmt = false,
                        "dependencies" if !in_dep_mgmt => in_dependencies = false,
                        "dependency" if in_dependency => {
                            in_dependency = false;

                            // Solo incluir scope compile (o sin scope = compile por defecto)
                            let scope = if current_scope.is_empty() { "compile" } else { &current_scope };
                            
                            if scope == "compile"
                                && !current_group.is_empty()
                                && !current_artifact.is_empty()
                                && !current_version.is_empty()
                                && !current_version.starts_with('$')  // Ignorar variables ${...}
                            {
                                deps.push(MavenCoordinate {
                                    group_id: current_group.clone(),
                                    artifact_id: current_artifact.clone(),
                                    version: current_version.clone(),
                                });
                            }
                        }
                        _ => {
                            current_tag.clear();
                        }
                    }
                }
                Ok(quick_xml::events::Event::Text(ref e)) => {
                    if in_dependency {
                        let text = e.unescape().unwrap_or_default().to_string();
                        match current_tag.as_str() {
                            "groupId" => current_group = text,
                            "artifactId" => current_artifact = text,
                            "version" => current_version = text,
                            "scope" => current_scope = text,
                            _ => {}
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        deps
    }

    /// Descarga un JAR individual si no está en caché.
    async fn download_dependency(
        &mut self,
        coord: &MavenCoordinate,
        target_dir: &Path,
    ) -> ForgeResult<PathBuf> {
        let key = format!("{}:{}", target_dir.to_string_lossy(), coord.display());

        // Evitar resolver la misma dependencia dos veces
        if self.resolved.contains(&key) {
            return Ok(target_dir.join(coord.jar_filename()));
        }

        let jar_path = target_dir.join(coord.jar_filename());

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
