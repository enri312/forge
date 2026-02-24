# 🤝 Guía de Contribución — FORGE

¡Gracias por tu interés en contribuir a FORGE! 🔥

Este documento te guiará a través del proceso para contribuir al proyecto.

---

## 📋 Requisitos Previos

- **Rust 1.75+** — [Instalar con rustup](https://rustup.rs)
- **Git** — Para clonar y gestionar el código
- **Java 17-25** (opcional) — Para probar el módulo Java ([Descargar](https://adoptium.net/))
- **Kotlin 2.1+** (opcional) — Para probar el módulo Kotlin ([Descargar](https://kotlinlang.org/))
- **Python 3.10-3.14** (opcional) — Para probar el módulo Python ([Descargar](https://python.org))

---

## 🚀 Configurar el Entorno

```bash
# 1. Fork del repositorio en GitHub

# 2. Clonar tu fork
git clone https://github.com/enri312/forge.git
cd forge

# 3. Compilar el proyecto
cargo build

# 4. Ejecutar tests
cargo test --workspace

# 5. Ejecutar FORGE localmente
cargo run -- --help
```

---

## 📁 Estructura del Proyecto

```
forge/
├── Cargo.toml           ← Workspace raíz
├── README.md            ← Documentación principal
├── CONTRIBUTING.md      ← Esta guía
├── LICENSE              ← Licencia MIT
│
├── crates/
│   ├── forge-cli/       ← CLI (punto de entrada)
│   │   └── src/
│   │       ├── main.rs      ← Funciones Core (build, run, test)
│   │       ├── ide.rs       ← Integración IDE (VS Code, IntelliJ)
│   │       ├── hooks.rs     ← Ejecución de pre/post actions 
│   │       ├── add.rs       ← Inyector de TOML (forge add)
│   │       ├── tree.rs      ← UI de dependencias (forge tree)
│   │       ├── fmt.rs       ← Formateadores
│   │       └── lint.rs      ← Linter estático
│   │
│   ├── forge-core/      ← Motor principal
│   │   └── src/
│   │       ├── config.rs    ← Parser forge.toml
│   │       ├── dag.rs       ← Grafo de tareas
│   │       ├── executor.rs  ← Ejecutor paralelo
│   │       ├── cache.rs     ← Caché incremental
│   │       └── error.rs     ← Tipos de error
│   │
│   ├── forge-langs/     ← Módulos de lenguaje
│   │   └── src/
│   │       ├── java.rs      ← Compilación Java
│   │       ├── kotlin.rs    ← Compilación Kotlin
│   │       └── python.rs    ← Gestión Python
│   │
│   └── forge-deps/      ← Resolución de dependencias
│       └── src/
│           ├── maven.rs     ← Maven Central
│           └── pypi.rs      ← PyPI
│
└── tests/               ← Proyectos de prueba
    ├── java_project/
    ├── kotlin_project/
    └── python_project/
```

---

## 🔧 Flujo de Contribución

### 1. Crear un Issue

Antes de empezar a trabajar en algo, crea un issue describiendo:
- **Qué** quieres cambiar
- **Por qué** es necesario
- **Cómo** planeas implementarlo

### 2. Crear una Branch

```bash
git checkout -b feature/mi-nueva-funcionalidad
```

### 3. Hacer tus Cambios

- Escribe código limpio y documentado
- Agrega tests para la nueva funcionalidad
- Asegúrate de que todos los tests pasan: `cargo test --workspace`

### 4. Commit y Push

```bash
git add .
git commit -m "feat: agregar soporte para XYZ"
git push origin feature/mi-nueva-funcionalidad
```

### 5. Crear un Pull Request

1. Ve a tu fork en GitHub
2. Crea un Pull Request hacia `main`
3. Describe tus cambios con detalle
4. Espera la revisión

---

## 📝 Convenciones de Código

### Mensajes de Commit

Usamos [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: agregar soporte para TypeScript
fix: corregir detección de ciclos en DAG
docs: actualizar README con nuevos ejemplos
refactor: simplificar lógica del ejecutor
test: agregar tests para caché incremental
```

### Estilo de Código Rust

- Usar `cargo fmt` antes de commitear
- Pasar `cargo clippy` sin warnings
- Documentar funciones públicas con `///`
- Tests dentro de cada módulo con `#[cfg(test)]`

---

## 🎯 Áreas donde Necesitamos Ayuda

### 🟢 Principiante (Good First Issue)
- Crear nuevas plantillas para `forge new` (Ej. frameworks web, microservicios)
- Mejorar mensajes de error con sugerencias (`error.rs`)
- Agregar más tests E2E
- Mejorar la documentación en The Docs y ejemplos

### 🟡 Intermedio
- Nuevo módulo para gestionar tareas con Docker
- Nuevo módulo de lenguaje nativo (Ej. Go, TypeScript, C++, Rust)
- Soporte para Test Coverage integrado (Jacoco, PyTest-Cov)
- Implementar la lógica completa de parsing para `forge upgrade`

### 🔴 Avanzado
- Sistema de plugins dinámicos y Scripts pre/post escritos en Rust
- Servidor LSP (Language Server Protocol) para validación de TOML en tiempo real
- Caché remoto y Builds Distribuidos
- Cross-Compilation (Compilación cruzada) desde Windows Host a Linux Targets

---

## ❓ ¿Preguntas?

- Abre un [Issue en GitHub](https://github.com/enri312/forge/issues)
- Únete a las discusiones del proyecto

¡Gracias por ayudar a forjar el futuro del build tooling! 🔥🦀
