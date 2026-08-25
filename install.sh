#!/usr/bin/env bash
# Instala la última release de FORGE y verifica el SHA-256 publicado.

set -euo pipefail

readonly FORGE_REPO="enri312/forge"
readonly FORGE_INSTALL_DIR="${FORGE_INSTALL_DIR:-$HOME/.cargo/bin}"

command -v curl >/dev/null 2>&1 || {
    echo "Error: curl es necesario para instalar FORGE." >&2
    exit 1
}

case "$(uname -s)" in
    Linux) forge_os="unknown-linux-gnu" ;;
    Darwin) forge_os="apple-darwin" ;;
    *)
        echo "Error: sistema operativo no soportado: $(uname -s)." >&2
        exit 1
        ;;
esac

case "$(uname -m)" in
    x86_64|amd64) forge_arch="x86_64" ;;
    arm64|aarch64) forge_arch="aarch64" ;;
    *)
        echo "Error: arquitectura no soportada: $(uname -m)." >&2
        exit 1
        ;;
esac

if [ "$forge_os" = "unknown-linux-gnu" ] && [ "$forge_arch" != "x86_64" ]; then
    echo "Error: la release oficial de Linux solo está disponible para x86_64." >&2
    echo "Instala desde fuente: cargo install --git https://github.com/$FORGE_REPO.git cyrce-forge-cli" >&2
    exit 1
fi

readonly forge_target="${forge_arch}-${forge_os}"
readonly forge_asset="forge-${forge_target}.tar.gz"
readonly forge_base_url="https://github.com/${FORGE_REPO}/releases/latest/download"
forge_temp_dir="$(mktemp -d)"
trap 'rm -rf -- "$forge_temp_dir"' EXIT INT TERM

echo "Instalando FORGE para ${forge_target}..."

curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
    "${forge_base_url}/${forge_asset}" \
    --output "${forge_temp_dir}/${forge_asset}"
curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
    "${forge_base_url}/${forge_asset}.sha256" \
    --output "${forge_temp_dir}/${forge_asset}.sha256"

forge_expected_hash="$(awk 'NR == 1 {print tolower($1)}' "${forge_temp_dir}/${forge_asset}.sha256")"
if ! printf '%s' "$forge_expected_hash" | grep -Eq '^[0-9a-f]{64}$'; then
    echo "Error: el archivo de checksum publicado no es válido." >&2
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    forge_actual_hash="$(sha256sum "${forge_temp_dir}/${forge_asset}" | awk '{print tolower($1)}')"
elif command -v shasum >/dev/null 2>&1; then
    forge_actual_hash="$(shasum -a 256 "${forge_temp_dir}/${forge_asset}" | awk '{print tolower($1)}')"
else
    echo "Error: se necesita sha256sum o shasum para verificar la descarga." >&2
    exit 1
fi

if [ "$forge_actual_hash" != "$forge_expected_hash" ]; then
    echo "Error: el SHA-256 de ${forge_asset} no coincide; no se instalará." >&2
    exit 1
fi

tar -xzf "${forge_temp_dir}/${forge_asset}" -C "$forge_temp_dir" forge
mkdir -p -- "$FORGE_INSTALL_DIR"
install -m 0755 "${forge_temp_dir}/forge" "${FORGE_INSTALL_DIR}/forge"

echo "FORGE se instaló en ${FORGE_INSTALL_DIR}/forge"
case ":$PATH:" in
    *":${FORGE_INSTALL_DIR}:"*) ;;
    *) echo "Añade ${FORGE_INSTALL_DIR} a PATH para ejecutar 'forge'." ;;
esac
