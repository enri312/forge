# 🔥 FORGE — Guía de Inicio Rápido

> Build system ultrarrápido para Java, Kotlin y Python.  
> Escrito en Rust. Sin XML. Sin Groovy. Solo TOML.

---

## ⚡ Instalación

### Desde código fuente (requiere Rust 1.80+):
```bash
git clone https://github.com/enri312/forge.git
cd forge
cargo install --path crates/forge-cli
```

Verifica la instalación:
```bash
forge --version
forge doctor       # Diagnostica si tienes las herramientas necesarias
```

---

## 🚀 Tu Primer Proyecto en 30 Segundos

### Crear proyecto Java:
```bash
forge new mi-app -l java
cd mi-app
forge build    # Compila
forge run      # Ejecuta
forge test     # Tests con JUnit 6 (auto-descargado)
```

### Crear proyecto Kotlin:
```bash
forge new mi-app -l kotlin
```

### Crear proyecto Python:
```bash
forge new mi-app -l python
```

---

## 📝 Estructura del `forge.toml`

```toml
[project]
name = "mi-app"
version = "1.0.0"
lang = "java"
description = "Mi aplicación Java"
output_dir = "build"

[java]
source = "src/main/java"
test-source = "src/test/java"
target = "21"
main-class = "com.ejemplo.Main"

# Dependencias de Maven Central (transitivas resueltas automáticamente)
[dependencies]
"com.google.gson:gson" = "2.11.0"
"org.slf4j:slf4j-api" = "2.0.9"

# Dependencias solo para testing
[test-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "6.0.3"

# Hooks de ciclo de vida
[hooks]
pre-build = ["echo Compilando..."]
post-build = ["echo Build completo!"]
pre-test = ["echo Preparando tests..."]

# Tareas personalizadas
[tasks.deploy]
command = "scp build/*.jar server:/opt/app/"
depends-on = ["build"]
description = "Desplegar a producción"
```

---

## 📦 Comandos Esenciales

| Comando | Descripción |
|---------|------------|
| `forge init <lang>` | Inicializar en directorio actual |
| `forge new <nombre>` | Crear proyecto en carpeta nueva |
| `forge build` | Compilar (con caché incremental) |
| `forge run` | Compilar y ejecutar |
| `forge test` | Ejecutar tests |
| `forge clean` | Limpiar artefactos |
| `forge deps` | Resolver dependencias |
| `forge fmt` | Formatear código |
| `forge lint` | Análisis estático |
| `forge watch` | Auto-rebuild al cambiar código |
| `forge info` | Información del proyecto |
| `forge doctor` | Diagnosticar sistema |

---

## 🏗️ Multi-Módulo (Workspaces)

Para proyectos con múltiples sub-módulos:

```toml
# forge.toml (raíz del workspace)
[project]
name = "mi-workspace"
lang = "java"

modules = ["core", "api", "web"]
```

Cada sub-módulo tiene su propio `forge.toml`. Al ejecutar `forge build` desde la raíz, se compilan **todos los módulos** en orden.

---

## 🛠️ Integración con IDEs

```bash
forge ide vscode     # Genera .vscode/settings.json, launch.json, tasks.json
forge ide intellij   # Genera .idea/modules.xml, proyecto.iml
```

---

## 🪝 Hooks

Los hooks ejecutan comandos antes y después de las fases de build y test:

```toml
[hooks]
pre-build = ["echo Antes de compilar", "npm run generate-version"]
post-build = ["echo Después de compilar"]
pre-test = ["echo Preparando base de datos de test"]
post-test = ["echo Limpiando datos de test"]
```

---

## 📊 Comparación con Gradle

| Aspecto | Gradle | FORGE |
|---------|--------|-------|
| Arranque | ~3-5s | ~50ms |
| Memoria | 500MB-2GB | ~15MB |
| Config | Groovy/Kotlin DSL | TOML simple |
| Multi-lang | Solo JVM | Java + Kotlin + Python |
| Curva | Alta | Mínima |

---

## 🔗 Links Útiles

- [README completo](./README.md)
- [CHANGELOG](./CHANGELOG.md)
- [Reportar un Bug](https://github.com/enri312/forge/issues)
- [Esquema JSON para intellisense](./schemas/forge.schema.json)
