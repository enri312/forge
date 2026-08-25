# Contribuir a FORGE

Gracias por ayudar a mejorar FORGE. El proyecto acepta correcciones, pruebas, documentación y propuestas acotadas.

## Antes de empezar

- Usa Rust estable y Git.
- Instala Node.js 22 para modificar el dashboard.
- Instala JDK, Kotlin o Python solo si vas a probar ese backend.
- Para vulnerabilidades, sigue [SECURITY.md](SECURITY.md) y no abras un issue público con detalles explotables.

En cambios grandes, abre primero un issue para acordar alcance y compatibilidad. Las correcciones pequeñas pueden ir directamente en un pull request bien explicado.

## Preparar el repositorio

```bash
git clone https://github.com/enri312/forge.git
cd forge

cargo build --workspace
cargo test --workspace
```

El dashboard se compila y queda embebido en la CLI:

```bash
cd forge-dashboard
npm ci
npm run build
cd ..

cargo build --workspace
```

La extensión de VS Code se valida por separado:

```bash
cd editors/vscode
npm ci
npm run compile
```

## Estructura

```text
crates/
├── forge-cli/    comandos y servidor del dashboard
├── forge-core/   configuración, DAG, caché, ejecución y telemetría
├── forge-deps/   Maven Central y PyPI
├── forge-langs/  Java, Kotlin, Python y pruebas
└── forge-lsp/    servidor LSP para forge.toml

forge-dashboard/  frontend React/Vite
editors/vscode/   extensión de VS Code
schemas/          JSON Schema de forge.toml
tests/            proyectos de integración manual
```

## Validación obligatoria

Antes de enviar cambios Rust:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo audit
```

Para el dashboard:

```bash
cd forge-dashboard
npm ci
npm run typecheck
npm run build
npm audit --audit-level=high
```

Para la extensión:

```bash
cd editors/vscode
npm ci
npm run compile
npm audit --audit-level=high
```

La CI repite estas verificaciones en Linux, macOS y Windows. No reduzcas una comprobación bloqueante para hacer pasar un cambio; corrige la causa o documenta por qué debe cambiar la política.

## Reglas de diseño

- Mantén `forge.toml` estricto: toda clave nueva debe existir en los tipos Rust, el JSON Schema, las plantillas y la documentación.
- Valida rutas antes de crear, extraer, mover o borrar archivos.
- Toda descarga debe tener timeout, límite de tamaño, redirecciones limitadas y verificación de integridad cuando el repositorio publique un digest.
- Propaga los códigos de salida: una compilación, prueba, hook o tarea fallida no puede reportarse como exitosa.
- No añadas telemetría externa ni expongas el dashboard fuera de localhost sin una propuesta explícita de seguridad y privacidad.
- Añade pruebas de regresión para validación, grafos, caché y parsers.
- Evita métricas de rendimiento en la documentación si no provienen de un benchmark reproducible.

## Commits y pull requests

Usa mensajes breves y descriptivos, por ejemplo:

```text
fix: reject symlinks in remote cache archives
feat: add Kotlin compiler option
docs: document remote cache protocol
```

El pull request debe incluir:

- problema y solución;
- riesgos o incompatibilidades;
- pruebas ejecutadas;
- capturas si modifica el dashboard;
- documentación y changelog cuando el comportamiento público cambia.

## Releases

Las releases se generan al publicar un tag `v*`. El workflow compila los targets soportados, publica SHA-256 y genera una atestación de procedencia. Solo los mantenedores deben crear tags oficiales.

Al contribuir aceptas que tu código se distribuya bajo la [licencia MIT](LICENSE).
