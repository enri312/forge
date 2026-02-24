# 📖 Guía de Inicio Rápido — FORGE

Esta guía te lleva paso a paso desde la instalación hasta tu primer build con FORGE.

---

## 1. Prerrequisitos

### Obligatorio
- **Rust 1.75+** — [Instalar desde rustup.rs](https://rustup.rs)

### Según el lenguaje que uses
- **Java 17-25** — Para proyectos Java ([Descargar](https://adoptium.net/))
- **Kotlin 2.1+** — Para proyectos Kotlin ([Descargar](https://github.com/JetBrains/kotlin/releases)) o disponible con IntelliJ IDEA
- **Python 3.10-3.14** — Para proyectos Python ([Descargar](https://python.org))

> **💡 Tip**: Ejecuta `forge doctor` para verificar qué herramientas tienes instaladas y cuáles faltan.

---

## 2. Instalar FORGE

```bash
# Clonar el repositorio
git clone https://github.com/enri312/forge.git
cd forge

# Compilar e instalar globalmente
cargo install --path crates/forge-cli

# Verificar la instalación
forge --version

# Diagnosticar el sistema
forge doctor
```

---

## 3. Tu Primer Proyecto — Java

```bash
# Crear proyecto en carpeta nueva
forge new mi-app-java -l java
cd mi-app-java

# Compilar y ejecutar
forge run
```

Deberías ver:
```
🔥 ¡Hola desde FORGE! — Proyecto Java
   Build system de nueva generación
```

---

## 4. Tu Primer Proyecto — Kotlin

```bash
forge new mi-app-kotlin -l kotlin
cd mi-app-kotlin
forge run
```

---

## 5. Tu Primer Proyecto — Python

```bash
forge new mi-script -l python
cd mi-script
forge run
```

---

## 6. Agregar Dependencias

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

## 7. Watch Mode (Auto-Rebuild)

Mientras desarrollas, FORGE puede vigilar tus archivos y recompilar automáticamente:

```bash
forge watch
# Edita tu código → FORGE detecta el cambio → Recompila automáticamente
# Ctrl+C para detener
```

---

## 8. Tareas Personalizadas

Define tareas reutilizables en `forge.toml`:

```toml
[tasks.lint]
command = "echo Verificando estilo..."
description = "Verificar estilo de código"

[tasks.hello]
command = "echo ¡Hola desde FORGE!"
```

```bash
forge task lint
forge task hello
```

---

## 9. Empaquetar para Distribución

```bash
forge package   # Empaqueta en dist/
```

Para Java/Kotlin: copia el JAR a `dist/`.
Para Python: copia fuente + `requirements.txt` a `dist/`.

---

## 10. Benchmark de Compilación

```bash
forge bench    # 3 rondas clean+build con estadísticas
```

---

## 11. Información y Estadísticas

```bash
forge info     # Info del proyecto + versiones del sistema
forge stats    # Archivos, líneas de código, tamaño
forge doctor   # Diagnóstico completo + sugerencias de instalación
```

---

## 12. Autocompletado de Shell

```bash
# PowerShell
forge completions powershell >> $PROFILE

# Bash
forge completions bash >> ~/.bashrc

# Zsh
forge completions zsh >> ~/.zshrc

# Fish
forge completions fish > ~/.config/fish/completions/forge.fish
```

---

## 13. Todos los Comandos

```bash
forge init <lang>              # Inicializar en directorio actual
forge new <nombre> -l <lang>   # Crear proyecto en carpeta nueva
forge build                    # Compilar
forge run                      # Compilar + ejecutar
forge test                     # Ejecutar tests
forge clean                    # Limpiar artefactos
forge deps                     # Resolver dependencias
forge watch                    # Auto-rebuild al detectar cambios
forge task <nombre>            # Ejecutar tarea personalizada
forge info                     # Info del proyecto
forge stats                    # Estadísticas del proyecto
forge doctor                   # Diagnóstico del sistema
forge bench                    # Benchmark de compilación
forge package                  # Empaquetar para distribución
forge completions <shell>      # Generar autocompletado
```

---

## ¿Problemas?

- Ejecuta `forge doctor` para diagnóstico automático
- `javac no encontrado` → Instala JDK desde [adoptium.net](https://adoptium.net)
- `kotlinc no encontrado` → Descarga desde [Kotlin releases](https://github.com/JetBrains/kotlin/releases) o usa IntelliJ IDEA
- `python no encontrado` → Instala Python 3.10+ desde [python.org](https://python.org)
- Abre un [issue en GitHub](https://github.com/enri312/forge/issues) si encuentras un bug 🐛

---

🔥 ¡Felicidades! Ya estás usando FORGE. Explora la [documentación completa](../README.md) para más detalles.
