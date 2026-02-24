// =============================================================================
// 🔥 FORGE — Motor Core: Caché Incremental
// =============================================================================
// Evita re-compilar archivos que no han cambiado usando hashes SHA-256.
// Almacena estado en .forge/cache.json dentro del proyecto.
// =============================================================================

use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use colored::Colorize;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tar::{Archive, Builder};
use walkdir::WalkDir;

use crate::config::RemoteCacheConfig;
use crate::error::{ForgeError, ForgeResult};

/// Estado de caché del build, persiste entre ejecuciones.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildCache {
    /// Versión del formato de caché
    pub version: u32,

    /// Mapa de ruta de archivo -> hash SHA-256 del contenido
    pub file_hashes: HashMap<String, String>,

    /// Timestamp de la última ejecución exitosa
    pub last_build_timestamp: Option<u64>,
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

        serde_json::from_str(&content).map_err(|_| ForgeError::CacheCorrupted {
            path: cache_path,
        }.into())
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

        std::fs::write(&cache_path, content).map_err(|e| ForgeError::IoError {
            path: cache_path,
            message: e.to_string(),
        })?;

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

    /// Devuelve los archivos que han cambiado desde el último build.
    pub fn changed_files(&self, source_dir: &Path, extensions: &[&str]) -> ForgeResult<Vec<PathBuf>> {
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
        let remote_url = format!("{}/cache/{}", remote_config.remote.trim_end_matches('/'), archive_name);

        println!("   {} Subiendo build al caché distribuido ({})", "⬆️".cyan(), master_hash);

        // 2. Comprimir el directorio de salida en un buffer en memoria o disco
        let output_path = project_dir.join(output_dir_name);
        if !output_path.exists() {
            return Ok(());
        }

        let tar_gz_path = std::env::temp_dir().join(&archive_name);
        let tar_gz_file = File::create(&tar_gz_path).map_err(|e| ForgeError::IoError {
            path: tar_gz_path.clone(),
            message: e.to_string(),
        })?;

        let enc = GzEncoder::new(tar_gz_file, Compression::default());
        let mut tar = Builder::new(enc);
        tar.append_dir_all(".", &output_path).map_err(|e| ForgeError::IoError {
            path: output_path.clone(),
            message: format!("Error al comprimir caché: {}", e),
        })?;
        tar.into_inner().unwrap().finish().unwrap();

        // 3. Subir vía HTTP PUT
        let client = Client::new();
        let mut req = client.put(&remote_url);
        if let Some(token) = &remote_config.token {
            req = req.bearer_auth(token);
        }

        let file_bytes = std::fs::read(&tar_gz_path).unwrap();
        let res: Result<reqwest::Response, reqwest::Error> = req.body(file_bytes).send().await;
        let _ = std::fs::remove_file(&tar_gz_path); // Limpiar tmp local

        match res {
            Ok(resp) if resp.status().is_success() => {
                println!("   {} Caché remoto actualizado exitosamente", "✅".green());
                Ok(())
            }
            Ok(resp) => {
                eprintln!("   {} Fallo al subir caché ({})", "⚠️".yellow(), resp.status());
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
        let remote_url = format!("{}/cache/{}", remote_config.remote.trim_end_matches('/'), archive_name);

        let client = Client::new();
        let mut req = client.get(&remote_url);
        if let Some(token) = &remote_config.token {
            req = req.bearer_auth(token);
        }

        let res: Result<reqwest::Response, reqwest::Error> = req.send().await;
        match res {
            Ok(resp) if resp.status().is_success() => {
                println!("   {} Caché distribuido encontrado ({})", "☁️".cyan(), master_hash);
                
                let bytes = resp.bytes().await.unwrap();
                
                // Extraer
                let output_path = project_dir.join(output_dir_name);
                if output_path.exists() {
                     let _ = std::fs::remove_dir_all(&output_path);
                }
                std::fs::create_dir_all(&output_path).unwrap();

                let tar_gz = std::io::Cursor::new(bytes);
                let tar = GzDecoder::new(tar_gz);
                let mut archive = Archive::new(tar);
                
                if let Err(e) = archive.unpack(&output_path) {
                    eprintln!("   {} Error extrayendo caché: {}", "⚠️".yellow(), e);
                    return Ok(false);
                }

                println!("   {} Caché remoto restaurado en {}", "⚡".green(), output_dir_name);
                return Ok(true);
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
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

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
}
