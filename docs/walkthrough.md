# 🔥 FORGE — Walkthrough del Proyecto

## Resumen

Se creó **FORGE**, un build system de nueva generación escrito en Rust, diseñado para superar a Gradle. El proyecto está completo y listo para compilar.

## Archivos Creados (27 archivos)

### Raíz del Proyecto
| Archivo | Propósito |
|---|---|
| [Cargo.toml](file:///E:/CYRCE/Cargo.toml) | Workspace Cargo con 4 crates y dependencias compartidas |
| [README.md](file:///E:/CYRCE/README.md) | Documentación completa para GitHub |
| [CONTRIBUTING.md](file:///E:/CYRCE/CONTRIBUTING.md) | Guía de contribución |
| [LICENSE](file:///E:/CYRCE/LICENSE) | Licencia MIT |
| [.gitignore](file:///E:/CYRCE/.gitignore) | Archivos a ignorar |

### `forge-core` — Motor Principal (7 archivos)
| Archivo | Propósito |
|---|---|
| [lib.rs](file:///E:/CYRCE/crates/forge-core/src/lib.rs) | Punto de entrada del crate |
| [error.rs](file:///E:/CYRCE/crates/forge-core/src/error.rs) | 12 tipos de error descriptivos |
| [config.rs](file:///E:/CYRCE/crates/forge-core/src/config.rs) | Parser forge.toml + validación + 3 tests |
| [dag.rs](file:///E:/CYRCE/crates/forge-core/src/dag.rs) | DAG con ciclos, topología, paralelismo + 3 tests |
| [executor.rs](file:///E:/CYRCE/crates/forge-core/src/executor.rs) | Ejecutor paralelo async con tokio |
| [cache.rs](file:///E:/CYRCE/crates/forge-core/src/cache.rs) | Caché SHA-256 incremental + 3 tests |
| 3x plantillas TOML | Plantillas para Java, Kotlin, Python |

### `forge-langs` — Módulos de Lenguaje (4 archivos)
| Archivo | Propósito |
|---|---|
| [java.rs](file:///E:/CYRCE/crates/forge-langs/src/java.rs) | javac + jar + java |
| [kotlin.rs](file:///E:/CYRCE/crates/forge-langs/src/kotlin.rs) | kotlinc + jar + java |
| [python.rs](file:///E:/CYRCE/crates/forge-langs/src/python.rs) | venv + pip + pytest |

### `forge-deps` — Dependencias (3 archivos)
| Archivo | Propósito |
|---|---|
| [maven.rs](file:///E:/CYRCE/crates/forge-deps/src/maven.rs) | Descarga JARs de Maven Central |
| [pypi.rs](file:///E:/CYRCE/crates/forge-deps/src/pypi.rs) | Verifica paquetes en PyPI |

### `forge-cli` — CLI (1 archivo)
| Archivo | Propósito |
|---|---|
| [main.rs](file:///E:/CYRCE/crates/forge-cli/src/main.rs) | 7 comandos: init, build, run, test, clean, deps, info |

### Proyectos de Ejemplo (6 archivos)
- `tests/java_project/` — Proyecto Java simple con `Main.java`
- `tests/kotlin_project/` — Proyecto Kotlin simple con `Main.kt`
- `tests/python_project/` — Proyecto Python simple con `main.py`

## Versiones Soportadas
- **Java**: 17 a 25
- **Kotlin**: 2.1+
- **Python**: 3.10 a 3.14.3

## Próximo Paso: Compilar

Abre un terminal (PowerShell o CMD) en `E:\CYRCE` y ejecuta:

```bash
cargo build
```

Si hay errores de compilación, los corregiremos juntos.
