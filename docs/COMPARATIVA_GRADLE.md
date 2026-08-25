# FORGE y Gradle: alcance y decisiones de diseño

Gradle es una plataforma madura del ecosistema JVM, con un modelo extensible, plugins, toolchains, publicación y compatibilidad con builds empresariales. FORGE `0.11` es un proyecto joven que explora una alternativa más pequeña para Java, Kotlin y Python.

Esta comparación describe decisiones arquitectónicas; no afirma superioridad de rendimiento. Cualquier cifra de tiempo o memoria debe provenir de benchmarks reproducibles sobre el mismo proyecto y equipo.

| Aspecto | Gradle | FORGE 0.11 |
|---|---|---|
| Runtime | JVM y Gradle Daemon | Binario nativo escrito en Rust |
| Configuración | DSL Groovy/Kotlin | TOML declarativo y estricto |
| Ecosistema | Amplio catálogo de plugins | Sin sistema público de plugins |
| Lenguajes principales | Ecosistema JVM y plugins | Java, Kotlin y Python integrados |
| Dependencias JVM | Modelo maduro, repositorios y metadatos completos | Maven Central con un subconjunto del modelo POM |
| Tareas | Grafo altamente extensible | DAG acotado, hooks y comandos personalizados |
| Caché local | Caché de build y dependencias | Caché incremental y CAS global |
| Caché remota | Caché de build configurable | Protocolo HTTP(S) simple para archivos `tar.gz` |
| IDE | Integraciones maduras | Generadores iniciales, JSON Schema y LSP |
| Observabilidad | Build scans y herramientas del ecosistema | Dashboard local y eventos SSE |

## Qué intenta simplificar FORGE

- Un conjunto pequeño de comandos consistentes entre tres lenguajes.
- Configuración declarativa que rechaza claves desconocidas.
- Binario único para el motor, la CLI y el dashboard local.
- Caché compartida entre proyectos mediante contenido SHA-256.
- Controles de integridad y límites de extracción/descarga incluidos en el motor.

## Qué todavía no reemplaza

FORGE no implementa el modelo efectivo completo de Maven, perfiles avanzados, publicación de artefactos, toolchains equivalentes, variantes complejas, catálogo de plugins ni la amplitud de integraciones de Gradle. Para proyectos JVM grandes o builds que dependan de plugins, Gradle continúa siendo la opción más completa.

La dirección de FORGE es mantener un núcleo pequeño y verificable. Una futura extensibilidad por plugins requiere antes definir capacidades, permisos, autenticidad de módulos y aislamiento; el prototipo WebAssembly anterior no formaba parte del producto y fue retirado en `0.11.0`.
