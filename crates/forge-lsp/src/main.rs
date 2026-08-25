// =============================================================================
// 🔥 FORGE — Language Server Protocol (LSP)
// =============================================================================
// Motor LSP oficial para FORGE. Provee diagnóstico, hover y autocompletado
// para archivos `forge.toml` en editores compatibles (ej: VS Code).
// Utiliza la crate `tower-lsp` para manejar la comunicación JSON-RPC.
// =============================================================================

use cyrce_forge_core::config::ForgeConfig;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct ForgeBackend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for ForgeBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "forge-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                "🔥 FORGE LSP Server inicializado exitosamente.",
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("Abierto: {}", params.text_document.uri.as_str()),
            )
            .await;
        self.validate_document(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.pop() {
            self.validate_document(params.text_document.uri, change.text)
                .await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let _uri = params.text_document_position_params.text_document.uri;
        let _position = params.text_document_position_params.position;

        // Para el MVP, responderemos con algo de información estática sobre FORGE.
        // En el futuro, determinaremos el contexto de la línea y columna para dar
        // descripciones específicas de "dependencies", "project.name", etc.
        let hover_text = "🔥 **FORGE Configuration**\n\nArchivo principal de configuración de compilación de FORGE. Usa formato TOML.";
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(hover_text.to_string())),
            range: None,
        }))
    }
}

impl ForgeBackend {
    /// Valida el contenido de un `forge.toml` simulando la carga en `forge_core`
    /// y publica los diagnósticos (errores) de vuelta al cliente.
    async fn validate_document(&self, uri: Url, text: String) {
        let mut diagnostics = Vec::new();

        // 1. Verificación básica de sintaxis TOML
        match toml::from_str::<toml::Value>(&text) {
            Ok(_) => {
                // La sintaxis es válida; aplicar exactamente la misma validación
                // estructural y semántica que usa la CLI.
                if let Err(error) = ForgeConfig::parse(&text) {
                    diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 1),
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: error.to_string(),
                        source: Some("forge-lsp".to_string()),
                        ..Default::default()
                    });
                }
            }
            Err(e) => {
                // Extraer línea/columna del error si es posible
                let (line, col) = match e.span() {
                    Some(span) => {
                        // Calcular linea/col basada en el span offset (simplificado)
                        let prefix = &text[..span.start];
                        let line = prefix.lines().count().saturating_sub(1) as u32;
                        let col = prefix.lines().last().unwrap_or("").len() as u32;
                        (line, col)
                    }
                    None => (0, 0),
                };

                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position::new(line, col),
                        end: Position::new(line, col + 1), // Marcar al menos 1 caracter
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Sintaxis TOML inválida: {}", e.message()),
                    source: Some("forge-lsp".to_string()),
                    ..Default::default()
                };
                diagnostics.push(diagnostic);
            }
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tokio::main]
async fn main() {
    // Configurar tracing local a stderr si es necesario,
    // pero LSP usa stdin/stdout para comunicarse.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    // Tower-LSP se comunica a través de stdin/stdout
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| ForgeBackend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
