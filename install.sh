#!/usr/bin/env bash
# ezCMD automated installer for Linux and macOS
# Works across Bash, Zsh, and Fish terminals.

set -e

REPO="ItzEvhyx/ezCMD"
INSTALL_DIR="${HOME}/.local/bin"
BIN_NAME="ezcmd"
TARGET_PATH="${INSTALL_DIR}/${BIN_NAME}"

echo "Installing ezCMD..."
mkdir -p "${INSTALL_DIR}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || pwd)"
if [ -f "${SCRIPT_DIR}/target/release/ezcmd" ]; then
    echo "Installing from local release build (${SCRIPT_DIR}/target/release/ezcmd)..."
    rm -f "${TARGET_PATH}"
    cp "${SCRIPT_DIR}/target/release/ezcmd" "${TARGET_PATH}"
    chmod +x "${TARGET_PATH}"
elif [ -f "${SCRIPT_DIR}/ezcmd" ]; then
    echo "Installing from local binary (${SCRIPT_DIR}/ezcmd)..."
    rm -f "${TARGET_PATH}"
    cp "${SCRIPT_DIR}/ezcmd" "${TARGET_PATH}"
    chmod +x "${TARGET_PATH}"
else
    # Detect OS & Arch for prebuilt release download
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "${OS}" in
        Linux*)
            case "${ARCH}" in
                x86_64) ASSET_TARGET="x86_64-unknown-linux-musl" ;;
                aarch64|arm64) ASSET_TARGET="aarch64-unknown-linux-musl" ;;
                *) echo "Unsupported Linux architecture: ${ARCH}"; exit 1 ;;
            esac
            ;;
        Darwin*)
            case "${ARCH}" in
                x86_64) ASSET_TARGET="x86_64-apple-darwin" ;;
                arm64|aarch64) ASSET_TARGET="aarch64-apple-darwin" ;;
                *) echo "Unsupported macOS architecture: ${ARCH}"; exit 1 ;;
            esac
            ;;
        *)
            echo "Unsupported OS: ${OS}. Please use install.ps1 on Windows."
            exit 1
            ;;
    esac

    TMP_DIR="$(mktemp -d)"

    # Try GitHub API first to find exact asset URL
    TAR_URL=""
    LATEST_JSON=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null || true)
    if [ -n "$LATEST_JSON" ]; then
        TAR_URL=$(echo "$LATEST_JSON" | grep "browser_download_url" | grep -v "\.sha256" | grep "${ASSET_TARGET}\.tar\.gz" | head -n 1 | cut -d '"' -f 4 || true)
    fi

    # Fallback to direct download URLs
    if [ -z "$TAR_URL" ]; then
        TAR_URL="https://github.com/${REPO}/releases/download/v2.0.0/ezcmd-v2.0.0-${ASSET_TARGET}.tar.gz"
    fi

    echo "Downloading ${TAR_URL}..."
    if ! curl -fsSL "${TAR_URL}" -o "${TMP_DIR}/ezcmd.tar.gz" 2>/dev/null; then
        FALLBACK_URL="https://github.com/${REPO}/releases/download/v2.0.0/ezcmd-v2.0.0-${ASSET_TARGET}.tar.gz"
        echo "Trying fallback ${FALLBACK_URL}..."
        curl -fsSL "${FALLBACK_URL}" -o "${TMP_DIR}/ezcmd.tar.gz"
    fi

    tar -xzf "${TMP_DIR}/ezcmd.tar.gz" -C "${TMP_DIR}"
    rm -f "${TARGET_PATH}"
    mv "${TMP_DIR}/ezcmd" "${TARGET_PATH}"
    chmod +x "${TARGET_PATH}"
    rm -rf "${TMP_DIR}"
fi

echo ""
echo "✓ ezcmd was successfully installed to ${TARGET_PATH}!"
echo ""

# Check if ~/.local/bin is in PATH
case ":${PATH}:" in
    *:"${INSTALL_DIR}":*) ;;
    *)
        echo "NOTE: ${INSTALL_DIR} is not currently in your PATH."
        echo "Add it with:"
        echo "  • Fish:        set -Ua fish_user_paths ~/.local/bin"
        echo "  • Bash:        echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc && source ~/.bashrc"
        echo "  • Zsh:         echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc && source ~/.zshrc"
        echo ""
        ;;
esac

# Create configuration marker
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/ezcmd"
mkdir -p "${CONFIG_DIR}"
echo "installed" > "${CONFIG_DIR}/.installed"

# Automatically show the welcome banner
"${TARGET_PATH}" --welcome
