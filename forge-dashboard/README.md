# FORGE Dashboard

Frontend React/Vite embebido en el binario `forge`. Muestra eventos reales de compilación y caché recibidos desde el backend Rust mediante Server-Sent Events (SSE).

## Desarrollo local

```bash
npm install
npm run lint
npm run dev
```

En desarrollo, el endpoint `/api/events` debe apuntar a una instancia de `forge dashboard`. Para generar los archivos que Rust incorpora al binario:

```bash
npm run build
cargo build -p cyrce-forge-cli
```

El servidor embebido escucha únicamente en `127.0.0.1` y añade una política CSP y cabeceras de seguridad. No requiere claves de API ni servicios de IA.
