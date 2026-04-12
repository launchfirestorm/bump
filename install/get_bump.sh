#!/usr/bin/env bash
#
# Bump installer — Linux, macOS, and other Unix-like systems (including WSL).
# Downloads the latest release from GitHub.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/launchfirestorm/bump/main/install/get_bump.sh | bash
#
# CI: set GITHUB_TOKEN or GH_TOKEN so api.github.com is authenticated (avoids HTTP 403 on shared runners).
#
# Native Windows (Git Bash / MSYS / Cygwin): use PowerShell instead:
#   irm https://raw.githubusercontent.com/launchfirestorm/bump/main/install/get_bump.ps1 | iex
#
set -eu
# shellcheck disable=SC3040
(set -o pipefail 2>/dev/null) && set -o pipefail

REPO="launchfirestorm/bump"
BINARY_NAME="bump"
TARGET_PATH=""

# Optional: GITHUB_TOKEN (GitHub Actions) or GH_TOKEN (gh CLI) — Bearer for GitHub API / release downloads.
init_github_curl_auth() {
  GITHUB_CURL_AUTH=()
  if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    GITHUB_CURL_AUTH+=(-H "Authorization: Bearer ${GITHUB_TOKEN}")
  elif [[ -n "${GH_TOKEN:-}" ]]; then
    GITHUB_CURL_AUTH+=(-H "Authorization: Bearer ${GH_TOKEN}")
  fi
}

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() {
  echo -e "${RED}[ERROR]${NC} $1"
  exit 1
}

get_latest_release_tag() {
  local json tag
  json=$(curl -fsSL "${GITHUB_CURL_AUTH[@]}" -H "Accept: application/vnd.github+json" -H "User-Agent: bump-install" \
    "https://api.github.com/repos/${REPO}/releases/latest") || return 1
  tag=$(printf '%s\n' "$json" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
  [[ -n "$tag" ]] || return 1
  printf '%s\n' "$tag"
}

detect_platform() {
  local kernel machine platform
  kernel=$(uname -s | cut -d- -f1)
  machine=$(uname -m)
  platform="${machine}-${kernel}"

  case $platform in
    x86_64-MINGW64_NT | x86_64-MSYS_NT | x86_64-CYGWIN_NT | aarch64-MINGW64_NT | aarch64-MSYS_NT | arm64-MINGW64_NT)
      error "Native Windows is not supported by this shell script. Use PowerShell: irm https://raw.githubusercontent.com/launchfirestorm/bump/main/install/get_bump.ps1 | iex"
      ;;
    x86_64-Linux)
      OS="linux"
      ARCH="amd64"
      INSTALL_DIR="/usr/local/bin"
      ;;
    aarch64-Linux)
      OS="linux"
      ARCH="arm64"
      INSTALL_DIR="/usr/local/bin"
      ;;
    x86_64-Darwin)
      OS="macos"
      ARCH="amd64"
      INSTALL_DIR="/usr/local/bin"
      ;;
    arm64-Darwin)
      OS="macos"
      ARCH="arm64"
      INSTALL_DIR="/usr/local/bin"
      ;;
    *)
      error "Unsupported platform: ${platform}. Supported: Linux (x86_64/aarch64), macOS (x86_64/arm64). On Windows use install/get_bump.ps1."
      ;;
  esac

  info "Platform: ${platform} (OS: ${OS}, architecture: ${ARCH})"
}

check_dependencies() {
  command -v curl >/dev/null || error "Missing required command: curl"
}

install_bump() {
  local asset_name temp_file tag_name download_url target_path

  case "$OS" in
    linux) asset_name="bump-linux-${ARCH}" ;;
    macos) asset_name="bump-macos-${ARCH}" ;;
    *) error "Unsupported OS: $OS" ;;
  esac

  temp_file=$(mktemp) || error "Failed to create temporary file"

  tag_name=$(get_latest_release_tag) || error "Failed to fetch latest release from GitHub API"
  info "Latest release: ${tag_name}"

  download_url="https://github.com/${REPO}/releases/download/${tag_name}/${asset_name}"
  info "Downloading from: ${download_url}"

  curl -fsSL "${GITHUB_CURL_AUTH[@]}" -o "$temp_file" "$download_url" || error "Download failed"

  [[ -s "$temp_file" ]] || error "Downloaded file is empty"
  chmod +x "$temp_file" || error "Failed to make binary executable"

  if [[ ! -d "$INSTALL_DIR" ]]; then
    if [[ -w "$(dirname "$INSTALL_DIR")" ]]; then
      mkdir -p "$INSTALL_DIR" || error "Failed to create ${INSTALL_DIR}"
    else
      sudo mkdir -p "$INSTALL_DIR" || error "Failed to create ${INSTALL_DIR} (try with sudo)"
    fi
  fi

  target_path="${INSTALL_DIR}/${BINARY_NAME}"

  if [[ -w "$INSTALL_DIR" ]]; then
    mv "$temp_file" "$target_path" || error "Failed to install to ${target_path}"
  else
    sudo mv "$temp_file" "$target_path" || error "Failed to install to ${target_path} (try with sudo)"
  fi

  TARGET_PATH="$target_path"
  success "${BINARY_NAME} installed to ${target_path}"
}

verify_installation() {
  local version

  if [[ -z "${TARGET_PATH}" ]]; then
    warning "Could not verify: install path was not set."
    return
  fi
  if [[ ! -f "$TARGET_PATH" ]]; then
    warning "Expected binary missing at ${TARGET_PATH}"
    return
  fi

  version=$("$TARGET_PATH" --version 2>/dev/null) || version="unknown"
  success "Installation complete. Version: ${version} (${TARGET_PATH})"

  if ! command -v "$BINARY_NAME" >/dev/null 2>&1; then
    warning "Add ${INSTALL_DIR} to your PATH, e.g.: export PATH=\"${INSTALL_DIR}:\$PATH\""
  fi
}

main() {
  echo "Bump installer (Unix)"
  echo ""

  init_github_curl_auth
  detect_platform
  check_dependencies
  install_bump
  verify_installation

  echo ""
  echo "Done. Run 'bump --help' to get started."
}

main "$@"
