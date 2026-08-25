use anyhow::{Context, Result};
use axum::{
    http::{header, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
    routing::get,
    Router,
};
use colored::Colorize;
use rust_embed::RustEmbed;
use std::convert::Infallible;
use std::path::Path;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

/// Empaqueta los estáticos de React compilados dentro del propio binario
#[derive(RustEmbed)]
#[folder = "../../forge-dashboard/dist/"]
struct Assets;

async fn security_headers(request: Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; connect-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self'; frame-ancestors 'none'; base-uri 'none'",
        ),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

/// Handler estático para resolver archivos (o rebotar a index.html en SPA)
async fn static_handler(uri: axum::extract::OriginalUri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Assets::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            // Comportamiento SPA: Si no se encuentra un estático, servir index.html
            match Assets::get("index.html") {
                Some(content) => {
                    let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                    ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
                }
                None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
            }
        }
    }
}

/// Endpoint SSE: Sirve Eventos en Tiempo Real desde el EventBus
async fn api_events() -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    // Suscribirse al bus global de FORGE
    let rx = cyrce_forge_core::telemetry::global_event_bus().subscribe();

    // Convertir el Receiver en un Async Stream que Axum pueda bombear como SSE
    let stream = BroadcastStream::new(rx).filter_map(|msg| {
        match msg {
            Ok(event) => {
                if let Ok(json_str) = serde_json::to_string(&event) {
                    Some(Ok(Event::default().data(json_str)))
                } else {
                    None
                }
            }
            Err(_) => None, // Ignorar lags en el canal
        }
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

/// Inicia el Dashboard Web
pub async fn cmd_dashboard(_project_dir: &Path, port: u16) -> Result<()> {
    println!("{}", "🚀 Iniciando FORGE Web Dashboard...".cyan().bold());

    let app = Router::new()
        .route("/api/events", get(api_events))
        .fallback(get(static_handler))
        .layer(middleware::from_fn(security_headers));

    // El dashboard expone logs y nombres de tareas: por defecto solo debe ser
    // accesible desde la máquina local.
    let addr = format!("127.0.0.1:{}", port);
    println!(
        "{} El dashboard está corriendo en: {} 🌐",
        "✅ Listo:".green(),
        format!("http://localhost:{}", port).cyan().underline()
    );

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context(format!("No se pudo enlazar el puerto {}", port))?;

    axum::serve(listener, app)
        .await
        .context("El servidor web se detuvo inesperadamente")?;

    Ok(())
}
