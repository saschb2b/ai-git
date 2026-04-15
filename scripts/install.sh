#!/bin/sh
# aig installer — detects platform and installs the latest release.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/saschb2b/ai-git/main/scripts/install.sh | sh
#
set -e

REPO="saschb2b/ai-git"
INSTALL_DIR="/usr/local/bin"

# Colors (if terminal supports them)
if [ -t 1 ]; then
  BOLD="\033[1m"
  DIM="\033[2m"
  CYAN="\033[36m"
  GREEN="\033[32m"
  RED="\033[31m"
  RESET="\033[0m"
else
  BOLD="" DIM="" CYAN="" GREEN="" RED="" RESET=""
fi

info()  { printf "${CYAN}>${RESET} %s\n" "$1"; }
ok()    { printf "${GREEN}>${RESET} %s\n" "$1"; }
err()   { printf "${RED}error:${RESET} %s\n" "$1" >&2; exit 1; }

# --- Detect platform ---

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)   PLATFORM="linux" ;;
  Darwin)  PLATFORM="macos" ;;
  *)       err "unsupported OS: $OS (try installing from source with cargo)" ;;
esac

case "$ARCH" in
  x86_64|amd64)   ARCH_LABEL="x86_64" ;;
  aarch64|arm64)   ARCH_LABEL="aarch64" ;;
  *)               err "unsupported architecture: $ARCH" ;;
esac

ASSET="aig-${ARCH_LABEL}-${PLATFORM}.tar.gz"
URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

# --- Check dependencies ---

if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
  err "curl or wget is required to download aig"
fi

if ! command -v tar >/dev/null 2>&1; then
  err "tar is required to extract aig"
fi

# --- Download and install ---

printf "\n"
printf "  ${BOLD}aig installer${RESET}\n"
printf "  ${DIM}Version Control for the AI Age${RESET}\n"
printf "\n"

info "Detected platform: ${PLATFORM} (${ARCH_LABEL})"
info "Downloading ${ASSET}..."

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$URL" -o "$TMPDIR/$ASSET" || err "download failed — check https://github.com/${REPO}/releases"
else
  wget -q "$URL" -O "$TMPDIR/$ASSET" || err "download failed — check https://github.com/${REPO}/releases"
fi

info "Extracting..."
tar xzf "$TMPDIR/$ASSET" -C "$TMPDIR"

# --- Install binary ---

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMPDIR/aig" "$INSTALL_DIR/aig"
  ok "Installed to ${INSTALL_DIR}/aig"
else
  info "Installing to ${INSTALL_DIR} (requires sudo)..."
  sudo mv "$TMPDIR/aig" "$INSTALL_DIR/aig" || err "failed to install — try: sudo mv $TMPDIR/aig $INSTALL_DIR/aig"
  ok "Installed to ${INSTALL_DIR}/aig"
fi

chmod +x "$INSTALL_DIR/aig"

# --- Verify ---

VERSION="$("$INSTALL_DIR/aig" --help 2>/dev/null | head -1 || echo "unknown")"
printf "\n"
ok "aig is ready!"
printf "\n"
printf "  Get started:\n"
printf "    ${CYAN}cd your-project${RESET}\n"
printf "    ${CYAN}aig init --import${RESET}\n"
printf "    ${CYAN}aig log${RESET}\n"
printf "\n"
printf "  Docs: ${DIM}https://saschb2b.github.io/ai-git/guide/getting-started${RESET}\n"
printf "\n"
