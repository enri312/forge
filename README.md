<div align="center">
  <img src="assets/forge-logo-dark.png" alt="Forge Build System Logo" width="350"/>
  <h1>🔥 FORGE — The Next-Gen Cargo-like Build System</h1>
</div>

<div align="center">

<img src="assets/forge-icon.svg" width="120" alt="FORGE Logo">

```
   ███████╗ ██████╗ ██████╗  ██████╗ ███████╗
   ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
   █████╗  ██║   ██║██████╔╝██║  ███╗█████╗  
   ██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝  
   ██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
   ╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
```

**Un build system moderno, rápido y simple. Escrito en Rust 🦀**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Java](https://img.shields.io/badge/Java-17--25-red.svg)](#)
[![Kotlin](https://img.shields.io/badge/Kotlin-2.1+-purple.svg)](#)
[![Python](https://img.shields.io/badge/Python-3.10--3.14-blue.svg)](#)

</div>

---

## 🤔 ¿Qué es FORGE?

**FORGE** es un build system de nueva generación diseñado para reemplazar a Gradle con un enfoque más simple, rápido y multi-lenguaje. Escrito en Rust, arranca instantáneamente y consume mínima memoria.

### ¿Por qué FORGE en lugar de Gradle?

| Problema de Gradle | Solución de FORGE |
|---|---|
| 🐢 Arranque lento (JVM) | ⚡ Binario nativo — arranque instantáneo |
| 💾 Alto consumo de memoria | 🪶 Rust: mínima memoria, sin GC |
| 📚 Configuración compleja (Groovy/Kotlin DSL) | 📄 TOML simple y legible |
| 🤯 Difícil de depurar | 💬 Mensajes de error claros |
| 📈 Curva de aprendizaje alta | 🎯 Convención sobre configuración |
| ☕ Solo JVM nativo | 🌐 Java + Kotlin + Python desde el inicio |

---

## 🚀 Inicio Rápido

### 1. Instalar FORGE

```bash
# Para sistemas UNIX (Linux / macOS)
curl -fsSL https://raw.githubusercontent.com/enri312/forge/main/install.sh | bash

# Para sistemas Windows (PowerShell)
iwr https://raw.githubusercontent.com/enri312/forge/main/install.ps1 -useb | iex

# Instalación vía Cargo Native (requiere Rust)
cargo install cyrce-forge-cli
```

### 2. Crear un Proyecto

```bash
# Crear proyecto en carpeta nueva
forge new mi-app -l java
forge new mi-app -l kotlin
forge new mi-app -l python

# O inicializar en el directorio actual
forge init java
```

### 3. Compilar y Ejecutar

```bash
forge build    # Compilar
forge run      # Compilar y ejecutar
forge test     # Ejecutar tests
forge clean    # Limpiar artefactos
```

### 4. Verificar tu Sistema

```bash
forge doctor   # Diagnóstico completo del sistema
```

---

## 📄 Configuración (`forge.toml`)

FORGE usa un archivo `forge.toml` simple y legible en la raíz de tu proyecto:

### Proyecto Java

```toml
[project]
name = "mi-app"
version = "1.0.0"
lang = "java"

[java]
source = "src/main/java"
test-source = "src/test/java"
target = "17"
main-class = "com.ejemplo.Main"

[dependencies]
"com.google.guava:guava" = "33.0.0-jre"
"org.slf4j:slf4j-api" = "2.0.9"

[test-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "6.0.3"
"org.junit.jupiter:junit-jupiter-engine" = "6.0.3"

[tasks.lint]
command = "echo Linting..."
description = "Verificar estilo de código"
```

### Proyecto Kotlin

```toml
[project]
name = "mi-app-kotlin"
version = "1.0.0"
lang = "kotlin"

[kotlin]
source = "src/main/kotlin"
jvm_target = "17"
main-class = "MainKt"

[dependencies]
"org.jetbrains.kotlinx:kotlinx-coroutines-core" = "1.8.0"
```

### Proyecto Python

```toml
[project]
name = "mi-script"
version = "1.0.0"
lang = "python"

[python]
source = "src"
main-script = "main.py"

[dependencies]
"requests" = "2.31.0"
"flask" = "*"
```

---

## 📦 Comandos Disponibles (21)

### Esenciales

| Comando | Descripción |
|---|---|
| `forge init <lang>` | 🆕 Inicializar proyecto en directorio actual |
| `forge new <nombre> -l <lang>` | 📁 Crear proyecto en carpeta nueva |
| `forge build` | 🔨 Compilar el proyecto |
| `forge run` | 🚀 Compilar y ejecutar |
| `forge test` | 🧪 Ejecutar tests |
| `forge clean` | 🧹 Limpiar artefactos y caché |
| `forge deps` | 📦 Resolver dependencias |
| `forge add <dep>`| ➕ Añadir una dependencia automágicamente |
| `forge tree` | 🌲 Visualizar árbol de dependencias resueltas |
| `forge upgrade` | ⬆️ Actualizar versiones de dependencias declaradas |
| `forge ide <target>` | 🛠️ Generar metadatos IDE (`vscode` / `intellij`) |

### Desarrollo

| Comando | Descripción |
|---|---|
| `forge watch` | 👁️ Auto-rebuild al detectar cambios en código |
| `forge task <nombre>` | ⚙️ Ejecutar tarea personalizada del `forge.toml` |
| `forge bench` | ⏱️ Benchmark: medir tiempos de compilación |
| `forge package` | 📦 Empaquetar proyecto para distribución |
| `forge fmt` | 🎨 Formatear código (google-java-format, ktlint, black) |
| `forge lint` | 🔍 Análisis estático (checkstyle, detekt, ruff) |

### Información

| Comando | Descripción |
|---|---|
| `forge info` | ℹ️ Info del proyecto + herramientas del sistema |
| `forge stats` | 📊 Estadísticas: archivos, líneas, tamaño |
| `forge doctor` | 🩺 Diagnóstico completo del sistema |
| `forge completions <shell>` | 🐚 Autocompletado para bash/zsh/fish/PowerShell |

### Opciones Globales

```bash
forge --verbose build      # Modo verboso
forge build --release      # Modo de compilación optimizado
forge -p /otra/ruta build  # Especificar directorio del proyecto
forge --help               # Ver ayuda
forge --version            # Ver versión
```

---

## 🏗️ Arquitectura

FORGE está construido con una arquitectura modular:

```
forge/
├── forge-cli    → Interfaz de línea de comandos (clap)
├── forge-core   → Motor: DAG, ejecutor paralelo, caché
├── forge-langs  → Módulos: Java, Kotlin, Python
├── forge-deps   → Resolución: Maven Central, PyPI
└── editors/     → Extensiones oficiales (Ej. vscode)
```

### Características Técnicas

- **⚡ Ejecución Paralela**: Las tareas sin dependencias se ejecutan simultáneamente usando un grafo DAG
- **💾 Caché Incremental**: Solo recompila archivos que han cambiado (hashing SHA-256)
- **📦 Dependencias Automáticas**: Descarga JARs de Maven Central y paquetes de PyPI con **resolución transitiva** de POMs
- **🧪 Test Runners Nativos**: Integra PyTest y auto-descarga global de JUnit 6 Console Standalone para testear nativamente.
- **🛠️ Compatibilidad IDE**: Generación automática de setups con `forge ide` y bundle interactivo para VS Code.
- **🪝 Hooks de Ciclo de Vida**: `pre-build`, `post-build`, `pre-test`, `post-test` configurables en `forge.toml`
- **📦 Multi-Módulo**: Soporte de workspaces con sub-proyectos independientes (`modules = [...]`)
- **🎨 Formateo y Linting**: `forge fmt` y `forge lint` integran herramientas nativas por lenguaje
- **👁️ Watch Mode**: Vigila cambios y recompila automáticamente usando file watchers nativos
- **🩺 System Doctor**: Diagnóstico completo con sugerencias de instalación
- **📊 Project Stats**: Conteo de archivos, líneas de código y tamaño
- **⏱️ Benchmarking**: Mide y compara tiempos de compilación
- **🎨 UX Moderna**: Barras de progreso, colores y mensajes descriptivos
- **� Shell Completions**: Autocompletado para bash, zsh, fish y PowerShell
- **�🔌 Extensible**: Arquitectura modular con traits para agregar nuevos lenguajes

---

## 🤝 Contribuir

¡Las contribuciones son bienvenidas! Ver [CONTRIBUTING.md](CONTRIBUTING.md) para detalles.

### Ideas para Contribuir

- 🦀 **Nuevos lenguajes**: C/C++, TypeScript
- 📦 **Plugin system basados en WebAssembly (WASM)**
-  **Docker support**: Builds nativos en contenedores
- 📝 **Dashboard Web GUI**: UI gráfica del DAG

---

## 📋 Roadmap

- [x] **v0.1.0** — Estructura base, CLI y motor core
- [x] **v0.1.0** — Compilación Java, Kotlin y Python
- [x] **v0.1.0** — Resolución de dependencias (Maven Central, PyPI)
- [x] **v0.1.0** — Caché incremental con SHA-256
- [x] **v0.1.1** — Watch mode (recompilación automática)
- [x] **v0.1.1** — Shell completions (bash, zsh, fish, PowerShell)
- [x] **v0.1.1** — JSON Schema para `forge.toml`
- [x] **v0.1.1** — `forge doctor`, `forge stats`, `forge bench`
- [x] **v0.1.1** — `forge new`, `forge task`, `forge package`
- [x] **v0.1.1** — GitHub Actions CI (Linux, Windows, macOS)
- [x] **v0.2.0** — Test runners nativos (Aislacion de descargas de tests unitarios, auto-descarga global de JUnit 6 Console, setup nativo virtualenv pytest/unittest e integracion CLI)
- [x] **v0.3.0** — Generador de setups `forge ide` y Extensión básica de VS Code
- [x] **v0.4.0** — Hooks de ciclo de vida, dependencias transitivas Maven, multi-módulo, `forge fmt` y `forge lint`
- [x] **v0.5.0** — Caché remoto distribuido (S3/HTTP) con subida automática
- [x] **v0.6.0** — Servidor oficial LSP (Language Server Protocol) para `forge.toml`
- [x] **v0.7.0** — Dashboard Web Interactivo (Vite/React/Axum) con Server Embebido
- [x] **v0.8.0** — Telemetría de Compilación en Vivo con Event Bus Global y Server-Sent Events (SSE)
- [x] **v0.9.0** — Motor Multi-Módulo (DAG, Verificación de Ciclos, Local Paths y Cross-Inyección de Classpaths JVM)

### En desarrollo (Próximamente)
- [ ] **v1.0.0** — **Fase 17:** Adopción Pública y Plugins (WebAssembly / Extism)
- [ ] Publicación en `crates.io` y gestores comunitarios (Homebrew, Scoop)

---

## 📜 Licencia

Este proyecto está bajo la licencia **MIT**. Ver [LICENSE](LICENSE) para más detalles.

---

<div align="center">

**Hecho con 🔥 y Rust 🦀 por [SkyShoot](https://github.com/enri312)**

*FORGE es un proyecto de código abierto. ¡Únete a la fragua!*

🌐 [github.com/enri312/forge](https://github.com/enri312/forge)

</div>
