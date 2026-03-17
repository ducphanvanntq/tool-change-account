#!/bin/bash
set -e

REPO="ducphanvanntq/tool-change-account"
BINARY_NAME="rust-cli"
INSTALL_DIR="/usr/local/bin"
INSTALL_NAME="tool-change-account"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin)
    case "$ARCH" in
      x86_64)  ARTIFACT="tool-change-account-macos-x86_64.tar.gz" ;;
      arm64)   ARTIFACT="tool-change-account-macos-aarch64.tar.gz" ;;
      *)       error "Unsupported architecture: $ARCH" ;;
    esac
    ;;
  Linux)
    case "$ARCH" in
      x86_64)  ARTIFACT="tool-change-account-linux-x86_64.tar.gz" ;;
      *)       error "Unsupported architecture: $ARCH" ;;
    esac
    ;;
  *)
    error "Unsupported OS: $OS. Use the PowerShell script for Windows."
    ;;
esac

LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
  error "Cannot fetch latest release tag"
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARTIFACT}"

info "OS: $OS | Arch: $ARCH"
info "Version: $LATEST_TAG"
info "Downloading: $ARTIFACT"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fSL "$DOWNLOAD_URL" -o "$TMP_DIR/$ARTIFACT" || error "Download failed"

info "Extracting..."
tar xzf "$TMP_DIR/$ARTIFACT" -C "$TMP_DIR"

info "Installing to $INSTALL_DIR/$INSTALL_NAME"
mv "$TMP_DIR/$BINARY_NAME" "$INSTALL_DIR/$INSTALL_NAME"
chmod +x "$INSTALL_DIR/$INSTALL_NAME"

info "✅ Installed successfully!"
echo ""
$INSTALL_DIR/$INSTALL_NAME info
