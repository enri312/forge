# Todos los cambios notables del proyecto FORGE se documentan aquí.
# Formato basado en [Keep a Changelog](https://keepachangelog.com/es/1.1.0/).

## [0.9.0] — 2026-02-25

### Added
- **Arquitectura Multi-Módulo (DAG Inter-Proyecto)**: Soporte completo para iteraciones complejas entre submódulos locales mediante la directiva `path:` dentro del `forge.toml`. El motor detectará cualquier referencia cruzada cíclica inyectada por error emitiendo un Warning limpio y paralelizado de toda otra capa agnóstica a este loop a través de `tokio::spawn`.
- **Inyección Transversal de Classpaths (Java/Kotlin)**: Se rediseñaron los parsers nativos en las etapas de inter-compilación y ejecución (`java.rs` y `kotlin.rs`). Los mismos descubren recursivamente proyectos in-workspace construyendo los classpaths interconectados al invocar `javac` y `kotlinc`.
- **Estabilidad de Tokio**: Solución estructural a los *falsos positivos* de compilación de Rust `E0283` mediante `Boxed Futures` rompiendo la retro-referenciación en la dependencia de asincronicidad.

## [0.8.0] — 2026-02-25

### Added
- **Event Bus Global (`tokio::sync::broadcast`)**: Acople de un maestro canalizador general capaz de extraer eventos micro desde el motor interno de Rust sin obstruir el performance del DAG.
- **Server-Sent Events (SSE)**: La CLI reenvía todos los eventos originarios de Forge a `/api/events` sobre la red HTTP activa, inyectándolos en caliente al frontend a 30 eventos/seg.

## [0.7.0] — 2026-02-25

### Added
- **Dashboard Web Interactivo**: Integración directa de un frontend Vite/React empaquetado y transpuesto a embebimiento desde Axum ejecutado como sidecar nativo de Forge invocándolo desde `forge dashboard`. Provee monitoreo de estado gráfico con un UI temático _Industrial Cyberpunk_.

## [0.6.0] — 2026-02-24

### Added
- **Servidor LSP (`forge-lsp`)**: Binario dedicado al Language Server Protocol (LSP). Evalúa `forge.toml` asíncronamente mientras el usuario escribe, devolviendo diagnósticos inmediatos (errores de sintaxis) y proveyendo un motor base para *Hover tooltips* sobre llaves de configuración de compilación.
- **Caché Distribuido/Remoto**: FORGE ahora soporta subir y descargar el flag de dependencias y binarios ya compilados a través de la red (HTTP/S3) usando la nueva sección opcional `[cache]` en `forge.toml`. Si un desarrollador o máquina de CI/CD ya resolvió este nodo con su hash, FORGE extrae del tar.gz remoto y omite la compilación local (`⚡ Caché remoto restaurado`), ahorrando ancho de banda y latencia.

## [0.5.0] — 2026-02-24

### Added
- **Comando `forge add`**: Permite añadir dependencias programáticamente usando `forge add <dep>` o `forge add <dep> --test`. Previene duplicados automáticamente.
- **Comando `forge tree`**: Muestra una representación visual del árbol superficial de dependencias declaradas en `forge.toml`.
- **Bandera `--release`**: Nueva bandera para el comando `forge build` diseñada para admitir hooks y rutinas de compilación optimizada en producción. Las sub-llamadas asíncronas respetan el flag.
- **Comando `forge upgrade`**: Scaffold para futura resolución y actualización automática de librerías.

## [0.4.0] — 2026-02-24

### Added
- **Sistema de Hooks**: Soporte para `pre-build`, `post-build`, `pre-test`, `post-test` configurables en `forge.toml` bajo la sección `[hooks]`.
- **Dependencias Transitivas Maven**: El resolver ahora lee POMs y descarga sub-dependencias recursivamente (profundidad máx. 5, filtro scope compile).
- **Multi-módulo**: Soporte de workspaces con `modules = ["submod1", "submod2"]` en `forge.toml`. Compila sub-proyectos antes del proyecto padre.
- **`forge fmt`**: Formateo automático de código usando google-java-format (Java), ktlint (Kotlin) o black/autopep8 (Python).
- **`forge lint`**: Análisis estático con checkstyle (Java), detekt (Kotlin) o ruff/flake8 (Python).
- **Schema actualizado**: `forge.schema.json` ahora incluye `hooks`, `test-dependencies`, `test-source` y `modules`.

## [0.3.0] — 2026-02-24

### Added
- **Comando `forge ide`**: Agregado nuevo comando CLI para generar configuraciones de IDE automáticamente. Soporta `--target vscode` y `--target intellij`.
- **IntelliJ IDEA**: Soporte para auto-generar directorios `.idea/modules.xml` y `.iml` vinculando la estructura FORGE al motor de JetBrains sin Gradle ni Maven.
- **VS Code**: Generación preconfigurada de `tasks.json`, `settings.json` (auto-vinculación con Even Better TOML localmente) y `launch.json`.
- **Extensión VS Code (Básica)**: Compilado del primer pipeline VSIX en TypeScript, adjuntado en `editors/vscode` el bundle para forzar inyección nativa del schema y botones rápidos "Run".

## [0.2.0] — 2026-02-24

### 🧪 Soporte Nativo para Testing (Test Runners)

#### Nuevas Funcionalidades
- **Runner de Java/Kotlin (JUnit 6)**: Resolvimiento automático de dependencias en `[test-dependencies]`, auto-descarga global del ejecutable `junit-platform-console-standalone` y parseo en vivo del árbol de la prueba en formato consola.
- **Runner de Python (pytest)**: `forge test` integra llamadas fluidas al framework `pytest` montado sobre un VirtualEnv transparente, con failover a `unittest`.
- **Inyección por Plantillas Out-of-the-Box**: `forge new -l <lang>` inyecta un framework base con aserciones automáticas 1+1=2 transparentes listas para ejecutar. Cero configuración inicial necesaria. 

#### Mejoras Internas
- **Caché Separado**: Descarga y aislamiento para librerías en `.forge/test-deps` sin mezclar test-dependencies con variables productivas en runtime.
- Módulo `deps` refactorizado para soportar repositorios y metas paralelas asíncronas de descargas usando la crate nativa `reqwest`.

---

## [0.1.1] — 2026-02-24

#### Nuevos Comandos
- **`forge new <nombre>`** — Crear proyecto en carpeta nueva con `--lang` configurable
- **`forge watch`** — Watch mode: vigila cambios en código y recompila automáticamente
- **`forge task <nombre>`** — Ejecutar tareas personalizadas definidas en `forge.toml`
- **`forge doctor`** — Diagnóstico completo del sistema con detección de herramientas y sugerencias de instalación
- **`forge stats`** — Estadísticas del proyecto: archivos, líneas de código, tamaño, desglose por extensión
- **`forge bench`** — Benchmark de compilación: 3 rondas clean+build con estadísticas de tiempo
- **`forge package`** — Empaquetar proyecto para distribución (JAR para Java/Kotlin, carpeta para Python)
- **`forge completions <shell>`** — Generar autocompletado para bash, zsh, fish, PowerShell

#### Mejoras
- **`forge info` mejorado** — Ahora muestra versiones de herramientas del sistema (Rust, Java, Python, Kotlin)
- **Timer de ejecución** — Muestra tiempo transcurrido para comandos que toman más de 100ms
- **Fix Kotlin en Windows** — `kotlinc.bat` ahora se ejecuta correctamente via `cmd /C`
- **Resolución de paths** — Paths relativos se convierten a absolutos con `canonicalize`

#### CI/CD y Configuración
- **GitHub Actions CI** — Build, test, clippy y fmt en Linux, Windows y macOS
- **Release automático** — Generación de binarios para 3 plataformas al crear un tag
- **JSON Schema** — `forge.schema.json` para autocompletado de `forge.toml` en IDEs
- **VS Code config** — Extensiones recomendadas y schema linking
- **Issue templates** — Templates para bug reports y feature requests

#### Documentación
- **SECURITY.md** — Política de seguridad para reportar vulnerabilidades
- **CHANGELOG.md** — Este archivo de cambios
- **Logo e identidad visual** — SVG del ícono en `assets/forge-icon.svg`
- **README actualizado** — 15 comandos documentados, roadmap actualizado

---

## [0.1.0] — 2026-02-23

### 🎉 Lanzamiento Inicial

#### Agregado
- **CLI** con 7 comandos base: `init`, `build`, `run`, `test`, `clean`, `deps`, `info`
- **Motor Core**:
  - Grafo de tareas (DAG) con detección de ciclos y ejecución por niveles
  - Ejecutor paralelo asíncrono con tokio
  - Caché incremental basada en hashing SHA-256
  - Parser de configuración `forge.toml` con validación
- **Módulo Java**:
  - Compilación con `javac` (Java 17-25)
  - Empaquetado JAR con manifiesto
  - Ejecución con `java`
- **Módulo Kotlin**:
  - Compilación con `kotlinc` (Kotlin 2.1+)
  - Empaquetado JAR
  - Ejecución con `java`
- **Módulo Python**:
  - Gestión automática de entornos virtuales (`venv`)
  - Instalación de dependencias con `pip`
  - Ejecución de scripts
  - Soporte para `pytest` y `unittest`
- **Resolución de Dependencias**:
  - Descarga de JARs desde Maven Central
  - Verificación de paquetes en PyPI
  - Caché local de dependencias
- **Documentación**:
  - README completo con inicio rápido y comparación con Gradle
  - Guía de contribución (CONTRIBUTING.md)
  - Guía de inicio rápido (docs/QUICKSTART.md)
  - Licencia MIT
- **Tests**: 9 tests unitarios cubriendo DAG, configuración y caché
- **Proyectos de ejemplo**: Java, Kotlin y Python listos para probar
