// =============================================================================
// 🔥 FORGE — Resolución de Dependencias: Maven Central
// =============================================================================
// Descarga JARs y resuelve dependencias transitivas desde Maven Central.
// =============================================================================

use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use colored::Colorize;
use sha1::Sha1;
use sha2::{Digest, Sha256};

use cyrce_forge_core::error::{ForgeError, ForgeResult};

/// URL base de Maven Central.
const MAVEN_CENTRAL_URL: &str = "https://repo.maven.apache.org/maven2";
const MAX_JAR_BYTES: u64 = 512 * 1024 * 1024;
const MAX_POM_BYTES: u64 = 4 * 1024 * 1024;
const MAX_CHECKSUM_BYTES: u64 = 1024;
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Coordenadas Maven (groupId:artifactId:version[:classifier]).
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct MavenCoordinate {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub classifier: Option<String>,
}

impl MavenCoordinate {
    /// Parsea una coordenada en formato "groupId:artifactId" (con :classifier opcional tras unirse la version).
    pub fn parse(key: &str, version: &str) -> ForgeResult<Self> {
        // En `forge.toml`, la dependency key suele ser "group:artifact" o "group:artifact:classifier"
        let parts: Vec<&str> = key.split(':').collect();

        let (group_id, artifact_id, classifier) = match parts.len() {
            2 => (parts[0].to_string(), parts[1].to_string(), None),
            3 => (parts[0].to_string(), parts[1].to_string(), Some(parts[2].to_string())),
            _ => return Err(ForgeError::DependencyResolutionFailed {
                dependency: format!(
                    "'{}' — Formato esperado: 'groupId:artifactId' o 'groupId:artifactId:classifier'",
                    key
                ),
            }.into()),
        };

        let coordinate = Self {
            group_id,
            artifact_id,
            version: version.to_string(),
            classifier,
        };
        coordinate.validate()?;
        Ok(coordinate)
    }

    /// Genera la URL del JAR en Maven Central.
    pub fn jar_url(&self) -> String {
        let classifier_suffix = match &self.classifier {
            Some(c) => format!("-{}", c),
            None => "".to_string(),
        };
        format!(
            "{}/{}/{}/{}/{}-{}{}.jar",
            MAVEN_CENTRAL_URL,
            self.group_id.replace('.', "/"),
            self.artifact_id,
            self.version,
            self.artifact_id,
            self.version,
            classifier_suffix
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
        match &self.classifier {
            Some(c) => format!("{}-{}-{}.jar", self.artifact_id, self.version, c),
            None => format!("{}-{}.jar", self.artifact_id, self.version),
        }
    }

    /// Representación legible.
    pub fn display(&self) -> String {
        match &self.classifier {
            Some(c) => format!(
                "{}:{}:{}:{}",
                self.group_id, self.artifact_id, self.version, c
            ),
            None => format!("{}:{}:{}", self.group_id, self.artifact_id, self.version),
        }
    }

    fn validate(&self) -> ForgeResult<()> {
        validate_group_id(&self.group_id)?;
        validate_coordinate_part("artifactId", &self.artifact_id)?;
        validate_coordinate_part("version", &self.version)?;
        if let Some(classifier) = &self.classifier {
            validate_coordinate_part("classifier", classifier)?;
        }
        Ok(())
    }
}

fn invalid_coordinate(label: &str, value: &str) -> anyhow::Error {
    ForgeError::DependencyResolutionFailed {
        dependency: format!("{} Maven inválido: '{}'", label, value),
    }
    .into()
}

fn validate_group_id(value: &str) -> ForgeResult<()> {
    if value.len() > 255
        || value.split('.').any(|segment| {
            segment.is_empty()
                || !segment
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        })
    {
        return Err(invalid_coordinate("groupId", value));
    }
    Ok(())
}

fn validate_coordinate_part(label: &str, value: &str) -> ForgeResult<()> {
    if value.is_empty()
        || value.len() > 255
        || value == "."
        || value == ".."
        || !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '+'))
    {
        return Err(invalid_coordinate(label, value));
    }
    Ok(())
}

/// Resuelve y descarga dependencias Maven.
pub struct MavenResolver {
    /// Cliente HTTP reutilizable
    client: reqwest::Client,
    /// Directorio local del proyecto (.forge/deps)
    cache_dir: PathBuf,
    /// Almacén Global CAS (~/.forge/repository/cas)
    global_cas_dir: PathBuf,
    /// Índice Global (~/.forge/repository/index/maven)
    global_index_dir: PathBuf,
    /// Dependencias ya resueltas (evitar ciclos)
    resolved: HashSet<String>,
}

impl MavenResolver {
    /// Crea un nuevo resolver.
    pub fn new(project_dir: &Path) -> Self {
        let cache_dir = project_dir.join(".forge").join("deps");

        let global_repo = dirs::home_dir()
            .map(|home| home.join(".forge").join("repository"))
            .unwrap_or_else(|| project_dir.join(".forge").join("global-repository"));
        let global_cas_dir = global_repo.join("cas");
        let global_index_dir = global_repo.join("index").join("maven");

        std::fs::create_dir_all(&global_cas_dir).ok();
        std::fs::create_dir_all(&global_index_dir).ok();

        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .redirect(reqwest::redirect::Policy::limited(5))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            cache_dir,
            global_cas_dir,
            global_index_dir,
            resolved: HashSet::new(),
        }
    }

    /// Carga el directorio en caché para dependencias de prueba
    pub fn test_cache_dir(&self) -> PathBuf {
        self.cache_dir.with_file_name("test-deps")
    }

    /// Resuelve y descarga todas las dependencias runtime a .forge/deps/.
    pub async fn resolve_all(
        &mut self,
        dependencies: &std::collections::HashMap<String, String>,
    ) -> ForgeResult<Vec<PathBuf>> {
        self.resolve_internal(dependencies, &self.cache_dir.clone())
            .await
    }

    /// Resuelve y descarga dependencias de prueba a .forge/test-deps/.
    pub async fn resolve_test_deps(
        &mut self,
        dependencies: &std::collections::HashMap<String, String>,
    ) -> ForgeResult<Vec<PathBuf>> {
        self.resolve_internal(dependencies, &self.test_cache_dir())
            .await
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
            self.resolve_recursive(&coord, target_dir, &mut downloaded, 0)
                .await?;
        }

        println!(
            "   {}",
            format!(
                "✅ {} dependencias resueltas (incluyendo transitivas)",
                downloaded.len()
            )
            .green()
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
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ForgeResult<()>> + Send + 'a>> {
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

            // Un POM ausente es válido, pero un error de red, tamaño o checksum debe propagarse.
            let transitive_deps = self.fetch_transitive_deps(coord).await?;
            for dep_coord in transitive_deps {
                self.resolve_recursive(&dep_coord, target_dir, downloaded, depth + 1)
                    .await?;
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

        let response =
            self.client
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

        if response
            .content_length()
            .is_some_and(|length| length > MAX_POM_BYTES)
        {
            return Err(ForgeError::DownloadError {
                url: pom_url,
                message: "El POM excede el límite de 4 MiB".to_string(),
            }
            .into());
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ForgeError::DownloadError {
                url: pom_url.clone(),
                message: e.to_string(),
            })?;
        if bytes.len() as u64 > MAX_POM_BYTES {
            return Err(ForgeError::DownloadError {
                url: pom_url,
                message: "El POM excede el límite de 4 MiB".to_string(),
            }
            .into());
        }
        let expected_hash = fetch_remote_checksum(&self.client, &pom_url).await?;
        verify_published_checksum(&bytes, &expected_hash, &pom_url)?;
        let pom_text = std::str::from_utf8(&bytes).map_err(|e| ForgeError::DownloadError {
            url: pom_url,
            message: format!("El POM no es UTF-8 válido: {}", e),
        })?;

        Ok(Self::parse_pom_dependencies(pom_text))
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
                            let scope = if current_scope.is_empty() {
                                "compile"
                            } else {
                                &current_scope
                            };

                            if scope == "compile"
                                && !current_group.is_empty()
                                && !current_artifact.is_empty()
                                && !current_version.is_empty()
                                && !current_version.starts_with('$')
                            // Ignorar variables ${...}
                            {
                                let coordinate = MavenCoordinate {
                                    group_id: current_group.clone(),
                                    artifact_id: current_artifact.clone(),
                                    version: current_version.clone(),
                                    classifier: None,
                                };
                                if coordinate.validate().is_ok() {
                                    deps.push(coordinate);
                                }
                            }
                        }
                        _ => {
                            current_tag.clear();
                        }
                    }
                }
                Ok(quick_xml::events::Event::Text(ref e)) => {
                    if in_dependency {
                        let Ok(decoded) = e.decode() else {
                            buf.clear();
                            continue;
                        };
                        let text = quick_xml::escape::unescape(&decoded)
                            .map(|value| value.into_owned())
                            .unwrap_or_else(|_| decoded.into_owned());
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

    /// Descarga un JAR individual si no está en caché global/local usando lógica CAS atómica 2026.
    async fn download_dependency(
        &mut self,
        coord: &MavenCoordinate,
        target_dir: &Path,
    ) -> ForgeResult<PathBuf> {
        coord.validate()?;
        let key = format!("{}:{}", target_dir.to_string_lossy(), coord.display());

        // Evitar resolver la misma dependencia dos veces
        if self.resolved.contains(&key) {
            return Ok(target_dir.join(coord.jar_filename()));
        }

        let jar_path = target_dir.join(coord.jar_filename());

        // 1. Si ya existe localmente (caché del proyecto)
        if jar_path.exists() && is_valid_jar(&jar_path) {
            self.resolved.insert(key);
            println!(
                "   {}",
                format!("   ⚡ {} (caché local)", coord.display()).dimmed()
            );
            return Ok(jar_path);
        }
        if jar_path.exists() {
            std::fs::remove_file(&jar_path).map_err(|e| ForgeError::IoError {
                path: jar_path.clone(),
                message: format!("No se pudo retirar el JAR local corrupto: {}", e),
            })?;
        }

        // 2. Comprobar si existe en el Índice Global (CAS central)
        let global_index_path = self
            .global_index_dir
            .join(coord.group_id.replace('.', "/"))
            .join(&coord.artifact_id)
            .join(&coord.version)
            .join(coord.jar_filename());

        if global_index_path.exists() && is_valid_jar(&global_index_path) {
            // "Zero-Copy": El archivo ya fue descargado previamente por otro proyecto.
            // Creamos un Enlace Duro desde el Cajón Local al Índice Global O(1)
            if std::fs::hard_link(&global_index_path, &jar_path).is_err() {
                // Fallback de seguridad (ej: particiones distintas)
                std::fs::copy(&global_index_path, &jar_path).map_err(|e| ForgeError::IoError {
                    path: jar_path.clone(),
                    message: format!("Hardlink falló; tampoco se pudo copiar: {}", e),
                })?;
            }

            self.resolved.insert(key);
            println!(
                "   {}",
                format!("   💎 {} (global CAS O(1))", coord.display()).bright_cyan()
            );
            return Ok(jar_path);
        }
        if global_index_path.exists() {
            std::fs::remove_file(&global_index_path).map_err(|e| ForgeError::IoError {
                path: global_index_path.clone(),
                message: format!("No se pudo retirar el índice Maven corrupto: {}", e),
            })?;
        }

        // 3. No existe ni local ni globalmente -> Toca descargar
        println!(
            "   {}",
            format!("   ⬇️  Descargando {}...", coord.display()).dimmed()
        );

        let url = coord.jar_url();
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
            return Err(ForgeError::DownloadError {
                url,
                message: format!("HTTP {}", response.status()),
            }
            .into());
        }

        if response
            .content_length()
            .is_some_and(|length| length > MAX_JAR_BYTES)
        {
            return Err(ForgeError::DownloadError {
                url,
                message: "El JAR excede el límite de 512 MiB".to_string(),
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

        if bytes.len() as u64 > MAX_JAR_BYTES || !bytes.starts_with(b"PK") {
            return Err(ForgeError::DownloadError {
                url,
                message: "La respuesta no es un JAR válido o excede 512 MiB".to_string(),
            }
            .into());
        }

        // 4. Verificar el checksum publicado por Maven Central antes de aceptar el artefacto.
        let expected_hash = fetch_remote_checksum(&self.client, &url).await?;
        verify_published_checksum(&bytes, &expected_hash, &url)?;
        let hash_hex = sha256_hex(&bytes);
        let cas_path = self.global_cas_dir.join(format!("{}.jar", hash_hex));

        // 5. Atomic Write (Evita corrupciones si se cancela abruptamente Ctrl+C)
        if !cas_path.exists() {
            let temp_path =
                self.global_cas_dir
                    .join(format!("{}-{}.tmp", hash_hex, unique_suffix()));
            std::fs::write(&temp_path, &bytes).map_err(|e| ForgeError::IoError {
                path: temp_path.clone(),
                message: e.to_string(),
            })?;
            if let Err(error) = std::fs::rename(&temp_path, &cas_path) {
                if cas_path.exists() {
                    let _ = std::fs::remove_file(&temp_path);
                } else {
                    return Err(ForgeError::IoError {
                        path: cas_path.clone(),
                        message: format!("No se pudo publicar el objeto CAS: {}", error),
                    }
                    .into());
                }
            }
        }

        // 6. Hardlink del Índice Global -> Almacen CAS Intangible
        if let Some(parent) = global_index_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ForgeError::IoError {
                path: parent.to_path_buf(),
                message: e.to_string(),
            })?;
        }
        if !global_index_path.exists() && std::fs::hard_link(&cas_path, &global_index_path).is_err()
        {
            std::fs::copy(&cas_path, &global_index_path).map_err(|e| ForgeError::IoError {
                path: global_index_path.clone(),
                message: format!("No se pudo crear el índice Maven: {}", e),
            })?;
        }

        // 7. Hardlink del Repositorio Local -> Índice Global
        if std::fs::hard_link(&global_index_path, &jar_path).is_err() {
            std::fs::copy(&global_index_path, &jar_path).map_err(|e| ForgeError::IoError {
                path: jar_path.clone(),
                message: format!("Hardlink falló; tampoco se pudo copiar: {}", e),
            })?;
        }

        self.resolved.insert(key);
        Ok(jar_path)
    }
}

fn is_valid_jar(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() || metadata.len() < 2 || metadata.len() > MAX_JAR_BYTES {
        return false;
    }

    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut magic = [0_u8; 2];
    file.read_exact(&mut magic).is_ok() && magic == *b"PK"
}

fn unique_suffix() -> String {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{}-{}", std::process::id(), nanos, counter)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PublishedChecksum {
    Sha256(String),
    Sha1(String),
}

async fn fetch_remote_checksum(
    client: &reqwest::Client,
    artifact_url: &str,
) -> ForgeResult<PublishedChecksum> {
    if let Some(value) = fetch_checksum_text(client, artifact_url, "sha256").await? {
        return parse_checksum(&value, 64)
            .map(PublishedChecksum::Sha256)
            .ok_or_else(|| invalid_checksum_error(artifact_url, "SHA-256"));
    }
    if let Some(value) = fetch_checksum_text(client, artifact_url, "sha1").await? {
        return parse_checksum(&value, 40)
            .map(PublishedChecksum::Sha1)
            .ok_or_else(|| invalid_checksum_error(artifact_url, "SHA-1"));
    }

    Err(ForgeError::DownloadError {
        url: artifact_url.to_string(),
        message: "Maven Central no publicó un checksum SHA-256 ni SHA-1".to_string(),
    }
    .into())
}

async fn fetch_checksum_text(
    client: &reqwest::Client,
    artifact_url: &str,
    extension: &str,
) -> ForgeResult<Option<String>> {
    let checksum_url = format!("{artifact_url}.{extension}");
    let response =
        client
            .get(&checksum_url)
            .send()
            .await
            .map_err(|e| ForgeError::DownloadError {
                url: checksum_url.clone(),
                message: e.to_string(),
            })?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(ForgeError::DownloadError {
            url: checksum_url,
            message: format!("No se pudo obtener el checksum: HTTP {}", response.status()),
        }
        .into());
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_CHECKSUM_BYTES)
    {
        return Err(ForgeError::DownloadError {
            url: checksum_url,
            message: "La respuesta SHA-256 excede 1 KiB".to_string(),
        }
        .into());
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| ForgeError::DownloadError {
            url: checksum_url.clone(),
            message: e.to_string(),
        })?;
    if bytes.len() as u64 > MAX_CHECKSUM_BYTES {
        return Err(ForgeError::DownloadError {
            url: checksum_url,
            message: "La respuesta SHA-256 excede 1 KiB".to_string(),
        }
        .into());
    }
    let text = std::str::from_utf8(&bytes).map_err(|e| ForgeError::DownloadError {
        url: checksum_url.clone(),
        message: format!("Checksum no UTF-8: {e}"),
    })?;
    Ok(Some(text.to_string()))
}

fn invalid_checksum_error(artifact_url: &str, algorithm: &str) -> anyhow::Error {
    ForgeError::DownloadError {
        url: artifact_url.to_string(),
        message: format!("Respuesta {algorithm} inválida"),
    }
    .into()
}

fn parse_checksum(value: &str, expected_length: usize) -> Option<String> {
    let checksum = value.split_whitespace().next()?;
    (checksum.len() == expected_length && checksum.chars().all(|c| c.is_ascii_hexdigit()))
        .then(|| checksum.to_ascii_lowercase())
}

fn verify_published_checksum(
    bytes: &[u8],
    expected: &PublishedChecksum,
    url: &str,
) -> ForgeResult<()> {
    let (algorithm, expected_value, actual) = match expected {
        PublishedChecksum::Sha256(value) => ("SHA-256", value, sha256_hex(bytes)),
        PublishedChecksum::Sha1(value) => {
            let mut hasher = Sha1::new();
            hasher.update(bytes);
            ("SHA-1", value, format!("{:x}", hasher.finalize()))
        }
    };
    if &actual != expected_value {
        return Err(ForgeError::DownloadError {
            url: url.to_string(),
            message: format!(
                "Checksum {algorithm} no coincide (esperado {expected_value}, recibido {actual})"
            ),
        }
        .into());
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal_in_coordinates() {
        assert!(MavenCoordinate::parse("org.example:../escape", "1.0.0").is_err());
        assert!(MavenCoordinate::parse("org.example:demo:../../escape", "1.0.0").is_err());
        assert!(MavenCoordinate::parse("org.example:demo", "../1.0").is_err());
    }

    #[test]
    fn accepts_classifier_coordinates() {
        let coordinate = MavenCoordinate::parse("org.openjfx:javafx-controls:win", "21.0.2")
            .expect("classifier válido");
        assert_eq!(coordinate.jar_filename(), "javafx-controls-21.0.2-win.jar");
    }

    #[test]
    fn parses_and_verifies_sha256() {
        let expected = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert_eq!(
            parse_checksum(&format!("{expected}  artifact.jar"), 64),
            Some(expected.to_string())
        );
        let published = PublishedChecksum::Sha256(expected.to_string());
        assert!(
            verify_published_checksum(b"abc", &published, "https://example.invalid/a.jar").is_ok()
        );
        assert!(verify_published_checksum(
            b"tampered",
            &published,
            "https://example.invalid/a.jar"
        )
        .is_err());
        assert!(parse_checksum("not-a-checksum", 64).is_none());

        let legacy =
            PublishedChecksum::Sha1("a9993e364706816aba3e25717850c26c9cd0d89d".to_string());
        assert!(
            verify_published_checksum(b"abc", &legacy, "https://example.invalid/a.jar").is_ok()
        );
    }
}
