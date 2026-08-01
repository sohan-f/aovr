#!/bin/sh
set -e

REPO="sohan-f/aovr"
BINARY_NAME="aovr"

BOLD='\033[1m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
RESET='\033[0m'

info() { printf "${BLUE}▶${RESET} ${BOLD}%s${RESET}\n" "$1"; }
success() { printf "${GREEN}▶${RESET} ${BOLD}%s${RESET}\n" "$1"; }
warn() { printf "${YELLOW}[WARN]${RESET} %s\n" "$1"; }
error() { printf "${RED}[ERROR]${RESET} %s\n" "$1"; exit 1; }

printf "\n${BOLD}Installing AOVR (Android Overlay Manager TUI)${RESET}\n\n"

ARCH=$(uname -m)
case "$ARCH" in
  aarch64|arm64)
    ;;
  *)
    error "Architecture '$ARCH' is not supported. AOVR currently supports ARM64 (aarch64) devices."
    ;;
esac

if command -v curl >/dev/null 2>&1; then
  FETCH="curl -fsSL"
else
  error "curl is required to install AOVR. Please install curl first (e.g., 'pkg install curl')."
fi

info "Fetching latest release information from GitHub..."
LATEST_JSON=$($FETCH "https://api.github.com/repos/${REPO}/releases/latest")

DEB_URL=$(echo "$LATEST_JSON" | grep -o 'https://[^\"]*aarch64\.deb' | head -n 1 || true)
RAW_BIN_URL=$(echo "$LATEST_JSON" | grep -o 'https://[^\"]*/releases/download/[^\"]*/aovr"' | tr -d '"' | head -n 1 || true)

if command -v dpkg >/dev/null 2>&1 && [ -n "$DEB_URL" ]; then
  info "Downloading Termux package (.deb)..."
  TMP_DEB=$(mktemp /tmp/aovr_XXXXXX.deb 2>/dev/null || mktemp ./aovr_XXXXXX.deb)
  curl -fsSL "$DEB_URL" -o "$TMP_DEB"

  info "Installing AOVR using dpkg..."
  dpkg -i "$TMP_DEB" || apt-get install -f -y "$TMP_DEB"
  rm -f "$TMP_DEB"

  printf "\n"
  success "AOVR successfully installed!"
  info "Run 'aovr' to start managing your overlays."
  exit 0
fi

if [ -z "$RAW_BIN_URL" ]; then
  error "Failed to locate release binary assets for ${REPO}."
fi

info "Downloading compiled binary..."
TMP_BIN=$(mktemp /tmp/aovr_bin_XXXXXX 2>/dev/null || mktemp ./aovr_bin_XXXXXX)
curl -fsSL "$RAW_BIN_URL" -o "$TMP_BIN"
chmod 755 "$TMP_BIN"

if [ -n "$PREFIX" ] && [ -d "$PREFIX/bin" ]; then
  TARGET_DIR="$PREFIX/bin"
elif [ -d "/data/data/com.termux/files/usr/bin" ]; then
  TARGET_DIR="/data/data/com.termux/files/usr/bin"
elif [ -d "/usr/local/bin" ]; then
  TARGET_DIR="/usr/local/bin"
else
  TARGET_DIR="/data/local/tmp"
fi

info "Installing binary to ${TARGET_DIR}/${BINARY_NAME}..."
mv "$TMP_BIN" "${TARGET_DIR}/${BINARY_NAME}"

printf "\n"
success "AOVR successfully installed to ${TARGET_DIR}/${BINARY_NAME}!"
info "Run '${BINARY_NAME}' to start."
