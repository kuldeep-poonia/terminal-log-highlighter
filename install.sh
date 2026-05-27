#!/bin/bash
set -e

# --- Configuration ---
GITHUB_REPO="kuldeep-poonia/terminal-log-highlighter"          # Replace with your GitHub username/repo
BINARY_NAME="sentinel"
INSTALL_DIR="/usr/local/bin"

# --- Detect OS and architecture ---
OS=$(uname -s)
ARCH=$(uname -m)

# Map OS and ARCH to the naming convention used in your release assets
case "$OS" in
    Linux)  OS="Linux" ;;
    Darwin) OS="Darwin" ;;
    *)      echo "Unsupported operating system: $OS"; exit 1 ;;
esac

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *)      echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# --- Download URL ---
DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/${BINARY_NAME}-${OS}-${ARCH}"

echo "Downloading ${BINARY_NAME} for ${OS}-${ARCH} ..."
curl -fsSL "$DOWNLOAD_URL" -o "/tmp/${BINARY_NAME}"

# --- Install ---
echo "Installing to ${INSTALL_DIR}/${BINARY_NAME} ..."
sudo mv "/tmp/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

# --- Verify ---
if command -v "${BINARY_NAME}" &> /dev/null; then
    echo "✅ Installation successful! Run '${BINARY_NAME}' to start."
else
    echo "⚠️ Installation may have failed – check your PATH."
fi