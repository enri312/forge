#!/usr/bin/env bash
# =============================================================================
# 🔥 FORGE — Installer (Unix)
# =============================================================================
# Descarga e instala la última versión precompilada del Build System FORGE.
# Uso: curl -fsSL https://raw.githubusercontent.com/enri312/forge/main/install.sh | bash
# =============================================================================

set -e

REPO="enri312/forge"
INSTALL_DIR="$HOME/.cargo/bin"
BIN_NAME="forge"

echo "🔥 Instalando FORGE Build System..."

# 1. Detectar SO y Arquitectura
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [ "$OS" = "linux" ]; then
    OS="unknown-linux-gnu"
elif [ "$OS" = "darwin" ]; then
    OS="apple-darwin"
else
    echo "❌ OS no soportado: $OS"
    exit 1
fi

if [ "$ARCH" = "x86_64" ] || [ "$ARCH" = "amd64" ]; then
    ARCH="x86_64"
elif [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
    ARCH="aarch64"
else
    echo "❌ Arquitectura no soportada: $ARCH"
    exit 1
fi

TARGET="${ARCH}-${OS}"

# 2. Obtener URL de la última release en GitHub API
echo "🔍 Buscando última versión (Target: $TARGET)..."
LATEST_RELEASE_URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep "browser_download_url.*${TARGET}.tar.gz" | cut -d '"' -f 4)

if [ -z "$LATEST_RELEASE_URL" ]; then
    echo "⚠️  Artefacto precompilado no encontrado para ${TARGET} en la última release."
    echo "Intenta usando Cargo localmente: cargo install forge-cli"
    exit 1
fi

echo "⬇️  Descargando desde: $LATEST_RELEASE_URL"

# 3. Descargar y descomprimir
TMP_DIR=$(mktemp -d)
curl -sL "$LATEST_RELEASE_URL" | tar xz -C "$TMP_DIR"

# 4. Instalar en el sistema
mkdir -p "$INSTALL_DIR"
mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/$BIN_NAME"

rm -rf "$TMP_DIR"

echo ""
echo "✅ FORGE instalado exitosamente en $INSTALL_DIR/$BIN_NAME"
echo ""
echo "🚀 Asegúrate de tener '$INSTALL_DIR' en tu \$PATH."
echo "Prueba correr: forge --version"
