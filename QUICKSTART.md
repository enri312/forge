# FORGE — Guía de inicio rápido

Esta guía cubre la instalación y los flujos principales de FORGE `0.11.0`.

## 1. Instalar

Los binarios oficiales se publican para Windows x86_64, Linux x86_64 y macOS x86_64/aarch64. Los instaladores verifican el SHA-256 del paquete.

```bash
# Linux o macOS
curl -fsSL https://raw.githubusercontent.com/enri312/forge/main/install.sh | bash
```

```powershell
# Windows PowerShell
iwr https://raw.githubusercontent.com/enri312/forge/main/install.ps1 -useb | iex
```

Desde el código fuente, con Rust estable:

```bash
git clone https://github.com/enri312/forge.git
cd forge
cargo install --path crates/forge-cli
```

Comprueba la instalación y las herramientas del lenguaje:

```bash
forge --version
forge doctor
```

Necesitarás un JDK para Java, un JDK y `kotlinc` para Kotlin, o Python para proyectos Python.

## 2. Crear un proyecto

```bash
forge new mi-app-java -l java
forge new mi-app-kotlin -l kotlin
forge new mi-app-python -l python
```

Para trabajar en una carpeta existente:

```bash
mkdir mi-app
cd mi-app
forge init java
```

## 3. Compilar, ejecutar y probar

```bash
cd mi-app-java
forge build
forge run
forge test
```

`forge build --release` usa el perfil de release. La clave incremental separa los perfiles y también cambia cuando cambian la configuración, la plataforma o una dependencia local.

## 4. Configurar `forge.toml`

Ejemplo Java completo:

```toml
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
"com.google.gson:gson" = "2.13.1"

[test-dependencies]
"org.junit.jupiter:junit-jupiter-api" = "5.12.0"

[hooks]
pre-build = ["echo Preparando build"]
post-test = ["echo Pruebas finalizadas"]

[tasks.verify]
command = "forge test"
description = "Ejecutar pruebas"

[tasks.distribute]
command = "forge package"
depends-on = ["verify"]
description = "Probar y empaquetar"
```

El parser rechaza claves desconocidas y rutas que escapen del proyecto. Usa el [esquema oficial](schemas/forge.schema.json) para autocompletado.

## 5. Dependencias

```bash
forge add com.google.guava:guava@33.4.8-jre
forge deps
forge tree
```

Para Python:

```toml
[dependencies]
requests = "2.32.5"
```

`forge upgrade` es beta y actualmente solo actualiza dependencias PyPI.

## 6. Workspaces multi-módulo

`modules` debe declararse en el nivel superior, antes de abrir otra tabla TOML:

```toml
modules = ["core", "api", "web"]

[project]
name = "mi-workspace"
version = "1.0.0"
lang = "java"
```

Cada módulo necesita su propio `forge.toml`. FORGE valida el grafo y compila los módulos en orden de dependencia. También puedes declarar una dependencia local:

```toml
[dependencies]
core = "path:../core"
```

## 7. Caché

Inspecciona o depura el repositorio CAS global:

```bash
forge cache status
forge cache prune
```

Configura un servidor HTTP(S) compatible de forma opcional:

```toml
[cache]
remote = "https://cache.example.com/forge"
push = false
```

Si usas `token`, la URL debe ser HTTPS salvo para localhost. No guardes el token en el repositorio.

## 8. Desarrollo diario

```bash
forge watch
forge fmt
forge lint
forge task verify
forge package
forge bench
```

Integración con IDE:

```bash
forge ide vscode
forge ide intellij
```

Dashboard local:

```bash
forge dashboard --port 3000
```

Abre `http://127.0.0.1:3000`. El servidor no escucha interfaces externas.

## 9. Diagnóstico y ayuda

```bash
forge --help
forge info
forge stats
forge doctor
```

Si encuentras un fallo, abre un [issue](https://github.com/enri312/forge/issues). Para vulnerabilidades, usa el canal privado descrito en [SECURITY.md](SECURITY.md).

Vuelve al [README principal](README.md) para conocer la arquitectura, la seguridad y los límites actuales.
