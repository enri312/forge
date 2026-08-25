<div align="center">
  <img src="assets/forge-logo-dark.png" alt="FORGE Build System" width="360">

  # FORGE Build System

  **Builds reproducibles para Java, Kotlin y Python, con una configuración TOML y un motor nativo en Rust.**

  [![CI](https://github.com/enri312/forge/actions/workflows/ci.yml/badge.svg)](https://github.com/enri312/forge/actions/workflows/ci.yml)
  [![Latest release](https://img.shields.io/github/v/release/enri312/forge)](https://github.com/enri312/forge/releases/latest)
  [![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)
  [![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org/)
</div>

## Qué es

FORGE es un build system de código abierto orientado a proyectos Java, Kotlin y Python. Su objetivo es ofrecer un flujo sencillo —`forge build`, `forge run`, `forge test`— sin obligar a escribir XML ni un DSL ejecutable.

La versión `0.11.0` incluye:

- compilación, ejecución, pruebas, empaquetado, watch mode, formato y lint por lenguaje;
- dependencias de Maven Central y PyPI, incluidas dependencias Maven transitivas directas;
- builds incrementales cuya clave contempla fuentes, configuración, perfil, plataforma y dependencias locales;
- workspaces multi-módulo con orden topológico y detección de ciclos;
- repositorio CAS global en `~/.forge/repository/cas`, con deduplicación mediante enlaces duros cuando el sistema lo permite;
- caché remota HTTP(S) opcional, restaurada en un área aislada y validada antes de reemplazar el build local;
- dashboard local con eventos SSE, JSON Schema, LSP e integración para VS Code/IntelliJ;
- validación estricta de configuración, rutas, coordenadas de dependencias y grafos de tareas.

> FORGE sigue siendo software `0.x`. Es funcional y está cubierto por CI en Linux, macOS y Windows, pero todavía no pretende cubrir todo el modelo de Gradle o Maven. Consulta [Estado y límites](#estado-y-límites) antes de migrar un build crítico.

## Instalación

Los instaladores descargan la última release compatible y verifican su checksum SHA-256 publicado.

### Linux y macOS

```bash
curl -fsSL https://raw.githubusercontent.com/enri312/forge/main/install.sh | bash
```

Binarios disponibles: Linux x86_64 y macOS x86_64/aarch64.

### Windows PowerShell

```powershell
iwr https://raw.githubusercontent.com/enri312/forge/main/install.ps1 -useb | iex
```

Binario disponible: Windows x86_64.

### Desde el código fuente

Requiere Rust estable:

```bash
git clone https://github.com/enri312/forge.git
cd forge
cargo install --path crates/forge-cli
```

Para instalar exactamente la release `v0.11.0` desde Git:

```bash
cargo install --git https://github.com/enri312/forge.git --tag v0.11.0 cyrce-forge-cli
```

Verifica el entorno del lenguaje que utilizarás:

```bash
forge --version
forge doctor
```

## Inicio rápido

```bash
# java, kotlin o python
forge new mi-app -l java
cd mi-app

forge build
forge run
forge test
```

También puedes ejecutar `forge init java` dentro de una carpeta existente. La [guía de inicio rápido](QUICKSTART.md) contiene ejemplos de tareas, workspaces, caché y dashboard.

## Configuración

FORGE lee `forge.toml` desde la raíz del proyecto. Este ejemplo Java usa claves aceptadas por el [JSON Schema](schemas/forge.schema.json):

```toml
modules = ["core", "api"]

[project]
name = "mi-app"
version = "1.0.0"
lang = "java"
java-version = "21"
output_dir = "build"

[java]
source = "src/main/java"
test-source = "src/test/java"
target = "21"
main-class = "com.ejemplo.Main"

[dependencies]
"com.google.guava:guava" = "33.4.8-jre"

[test-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.12.0"

[hooks]
pre-build = ["echo Preparando build"]

[tasks.verify]
command = "forge test"
description = "Ejecutar verificaciones"

[tasks.package]
command = "forge package"
depends-on = ["verify"]
description = "Verificar y empaquetar"
```

Las claves `modules`, `dependencies`, `test-dependencies`, `hooks`, `tasks` y `cache` son de nivel superior. Cada entrada de `modules` apunta a una carpeta con su propio `forge.toml`.

### Kotlin

```toml
[project]
name = "mi-app-kotlin"
lang = "kotlin"

[kotlin]
source = "src/main/kotlin"
test-source = "src/test/kotlin"
jvm_target = "21"
main-class = "MainKt"
```

### Python

```toml
[project]
name = "mi-script"
lang = "python"

[python]
source = "src"
main-script = "main.py"
python_version = "3.12"

[dependencies]
requests = "2.32.5"
```

### Caché remota

El servidor debe aceptar `GET` y, si `push = true`, `PUT` sobre `/<clave>.tar.gz`. Usa HTTPS si configuras un token; HTTP con token solo se permite en localhost.

```toml
[cache]
remote = "https://cache.example.com/forge"
token = "token-del-entorno-seguro"
push = false
```

No confirmes tokens reales en Git. Para equipos y CI, genera el archivo desde un almacén de secretos o limita el token a la vida del job.

## Comandos

| Área | Comandos |
|---|---|
| Proyecto | `init`, `new`, `build`, `run`, `test`, `clean`, `package` |
| Dependencias | `deps`, `add`, `upgrade`, `tree` |
| Desarrollo | `watch`, `task`, `fmt`, `lint`, `bench` |
| Diagnóstico | `info`, `doctor`, `stats` |
| Herramientas | `completions`, `ide`, `dashboard`, `cache` |

```bash
forge --help
forge build --release
forge -p /ruta/al/proyecto build
forge dashboard --port 3000
forge cache status
forge cache prune
```

`forge dashboard` solo escucha en `127.0.0.1` y muestra datos reales del proyecto y eventos de compilación. `forge upgrade` es beta y actualmente actualiza dependencias PyPI.

## Arquitectura

```text
crates/
├── forge-cli/    CLI, comandos y servidor local del dashboard
├── forge-core/   configuración, DAG, ejecutor, caché y telemetría
├── forge-deps/   resolución Maven Central y PyPI
├── forge-langs/  Java, Kotlin, Python y runners de pruebas
└── forge-lsp/    diagnósticos LSP para forge.toml

forge-dashboard/  frontend React/Vite embebido en la CLI
editors/vscode/   extensión y tema de iconos para VS Code
schemas/          JSON Schema de forge.toml
```

## Seguridad y cadena de suministro

- Las descargas aplican timeouts, límites de redirección/tamaño y validación del formato esperado.
- Maven y sus POM se verifican con el SHA-256 publicado; se acepta SHA-1 únicamente cuando Maven Central no publica SHA-256. El runner JUnit usa un SHA-256 fijado.
- Los archivos de caché remota se extraen en staging con límites y rechazo de traversal, enlaces y entradas no regulares.
- CI ejecuta pruebas, formato, Clippy estricto, auditorías Rust/npm y genera un SBOM CycloneDX.
- Cada release publica checksums SHA-256 y procedencia firmada mediante GitHub Artifact Attestations/Sigstore.

Reporta vulnerabilidades de forma privada siguiendo [SECURITY.md](SECURITY.md). No abras un issue público con detalles explotables.

## Estado y límites

- El resolver Maven cubre coordenadas declaradas y transitividad de POM hasta una profundidad limitada; aún no implementa todo el modelo efectivo de Maven, perfiles ni la semántica completa de `dependencyManagement`.
- La caché remota usa un protocolo HTTP(S) simple; FORGE no incluye el servidor ni un adaptador S3 nativo.
- La deduplicación por hardlink depende del sistema de archivos; FORGE usa copia como alternativa cuando no es posible enlazar.
- El LSP y la extensión de VS Code son iniciales.
- El sistema de plugins WebAssembly todavía no forma parte del producto público.
- No existen aún paquetes oficiales de Homebrew o Scoop. La versión publicada en crates.io puede quedar detrás de GitHub Releases.

## Versiones

- `v0.11`: CAS global, classpath local híbrido, claves de caché completas y endurecimiento de seguridad/cadena de suministro.
- `v0.10`: classifiers Maven.
- `v0.9`: workspaces multi-módulo y dependencias locales.
- `v0.6–v0.8`: caché remota, LSP, dashboard y telemetría SSE.
- `v0.1–v0.5`: motor base, lenguajes, testing, IDE, hooks y herramientas de desarrollo.

El historial detallado está en [CHANGELOG.md](CHANGELOG.md).

## Desarrollo y contribuciones

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Consulta [CONTRIBUTING.md](CONTRIBUTING.md) para preparar el dashboard, la extensión y las pruebas de cada lenguaje.

## Licencia

FORGE se distribuye bajo la [licencia MIT](LICENSE).

<div align="center">
  Hecho con Rust por <a href="https://github.com/enri312">SkyShoot</a>.
</div>
