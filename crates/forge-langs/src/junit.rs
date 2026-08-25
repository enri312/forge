use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use cyrce_forge_core::error::{ForgeError, ForgeResult};
use sha2::{Digest, Sha256};

const JUNIT_VERSION: &str = "1.12.0";
const JUNIT_SHA256: &str = "7f66b9410172c0a330e3e5762e534aca8161399671aee311bc60cbd18a53b32d";
const MAX_JUNIT_JAR_BYTES: u64 = 128 * 1024 * 1024;

pub(crate) async fn download_junit_standalone() -> ForgeResult<PathBuf> {
    let tools_dir = dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".forge")
        .join("tools");
    std::fs::create_dir_all(&tools_dir).map_err(|e| ForgeError::IoError {
        path: tools_dir.clone(),
        message: e.to_string(),
    })?;

    let jar_name = format!("junit-platform-console-standalone-{JUNIT_VERSION}.jar");
    let jar_path = tools_dir.join(jar_name);
    if jar_path.exists() {
        if is_expected_junit_jar(&jar_path) {
            return Ok(jar_path);
        }
        std::fs::remove_file(&jar_path).map_err(|e| ForgeError::IoError {
            path: jar_path.clone(),
            message: format!("No se pudo retirar el JAR de JUnit corrupto: {e}"),
        })?;
    }

    let url = format!(
        "https://repo.maven.apache.org/maven2/org/junit/platform/junit-platform-console-standalone/{JUNIT_VERSION}/junit-platform-console-standalone-{JUNIT_VERSION}.jar"
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| ForgeError::DownloadError {
            url: url.clone(),
            message: e.to_string(),
        })?;
    let mut response = client
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
        .is_some_and(|size| size > MAX_JUNIT_JAR_BYTES)
    {
        return Err(ForgeError::DownloadError {
            url,
            message: "El JAR de JUnit excede el límite de 128 MiB".to_string(),
        }
        .into());
    }

    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| ForgeError::DownloadError {
            url: url.clone(),
            message: e.to_string(),
        })?
    {
        if bytes.len() as u64 + chunk.len() as u64 > MAX_JUNIT_JAR_BYTES {
            return Err(ForgeError::DownloadError {
                url,
                message: "El JAR de JUnit excede el límite de 128 MiB".to_string(),
            }
            .into());
        }
        bytes.extend_from_slice(&chunk);
    }
    if !has_zip_signature(&bytes) || sha256_hex(&bytes) != JUNIT_SHA256 {
        return Err(ForgeError::DownloadError {
            url,
            message: "El JAR de JUnit no coincide con el SHA-256 esperado".to_string(),
        }
        .into());
    }

    let temp_path = tools_dir.join(format!(".junit-{JUNIT_VERSION}-{}.tmp", std::process::id()));
    std::fs::write(&temp_path, bytes).map_err(|e| ForgeError::IoError {
        path: temp_path.clone(),
        message: e.to_string(),
    })?;
    if let Err(error) = std::fs::rename(&temp_path, &jar_path) {
        let _ = std::fs::remove_file(&temp_path);
        if jar_path.exists() && is_expected_junit_jar(&jar_path) {
            return Ok(jar_path);
        }
        return Err(ForgeError::IoError {
            path: jar_path,
            message: error.to_string(),
        }
        .into());
    }

    Ok(jar_path)
}

fn is_expected_junit_jar(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if metadata.len() < 4 || metadata.len() > MAX_JUNIT_JAR_BYTES {
        return false;
    }
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes).is_ok()
        && has_zip_signature(&bytes)
        && sha256_hex(&bytes) == JUNIT_SHA256
}

fn has_zip_signature(bytes: &[u8]) -> bool {
    matches!(
        bytes.get(..4),
        Some([b'P', b'K', 3, 4] | [b'P', b'K', 5, 6] | [b'P', b'K', 7, 8])
    )
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
    fn recognizes_only_zip_signatures() {
        assert!(has_zip_signature(b"PK\x03\x04content"));
        assert!(!has_zip_signature(b"<html>error"));
        assert!(!has_zip_signature(b"PK"));
    }
}
