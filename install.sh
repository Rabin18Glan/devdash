#!/usr/bin/env bash
set -e

# ⚡ DevDash Installer
REPO="Rabin18Glan/devdash"
BIN_NAME="devdash"

# Colors
CYAN="\033[1;36m"
GREEN="\033[1;32m"
YELLOW="\033[1;33m"
RED="\033[1;31m"
RESET="\033[0m"

echo -e "${CYAN}⚡ Installing DevDash...${RESET}"

# Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo -e "${RED}Unsupported Linux architecture: $ARCH${RESET}"; exit 1 ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
      *) echo -e "${RED}Unsupported macOS architecture: $ARCH${RESET}"; exit 1 ;;
    esac
    ;;
  *)
    echo -e "${RED}Unsupported operating system: $OS${RESET}"
    exit 1
    ;;
esac

# Find latest release tag if not provided
if [ -z "$VERSION" ]; then
  VERSION=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
  if [ -z "$VERSION" ]; then
    VERSION="v0.1.0"
  fi
fi

ARCHIVE_NAME="${BIN_NAME}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_NAME}"

# Installation directory
if [ -n "$INSTALL_DIR" ]; then
  DEST_DIR="$INSTALL_DIR"
elif [ "$(id -u)" -eq 0 ]; then
  DEST_DIR="/usr/local/bin"
else
  DEST_DIR="$HOME/.local/bin"
fi

mkdir -p "$DEST_DIR"

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo -e "📦 Downloading ${CYAN}${BIN_NAME} ${VERSION}${RESET} for ${TARGET}..."
curl -fL --progress-bar "$DOWNLOAD_URL" -o "$TMP_DIR/$ARCHIVE_NAME"

echo -e "🔧 Extracting to ${DEST_DIR}..."
tar -xzf "$TMP_DIR/$ARCHIVE_NAME" -C "$TMP_DIR"

install -m 755 "$TMP_DIR/$BIN_NAME" "$DEST_DIR/$BIN_NAME"

# Also symlink to ~/.cargo/bin if present in PATH to prevent bash hashing collisions
if [ -d "$HOME/.cargo/bin" ] && [ "$DEST_DIR" != "$HOME/.cargo/bin" ]; then
  ln -sf "$DEST_DIR/$BIN_NAME" "$HOME/.cargo/bin/$BIN_NAME" 2>/dev/null || true
fi

echo -e "${GREEN}✔ DevDash installed successfully to ${DEST_DIR}/${BIN_NAME}!${RESET}"

# Check if DEST_DIR is in PATH
case ":$PATH:" in
  *":$DEST_DIR:"*) ;;
  *)
    echo -e "${YELLOW}⚠️  Note: ${DEST_DIR} is not in your PATH.${RESET}"
    echo -e "Add this to your shell config (~/.bashrc, ~/.zshrc):"
    echo -e "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac

# Clear bash command hash table if running interactively
hash -r 2>/dev/null || true

echo -e "${CYAN}Run 'devdash' to get started! 🚀${RESET}"
