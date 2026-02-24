# 📖 Guía de Inicio Rápido — FORGE

Esta guía te lleva paso a paso desde la instalación hasta tu primer build con FORGE.

---

## 1. Prerrequisitos

### Obligatorio
- **Rust 1.75+** — [Instalar desde rustup.rs](https://rustup.rs)

### Según el lenguaje que uses
- **Java 17-25** — Para proyectos Java ([Descargar](https://adoptium.net/))
- **Kotlin 2.1+** — Para proyectos Kotlin ([Descargar](https://kotlinlang.org/docs/command-line.html))
- **Python 3.10-3.14** — Para proyectos Python ([Descargar](https://python.org))

---

## 2. Instalar FORGE

```bash
# Clonar el repositorio
git clone https://github.com/tu-usuario/forge.git
cd forge

# Compilar e instalar globalmente
cargo install --path crates/forge-cli

# Verificar la instalación
forge --version
```

---

## 3. Tu Primer Proyecto — Java

```bash
# Crear directorio del proyecto
mkdir mi-primer-proyecto
cd mi-primer-proyecto

# Inicializar con FORGE
forge init java
```

Esto crea:
```
mi-primer-proyecto/
├── forge.toml          ← Configuración del proyecto
├── .gitignore          ← Archivos a ignorar
└── src/main/java/
    └── Main.java       ← Tu código de ejemplo
```

### Compilar y Ejecutar

```bash
# Solo compilar
forge build

# Compilar y ejecutar
forge run
```

Deberías ver:
```
🔥 ¡Hola desde FORGE! — Proyecto Java
   Build system de nueva generación
```

---

## 4. Tu Primer Proyecto — Python

```bash
mkdir mi-script-python
cd mi-script-python

forge init python
forge run
```

---

## 5. Agregar Dependencias

### Java (Maven Central)
Edita `forge.toml`:
```toml
[dependencies]
"com.google.gson:gson" = "2.10.1"
```

```bash
forge deps    # Descargar
forge build   # Compilar con las dependencias
```

### Python (PyPI)
```toml
[dependencies]
"requests" = "2.31.0"
```

```bash
forge build   # Crea venv e instala automáticamente
```

---

## 6. Personalizar Tareas

Puedes definir tareas personalizadas en `forge.toml`:

```toml
[tasks.lint]
command = "echo Ejecutando linter..."
description = "Verificar estilo de código"

[tasks.deploy]
command = "echo Desplegando aplicación..."
depends-on = ["build"]
description = "Desplegar a producción"
```

---

## 7. Comandos Útiles

```bash
forge info     # Ver información del proyecto
forge clean    # Limpiar builds anteriores
forge --help   # Ver todos los comandos
```

---

## ¿Problemas?

- `javac no encontrado` → Asegúrate de que Java está en tu PATH
- `kotlinc no encontrado` → Instala Kotlin y agrégalo al PATH
- `python no encontrado` → Instala Python 3.10+ y agrégalo al PATH
- Abre un issue en GitHub si encuentras un bug 🐛

---

🔥 ¡Felicidades! Ya estás usando FORGE. Explora la [documentación completa](README.md) para más detalles.
