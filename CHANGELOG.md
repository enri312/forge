# Todos los cambios notables del proyecto FORGE se documentan aquí.
# Formato basado en [Keep a Changelog](https://keepachangelog.com/es/1.1.0/).

## [0.1.1] — 2026-02-24

### 🚀 Nuevas Funcionalidades

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
