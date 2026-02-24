// =============================================================================
// 🔥 FORGE — Motor Core: Sistema de Plugins (WASM)
// =============================================================================
// Implementación fase 17: Soporte para ejecución de Plugins de la comunidad
// aislados mediante WebAssembly usando el framework Extism.
// =============================================================================

use std::path::{Path, PathBuf};
use std::collections::HashMap;

use std::collections::HashMap;

use colored::Colorize;
use extism::{Manifest, Plugin, Wasm, extism_fn};

use crate::error::{ForgeError, ForgeResult};
use crate::config::ForgeConfig;

/// El Manager de Plugins aislará las Máquinas Virtuales Wasm.
pub struct PluginManager {
    project_dir: PathBuf,
    plugins: HashMap<String, Plugin>,
}

impl PluginManager {
    /// Inicializa el gestor de plugins recorriendo el bloque `[plugins]` de forge.toml.
    pub fn new(config: &ForgeConfig, project_dir: &Path) -> ForgeResult<Self> {
        let mut manager = Self {
            project_dir: project_dir.to_path_buf(),
            plugins: HashMap::new(),
        };

        if config.plugins.is_empty() {
            return Ok(manager);
        }

        println!("{}", "🔌 Inicializando Subsistema de Plugins (WASM)...".cyan().bold());

        for (name, source) in &config.plugins {
            // El source puede ser un path local o una URL. Extism SDK maneja Wasm::file / Wasm::url
            let wasm = if source.starts_with("http") {
                Wasm::url(source)
            } else {
                let local_path = project_dir.join(source);
                Wasm::file(local_path)
            };

            let path_str = manager.project_dir.to_string_lossy().to_string();
            let manifest = Manifest::new([wasm])
                .with_allowed_path(path_str, "/project");

            // Registramos las host functions públicas
            let functions = [
                extism::Function::from(forge_log_info),
            ];

            // Instanciar el plugin WASM.
            match Plugin::new(&manifest, functions, true) {
                Ok(plugin) => {
                    manager.plugins.insert(name.clone(), plugin);
                    println!("   {} Plugin '{}' cargado exitosamente.", "📦".green(), name);
                }
                Err(e) => {
                    eprintln!("   {} Fallo instanciando plugin '{}': {}", "⚠️ ".yellow(), name, e);
                    // No abortaremos el build global por un plugin dañado.
                }
            }
        }

        Ok(manager)
    }

    /// Llama a un método "export" en el plugin WASM especificado, pasándole datos por byte buffer.
    pub fn call_plugin<'a>(&mut self, plugin_name: &str, function: &str, input: impl extism::ToBytes<'a>) -> ForgeResult<Vec<u8>> {
        let plugin = self.plugins.get_mut(plugin_name).ok_or_else(|| ForgeError::TaskFailed {
            task_name: format!("Plugin '{}' no encontrado", plugin_name),
            exit_code: 1,
        })?;

        let output = plugin.call::<_, Vec<u8>>(function, input).map_err(|e| ForgeError::TaskFailed {
            task_name: format!("Error en WASM '{}::{}': {}", plugin_name, function, e),
            exit_code: 1,
        })?;

        Ok(output)
    }

    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }
}

// -----------------------------------------------------------------------------
// Host Functions (Expuestas a los plugins WASM)
// -----------------------------------------------------------------------------

extism_fn!(
    /// Imprime un log al estilo FORGE desde la VM WASM invitada.
    forge_log_info(_plugin, msg: String) -> () {
        println!("   {} {}", "🔌 Plugin:".cyan(), msg.dimmed());
    }
);
