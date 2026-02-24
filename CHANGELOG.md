# Todos los cambios notables del proyecto FORGE se documentan aquí.
# Formato basado en [Keep a Changelog](https://keepachangelog.com/es/1.1.0/).

## [0.1.0] — 2026-02-23

### 🎉 Lanzamiento Inicial

#### Agregado
- **CLI** con 7 comandos: `init`, `build`, `run`, `test`, `clean`, `deps`, `info`
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
- **CI/CD**:
  - GitHub Actions para build y tests en Linux, Windows y macOS
  - Build release con artefactos descargables
- **Autocompletado**:
  - JSON Schema para `forge.toml` (autocompletado en IDEs)
  - Autocompletado de shell integrado (PowerShell, bash, zsh, fish)
- **Tests**: 9 tests unitarios cubriendo DAG, configuración y caché
- **Proyectos de ejemplo**: Java, Kotlin y Python listos para probar
