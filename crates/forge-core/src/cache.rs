// =============================================================================
// 🔥 FORGE — Motor Core: Caché Incremental
// =============================================================================
// Evita re-compilar archivos que no han cambiado usando hashes SHA-256.
// Almacena estado en .forge/cache.json dentro del proyecto.
// =============================================================================

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use colored::Colorize;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tar::{Archive, Builder};
use walkdir::WalkDir;

use crate::config::RemoteCacheConfig;
use crate::error::{ForgeError, ForgeResult};

const MAX_REMOTE_CACHE_BYTES: u64 = 512 * 1024 * 1024;
const MAX_REMOTE_CACHE_UNPACKED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_REMOTE_CACHE_ENTRIES: usize = 100_000;
const HTTP_TIMEOUT: Duration = Duration::from_secs(120);

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Estado de caché del build, persiste entre ejecuciones.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildCache {
    /// Versión del formato de caché
    pub version: u32,

    /// Mapa de ruta de archivo -> hash SHA-256 del contenido
    pub file_hashes: HashMap<String, String>,

    /// Timestamp de la última ejecución exitosa
    pub last_build_timestamp: Option<u64>,

    /// Huella de forge.toml, versión de FORGE, plataforma y perfil de build.
    /// Evita reutilizar artefactos cuando cambia la configuración y no el código.
    #[serde(default)]
    pub build_fingerprint: Option<String>,
}

impl BuildCache {
    /// Carga la caché desde .forge/cache.json, o crea una nueva si no existe.
    pub fn load(project_dir: &Path) -> ForgeResult<Self> {
        let cache_path = Self::cache_path(project_dir);

        if !cache_path.exists() {
            return Ok(Self {
                version: 1,
                ..Default::default()
            });
        }

        let content = std::fs::read_to_string(&cache_path).map_err(|e| ForgeError::IoError {
            path: cache_path.clone(),
            message: e.to_string(),
        })?;

        serde_json::from_str(&content)
            .map_err(|_| ForgeError::CacheCorrupted { path: cache_path }.into())
    }

    /// Guarda la caché en .forge/cache.json.
    pub fn save(&self, project_dir: &Path) -> ForgeResult<()> {
        let forge_dir = project_dir.join(".forge");
        std::fs::create_dir_all(&forge_dir).map_err(|e| ForgeError::IoError {
            path: forge_dir.clone(),
            message: e.to_string(),
        })?;

        let cache_path = Self::cache_path(project_dir);
        let content = serde_json::to_string_pretty(self).map_err(|e| ForgeError::IoError {
            path: cache_path.clone(),
            message: e.to_string(),
        })?;

        let temp_path = forge_dir.join(format!("cache-{}.tmp", unique_suffix()));
        std::fs::write(&temp_path, content).map_err(|e| ForgeError::IoError {
            path: temp_path.clone(),
            message: e.to_string(),
        })?;
        replace_file(&temp_path, &cache_path)?;

        Ok(())
    }

    /// Verifica si algún archivo en el directorio fuente ha cambiado.
    /// Devuelve true si hay cambios (necesita recompilar).
    pub fn has_changes(&self, source_dir: &Path, extensions: &[&str]) -> ForgeResult<bool> {
        let current_hashes = Self::compute_hashes(source_dir, extensions)?;

        // Comparar con hashes guardados
        for (path, hash) in &current_hashes {
            match self.file_hashes.get(path) {
                Some(old_hash) if old_hash == hash => continue,
                _ => return Ok(true), // Archivo nuevo o modificado
            }
        }

        // Verificar archivos eliminados
        for old_path in self.file_hashes.keys() {
            if !current_hashes.contains_key(old_path) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Verifica cambios de fuentes y del contexto completo del build.
    pub fn has_build_changes(
        &self,
        source_dir: &Path,
        extensions: &[&str],
        fingerprint: &str,
    ) -> ForgeResult<bool> {
        if self.build_fingerprint.as_deref() != Some(fingerprint) {
            return Ok(true);
        }
        self.has_changes(source_dir, extensions)
    }

    /// Actualiza los hashes con el estado actual del directorio fuente.
    pub fn update_hashes(&mut self, source_dir: &Path, extensions: &[&str]) -> ForgeResult<()> {
        self.file_hashes = Self::compute_hashes(source_dir, extensions)?;
        self.last_build_timestamp = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        Ok(())
    }

    /// Actualiza fuentes y contexto después de una compilación exitosa.
    pub fn update_build_state(
        &mut self,
        source_dir: &Path,
        extensions: &[&str],
        fingerprint: String,
    ) -> ForgeResult<()> {
        self.update_hashes(source_dir, extensions)?;
        self.build_fingerprint = Some(fingerprint);
        Ok(())
    }

    /// Calcula una huella estable para invalidar cachés al cambiar forge.toml,
    /// la versión del motor, la plataforma o el perfil solicitado.
    pub fn compute_build_fingerprint(project_dir: &Path, profile: &str) -> ForgeResult<String> {
        let config_path = project_dir.join("forge.toml");
        let config = std::fs::read(&config_path).map_err(|e| ForgeError::IoError {
            path: config_path,
            message: e.to_string(),
        })?;

        let mut hasher = Sha256::new();
        hasher.update(b"forge-build-fingerprint-v1\0");
        hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
        hasher.update(b"\0");
        hasher.update(std::env::consts::OS.as_bytes());
        hasher.update(b"\0");
        hasher.update(std::env::consts::ARCH.as_bytes());
        hasher.update(b"\0");
        hasher.update(profile.as_bytes());
        hasher.update(b"\0");
        hasher.update(&config);

        let parsed =
            crate::config::ForgeConfig::parse(std::str::from_utf8(&config).map_err(|e| {
                ForgeError::ConfigParseError {
                    message: format!("forge.toml no es UTF-8 válido: {}", e),
                }
            })?)?;
        let mut visited = HashSet::new();
        hash_local_dependency_inputs(&mut hasher, &parsed, project_dir, &mut visited, 0)?;
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Devuelve los archivos que han cambiado desde el último build.
    pub fn changed_files(
        &self,
        source_dir: &Path,
        extensions: &[&str],
    ) -> ForgeResult<Vec<PathBuf>> {
        let current_hashes = Self::compute_hashes(source_dir, extensions)?;
        let mut changed = Vec::new();

        for (path, hash) in &current_hashes {
            match self.file_hashes.get(path) {
                Some(old_hash) if old_hash == hash => continue,
                _ => changed.push(PathBuf::from(path)),
            }
        }

        Ok(changed)
    }

    /// Limpia toda la caché.
    pub fn clean(project_dir: &Path) -> ForgeResult<()> {
        let forge_dir = project_dir.join(".forge");
        if forge_dir.exists() {
            std::fs::remove_dir_all(&forge_dir).map_err(|e| ForgeError::IoError {
                path: forge_dir,
                message: e.to_string(),
            })?;
        }
        Ok(())
    }

    /// Comprime un directorio de caché local (output) y lo sube al servidor remoto
    pub async fn upload_to_remote(
        &self,
        project_dir: &Path,
        output_dir_name: &str,
        remote_config: &RemoteCacheConfig,
    ) -> ForgeResult<()> {
        if !remote_config.push {
            return Ok(());
        }

        // 1. Calcular el hash maestro (representando el estado global de dependencias/ficheros base del proyecto)
        let master_hash = self.compute_master_hash()?;
        let archive_name = format!("{}.tar.gz", master_hash);
        let remote_url = format!(
            "{}/cache/{}",
            remote_config.remote.trim_end_matches('/'),
            archive_name
        );

        println!(
            "   {} Subiendo build al caché distribuido ({})",
            "⬆️".cyan(),
            master_hash
        );

        // 2. Comprimir el directorio de salida en un buffer en memoria o disco
        let output_path = project_dir.join(output_dir_name);
        if !output_path.exists() {
            return Ok(());
        }

        let tar_gz_path = std::env::temp_dir().join(format!(
            "forge-cache-upload-{}-{}.tar.gz",
            master_hash,
            unique_suffix()
        ));
        let tar_gz_file = File::create(&tar_gz_path).map_err(|e| ForgeError::IoError {
            path: tar_gz_path.clone(),
            message: e.to_string(),
        })?;

        let enc = GzEncoder::new(tar_gz_file, Compression::default());
        let mut tar = Builder::new(enc);
        tar.append_dir_all(".", &output_path)
            .map_err(|e| ForgeError::IoError {
                path: output_path.clone(),
                message: format!("Error al comprimir caché: {}", e),
            })?;
        let enc = tar.into_inner().map_err(|e| ForgeError::IoError {
            path: output_path.clone(),
            message: format!("Error interno al procesar tar: {}", e),
        })?;
        enc.finish().map_err(|e| ForgeError::IoError {
            path: output_path.clone(),
            message: format!("Error interno al finalizar tar.gz: {}", e),
        })?;

        // 3. Subir vía HTTP PUT
        let client = http_client()?;
        let mut req = client.put(&remote_url);
        if let Some(token) = &remote_config.token {
            req = req.bearer_auth(token);
        }

        let file_bytes = std::fs::read(&tar_gz_path).map_err(|e| ForgeError::IoError {
            path: tar_gz_path.clone(),
            message: format!("Error leyendo gzip temporal: {}", e),
        })?;
        let res: Result<reqwest::Response, reqwest::Error> = req.body(file_bytes).send().await;
        let _ = std::fs::remove_file(&tar_gz_path); // Limpiar tmp local

        match res {
            Ok(resp) if resp.status().is_success() => {
                println!("   {} Caché remoto actualizado exitosamente", "✅".green());
                Ok(())
            }
            Ok(resp) => {
                eprintln!(
                    "   {} Fallo al subir caché ({})",
                    "⚠️".yellow(),
                    resp.status()
                );
                Ok(()) // No es fatal
            }
            Err(e) => {
                eprintln!("   {} Fallo red al subir caché: {}", "⚠️".yellow(), e);
                Ok(())
            }
        }
    }

    /// Intenta descargar un caché pre-compilado desde el servidor remoto
    pub async fn download_from_remote(
        &self,
        project_dir: &Path,
        output_dir_name: &str,
        remote_config: &RemoteCacheConfig,
    ) -> ForgeResult<bool> {
        let master_hash = self.compute_master_hash()?;
        let archive_name = format!("{}.tar.gz", master_hash);
        let remote_url = format!(
            "{}/cache/{}",
            remote_config.remote.trim_end_matches('/'),
            archive_name
        );

        let client = http_client()?;
        let mut req = client.get(&remote_url);
        if let Some(token) = &remote_config.token {
            req = req.bearer_auth(token);
        }

        let res: Result<reqwest::Response, reqwest::Error> = req.send().await;
        match res {
            Ok(mut resp) if resp.status().is_success() => {
                println!(
                    "   {} Caché distribuido encontrado ({})",
                    "☁️".cyan(),
                    master_hash
                );

                if resp
                    .content_length()
                    .is_some_and(|size| size > MAX_REMOTE_CACHE_BYTES)
                {
                    eprintln!(
                        "   {} El caché remoto excede el límite de 512 MiB",
                        "⚠️".yellow()
                    );
                    return Ok(false);
                }

                let output_path = safe_output_path(project_dir, output_dir_name)?;
                let forge_dir = project_dir.join(".forge");
                std::fs::create_dir_all(&forge_dir).map_err(|e| ForgeError::IoError {
                    path: forge_dir.clone(),
                    message: e.to_string(),
                })?;

                let suffix = unique_suffix();
                let archive_path = forge_dir.join(format!("remote-cache-{}.tar.gz", suffix));
                let staging_path = forge_dir.join(format!("remote-cache-staging-{}", suffix));
                let mut archive_file =
                    File::create(&archive_path).map_err(|e| ForgeError::IoError {
                        path: archive_path.clone(),
                        message: e.to_string(),
                    })?;

                let mut downloaded = 0_u64;
                loop {
                    match resp.chunk().await {
                        Ok(Some(chunk)) => {
                            downloaded = downloaded.saturating_add(chunk.len() as u64);
                            if downloaded > MAX_REMOTE_CACHE_BYTES {
                                drop(archive_file);
                                let _ = std::fs::remove_file(&archive_path);
                                eprintln!(
                                    "   {} El caché remoto excede el límite de 512 MiB",
                                    "⚠️".yellow()
                                );
                                return Ok(false);
                            }
                            archive_file
                                .write_all(&chunk)
                                .map_err(|e| ForgeError::IoError {
                                    path: archive_path.clone(),
                                    message: e.to_string(),
                                })?;
                        }
                        Ok(None) => break,
                        Err(e) => {
                            drop(archive_file);
                            let _ = std::fs::remove_file(&archive_path);
                            eprintln!(
                                "   {} Error descargando cuerpo del caché: {}",
                                "⚠️".yellow(),
                                e
                            );
                            return Ok(false);
                        }
                    }
                }
                drop(archive_file);

                if let Err(e) = extract_cache_archive(&archive_path, &staging_path) {
                    let _ = std::fs::remove_file(&archive_path);
                    let _ = std::fs::remove_dir_all(&staging_path);
                    eprintln!("   {} Caché remoto rechazado: {}", "⚠️".yellow(), e);
                    return Ok(false);
                }
                let _ = std::fs::remove_file(&archive_path);

                if let Err(e) = replace_directory(&staging_path, &output_path) {
                    let _ = std::fs::remove_dir_all(&staging_path);
                    eprintln!(
                        "   {} No se pudo activar el caché restaurado: {}",
                        "⚠️".yellow(),
                        e
                    );
                    return Ok(false);
                }

                println!(
                    "   {} Caché remoto restaurado en {}",
                    "⚡".green(),
                    output_dir_name
                );
                Ok(true)
            }
            _ => {
                // Not found o error ("Miss")
                Ok(false)
            }
        }
    }

    /// Combina los file_hashes para generar un único hash que defina el estado global del código actual
    pub fn compute_master_hash(&self) -> ForgeResult<String> {
        let mut hasher = Sha256::new();
        let mut sorted_keys: Vec<&String> = self.file_hashes.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            if let Some(hash) = self.file_hashes.get(key) {
                hasher.update(key.as_bytes());
                hasher.update(hash.as_bytes());
            }
        }

        if let Some(fingerprint) = &self.build_fingerprint {
            hasher.update(b"\0build-fingerprint\0");
            hasher.update(fingerprint.as_bytes());
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Ruta del archivo de caché.
    fn cache_path(project_dir: &Path) -> PathBuf {
        project_dir.join(".forge").join("cache.json")
    }

    /// Calcula hashes SHA-256 de todos los archivos con las extensiones dadas.
    fn compute_hashes(
        source_dir: &Path,
        extensions: &[&str],
    ) -> ForgeResult<HashMap<String, String>> {
        let mut hashes = HashMap::new();

        if !source_dir.exists() {
            return Ok(hashes);
        }

        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();

            // Filtrar por extensión
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            if !extensions.is_empty() && !extensions.contains(&ext) {
                continue;
            }

            // Calcular hash SHA-256
            let content = std::fs::read(path).map_err(|e| ForgeError::IoError {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;

            let mut hasher = Sha256::new();
            hasher.update(&content);
            let hash = format!("{:x}", hasher.finalize());

            // Usar ruta relativa como clave
            let relative = path
                .strip_prefix(source_dir)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            hashes.insert(relative, hash);
        }

        Ok(hashes)
    }
}

fn http_client() -> ForgeResult<Client> {
    Client::builder()
        .timeout(HTTP_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| {
            ForgeError::DownloadError {
                url: "configuración del cliente HTTP".to_string(),
                message: e.to_string(),
            }
            .into()
        })
}

fn hash_local_dependency_inputs(
    hasher: &mut Sha256,
    config: &crate::config::ForgeConfig,
    project_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
) -> ForgeResult<()> {
    if depth > 32 {
        return Err(ForgeError::CyclicDependency {
            cycle: "más de 32 niveles de dependencias locales".to_string(),
        }
        .into());
    }

    let mut local_paths: Vec<&str> = config
        .dependencies
        .values()
        .chain(config.test_dependencies.values())
        .filter_map(|value| value.strip_prefix("path:"))
        .collect();
    local_paths.sort_unstable();

    for relative in local_paths {
        let dep_dir = project_dir.join(relative);
        let identity = std::fs::canonicalize(&dep_dir).unwrap_or_else(|_| dep_dir.clone());
        if !visited.insert(identity) {
            continue;
        }

        hasher.update(b"\0local-dependency\0");
        hasher.update(relative.as_bytes());

        let dep_config_path = dep_dir.join("forge.toml");
        if dep_config_path.is_file() {
            let dep_config_bytes =
                std::fs::read(&dep_config_path).map_err(|e| ForgeError::IoError {
                    path: dep_config_path.clone(),
                    message: e.to_string(),
                })?;
            hasher.update(&dep_config_bytes);
            let dep_config = crate::config::ForgeConfig::parse(
                std::str::from_utf8(&dep_config_bytes).map_err(|e| {
                    ForgeError::ConfigParseError {
                        message: format!("{} no es UTF-8 válido: {}", dep_config_path.display(), e),
                    }
                })?,
            )?;
            hash_tree(hasher, &dep_dir.join(dep_config.source_dir()))?;
            hash_local_dependency_inputs(hasher, &dep_config, &dep_dir, visited, depth + 1)?;
        } else {
            // Compatibilidad con proyectos Gradle/Maven ya compilados.
            for output in [
                dep_dir.join("build").join("libs"),
                dep_dir.join("build").join("classes"),
                dep_dir.join("target").join("classes"),
                dep_dir.join("out").join("production").join("classes"),
            ] {
                hash_tree(hasher, &output)?;
            }
        }
    }

    Ok(())
}

fn hash_tree(hasher: &mut Sha256, root: &Path) -> ForgeResult<()> {
    if !root.exists() {
        hasher.update(b"\0missing\0");
        hasher.update(root.to_string_lossy().as_bytes());
        return Ok(());
    }

    let mut files: Vec<PathBuf> = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect();
    files.sort();

    for path in files {
        let relative = path.strip_prefix(root).unwrap_or(&path);
        let content = std::fs::read(&path).map_err(|e| ForgeError::IoError {
            path: path.clone(),
            message: e.to_string(),
        })?;
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update(b"\0");
        hasher.update(content);
    }
    Ok(())
}

fn unique_suffix() -> String {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{}-{}-{}", std::process::id(), nanos, counter)
}

fn safe_output_path(project_dir: &Path, output_dir_name: &str) -> ForgeResult<PathBuf> {
    let relative = Path::new(output_dir_name);
    if output_dir_name.trim().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(ForgeError::ConfigParseError {
            message: format!(
                "project.output_dir debe permanecer dentro del proyecto: '{}'",
                output_dir_name
            ),
        }
        .into());
    }
    Ok(project_dir.join(relative))
}

fn extract_cache_archive(archive_path: &Path, staging_path: &Path) -> ForgeResult<()> {
    std::fs::create_dir_all(staging_path).map_err(|e| ForgeError::IoError {
        path: staging_path.to_path_buf(),
        message: e.to_string(),
    })?;

    let file = File::open(archive_path).map_err(|e| ForgeError::IoError {
        path: archive_path.to_path_buf(),
        message: e.to_string(),
    })?;
    let mut archive = Archive::new(GzDecoder::new(file));
    let entries = archive.entries().map_err(|e| ForgeError::IoError {
        path: archive_path.to_path_buf(),
        message: format!("Archivo de caché inválido: {}", e),
    })?;

    let mut total_size = 0_u64;
    for (index, entry) in entries.enumerate() {
        if index >= MAX_REMOTE_CACHE_ENTRIES {
            return Err(ForgeError::CacheCorrupted {
                path: archive_path.to_path_buf(),
            }
            .into());
        }

        let mut entry = entry.map_err(|_| ForgeError::CacheCorrupted {
            path: archive_path.to_path_buf(),
        })?;
        let entry_type = entry.header().entry_type();
        if !(entry_type.is_file() || entry_type.is_dir()) {
            return Err(ForgeError::CacheCorrupted {
                path: archive_path.to_path_buf(),
            }
            .into());
        }

        total_size = total_size
            .checked_add(entry.size())
            .filter(|size| *size <= MAX_REMOTE_CACHE_UNPACKED_BYTES)
            .ok_or_else(|| ForgeError::CacheCorrupted {
                path: archive_path.to_path_buf(),
            })?;

        let entry_path = entry.path().map_err(|_| ForgeError::CacheCorrupted {
            path: archive_path.to_path_buf(),
        })?;
        if entry_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
        {
            return Err(ForgeError::CacheCorrupted {
                path: archive_path.to_path_buf(),
            }
            .into());
        }

        let unpacked = entry
            .unpack_in(staging_path)
            .map_err(|_| ForgeError::CacheCorrupted {
                path: archive_path.to_path_buf(),
            })?;
        if !unpacked {
            return Err(ForgeError::CacheCorrupted {
                path: archive_path.to_path_buf(),
            }
            .into());
        }
    }

    Ok(())
}

fn replace_file(source: &Path, destination: &Path) -> ForgeResult<()> {
    let backup = destination.with_extension(format!("bak-{}", unique_suffix()));
    let had_destination = destination.exists();
    if had_destination {
        std::fs::rename(destination, &backup).map_err(|e| ForgeError::IoError {
            path: destination.to_path_buf(),
            message: e.to_string(),
        })?;
    }

    if let Err(error) = std::fs::rename(source, destination) {
        if had_destination {
            let _ = std::fs::rename(&backup, destination);
        }
        return Err(ForgeError::IoError {
            path: destination.to_path_buf(),
            message: error.to_string(),
        }
        .into());
    }

    if had_destination {
        let _ = std::fs::remove_file(backup);
    }
    Ok(())
}

fn replace_directory(source: &Path, destination: &Path) -> ForgeResult<()> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ForgeError::IoError {
            path: parent.to_path_buf(),
            message: e.to_string(),
        })?;
    }

    let backup = destination.with_extension(format!("bak-{}", unique_suffix()));
    let had_destination = destination.exists();
    if had_destination {
        std::fs::rename(destination, &backup).map_err(|e| ForgeError::IoError {
            path: destination.to_path_buf(),
            message: e.to_string(),
        })?;
    }

    if let Err(error) = std::fs::rename(source, destination) {
        if had_destination {
            let _ = std::fs::rename(&backup, destination);
        }
        return Err(ForgeError::IoError {
            path: destination.to_path_buf(),
            message: error.to_string(),
        }
        .into());
    }

    if had_destination {
        let _ = std::fs::remove_dir_all(backup);
    }
    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_cache_empty() {
        let cache = BuildCache::default();
        assert!(cache.file_hashes.is_empty());
        assert_eq!(cache.version, 0);
    }

    #[test]
    fn test_compute_hashes() {
        let temp_dir = std::env::temp_dir().join("forge_test_cache");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Crear archivo de prueba
        fs::write(temp_dir.join("test.java"), "public class Test {}").unwrap();
        fs::write(temp_dir.join("other.txt"), "ignorar").unwrap();

        let hashes = BuildCache::compute_hashes(&temp_dir, &["java"]).unwrap();

        assert_eq!(hashes.len(), 1);
        assert!(hashes.contains_key("test.java"));

        // Limpiar
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_changes() {
        let temp_dir = std::env::temp_dir().join("forge_test_changes");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        fs::write(temp_dir.join("Main.java"), "class Main {}").unwrap();

        let mut cache = BuildCache {
            version: 1,
            ..Default::default()
        };

        // Primera vez: hay cambios (caché vacía)
        assert!(cache.has_changes(&temp_dir, &["java"]).unwrap());

        // Actualizar caché
        cache.update_hashes(&temp_dir, &["java"]).unwrap();

        // Ahora no hay cambios
        assert!(!cache.has_changes(&temp_dir, &["java"]).unwrap());

        // Modificar archivo
        fs::write(temp_dir.join("Main.java"), "class Main { int x; }").unwrap();

        // Ahora sí hay cambios
        assert!(cache.has_changes(&temp_dir, &["java"]).unwrap());

        // Limpiar
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn build_fingerprint_changes_with_configuration() {
        let temp_dir =
            std::env::temp_dir().join(format!("forge_test_fingerprint_{}", unique_suffix()));
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(
            temp_dir.join("forge.toml"),
            "[project]\nname = \"demo\"\nlang = \"java\"\n",
        )
        .unwrap();

        let first = BuildCache::compute_build_fingerprint(&temp_dir, "debug").unwrap();
        fs::write(
            temp_dir.join("forge.toml"),
            "[project]\nname = \"demo\"\nlang = \"java\"\noutput_dir = \"dist\"\n",
        )
        .unwrap();
        let second = BuildCache::compute_build_fingerprint(&temp_dir, "debug").unwrap();

        assert_ne!(first, second);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn build_fingerprint_tracks_local_dependency_sources() {
        let temp_dir =
            std::env::temp_dir().join(format!("forge_test_local_dep_{}", unique_suffix()));
        let dependency_dir = temp_dir.join("dependency");
        fs::create_dir_all(dependency_dir.join("src/main/java")).unwrap();
        fs::write(
            temp_dir.join("forge.toml"),
            "[project]\nname = \"app\"\nlang = \"java\"\n\n[dependencies]\n\"local:dependency\" = \"path:dependency\"\n",
        )
        .unwrap();
        fs::write(
            dependency_dir.join("forge.toml"),
            "[project]\nname = \"dependency\"\nlang = \"java\"\n",
        )
        .unwrap();
        let source = dependency_dir.join("src/main/java/Dependency.java");
        fs::write(&source, "class Dependency {}").unwrap();

        let first = BuildCache::compute_build_fingerprint(&temp_dir, "debug").unwrap();
        fs::write(&source, "class Dependency { int changed; }").unwrap();
        let second = BuildCache::compute_build_fingerprint(&temp_dir, "debug").unwrap();

        assert_ne!(first, second);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn rejects_unsafe_output_paths() {
        let project = Path::new("project");
        assert!(safe_output_path(project, "../outside").is_err());
        assert!(safe_output_path(project, "build/output").is_ok());
    }

    #[test]
    fn rejects_links_in_remote_cache_archives() {
        let temp_dir = std::env::temp_dir().join(format!("forge_test_archive_{}", unique_suffix()));
        fs::create_dir_all(&temp_dir).unwrap();
        let archive_path = temp_dir.join("cache.tar.gz");
        let staging_path = temp_dir.join("staging");

        let file = File::create(&archive_path).unwrap();
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(encoder);
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        header.set_mode(0o777);
        header.set_link_name("../outside").unwrap();
        header.set_cksum();
        builder
            .append_data(&mut header, "unsafe-link", std::io::empty())
            .unwrap();
        let encoder = builder.into_inner().unwrap();
        encoder.finish().unwrap();

        assert!(extract_cache_archive(&archive_path, &staging_path).is_err());
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
