# Política de seguridad de FORGE

## Versiones con soporte

FORGE es un proyecto `0.x`. Solo la versión menor más reciente recibe correcciones de seguridad.

| Versión | Estado |
|---|---|
| `0.11.x` | Soportada |
| `<= 0.10.x` | Sin soporte; actualiza a la última release |

## Reportar una vulnerabilidad

No publiques detalles explotables en un issue, discusión o pull request.

Usa el formulario privado de [GitHub Security Advisories](https://github.com/enri312/forge/security/advisories/new). Incluye, cuando sea posible:

- versión, sistema operativo y arquitectura afectados;
- descripción e impacto;
- pasos mínimos para reproducir;
- prueba de concepto sin datos ni sistemas de terceros;
- mitigación o corrección propuesta.

Si el formulario privado no está disponible, abre un issue que solicite únicamente un canal privado, sin revelar la vulnerabilidad.

## Respuesta y divulgación

El mantenedor intentará confirmar la recepción en un plazo de 48 horas y compartir una evaluación inicial en siete días. Los plazos pueden variar porque el proyecto se mantiene de forma comunitaria.

La divulgación pública se coordinará después de disponer de una corrección o mitigación. Se reconocerá al reportante si lo desea.

## Alcance prioritario

- traversal, sobrescritura o extracción insegura de archivos;
- ejecución de comandos inesperada mediante `forge.toml`, dependencias, hooks o tareas;
- validación insuficiente de artefactos Maven, PyPI, JUnit o releases;
- exposición de tokens o uso inseguro de la caché remota;
- acceso de red no previsto desde el dashboard local;
- vulnerabilidades en la cadena de suministro o dependencias.

Los hooks y tareas ejecutan deliberadamente comandos definidos por el propietario del proyecto. Un `forge.toml` no confiable debe tratarse como código no confiable.

## Verificación de releases

Cada release publica un archivo `.sha256` junto a cada paquete. Verifica el checksum antes de ejecutar el binario. Los instaladores oficiales realizan esta comprobación automáticamente.

La procedencia de los artefactos también puede validarse con GitHub CLI:

```bash
gh attestation verify <archivo> --repo enri312/forge
```
