#!/usr/bin/env bash
# 
# Bump Installer - Cross Platform Installation Script
# 
# This script downloads and installs the latest release of bump
# from GitHub releases.
# 
# Supports: Linux (amd64/arm64), macOS (amd64/arm64), Windows (amd64/arm64)
# 
# Usage: 
#   curl -sSL https://raw.githubusercontent.com/launchfirestorm/bump/main/install.sh | bash
#   
# Or download and run:
#   wget https://raw.githubusercontent.com/launchfirestorm/bump/main/install.sh
#   chmod +x install.sh
#   ./install.sh
#
set -eu

# Check pipefail support in a subshell, ignore if unsupported
# shellcheck disable=SC3040
(set -o pipefail 2> /dev/null) && set -o pipefail

REPO="launchfirestorm/bump"
BINARY_NAME="bump"
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
  # Try to get the latest release tag from GitHub redirect
  local tag
  tag=$(curl -sSL -w '%{redirect_url}' -o /dev/null "https://github.com/${REPO}/releases/latest" | grep -o 'tag/[^"]*' | cut -d'/' -f2)
  
  # Fallback: scrape the releases page directly
  if [[ -z "$tag" ]]; then
    tag=$(curl -sSL "https://github.com/${REPO}/releases" | grep -o 'releases/tag/[^"]*' | head -1 | cut -d'/' -f3)
  fi
  
  echo "$tag"
}

detect_platform() {
  # Detect OS and architecture using uname (POSIX-compliant)
  # bash compiled with MINGW (e.g. git-bash, used in github windows runners),
  # unhelpfully includes a version suffix in `uname -s` output, so handle that.
  # e.g. MINGW64_NT-10-0.19044
  local kernel
  kernel=$(uname -s | cut -d- -f1)
  local machine
  machine=$(uname -m)
  local platform="${machine}-${kernel}"

  case $platform in
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
    x86_64-MINGW64_NT|x86_64-MSYS_NT|x86_64-CYGWIN_NT)
      OS="windows"
      ARCH="amd64"
      BINARY_NAME="bump.exe"
      if [[ -d "/c/Windows/System32" ]]; then
        INSTALL_DIR="/c/Program Files/bump"
      else
        INSTALL_DIR="/usr/local/bin"
      fi
      ;;
    aarch64-MINGW64_NT|aarch64-MSYS_NT|arm64-MINGW64_NT)
      OS="windows"
      ARCH="arm64"
      BINARY_NAME="bump.exe"
      if [[ -d "/c/Windows/System32" ]]; then
        INSTALL_DIR="/c/Program Files/bump"
      else
        INSTALL_DIR="/usr/local/bin"
      fi
      ;;
    *)
      error "Unsupported platform: ${platform}. Supported: Linux (x86_64/aarch64), macOS (x86_64/arm64), Windows (x86_64/aarch64)"
      ;;
  esac
  
  info "Platform: ${platform} (OS: $OS, Architecture: $ARCH)"
}

check_dependencies() {
  # Check for required commands
  command -v curl >/dev/null || error "Missing required command: curl"
  
  # Check for sudo only on Unix-like systems when needed
  if [[ "$OS" != "windows" ]] && [[ ! -w "$INSTALL_DIR" ]]; then
    command -v sudo >/dev/null || error "Missing required command: sudo (needed for installation to $INSTALL_DIR)"
  fi
}

install_bump() {
  # Construct asset name based on OS and architecture
  local asset_name
  case "$OS" in
    linux)
      asset_name="bump-linux-${ARCH}"
      ;;
    macos)
      asset_name="bump-macos-${ARCH}"
      ;;
    windows)
      asset_name="bump-windows-${ARCH}.exe"
      ;;
    *)
      error "Unsupported OS for asset naming: $OS"
      ;;
  esac

  local temp_file
  temp_file=$(mktemp) || error "Failed to create temporary file"

  info "Downloading $asset_name..."

  # Get the latest release tag
  local tag_name
  tag_name=$(get_latest_release_tag) || error "Failed to fetch latest release information"
  
  [[ -z "$tag_name" ]] && error "Could not find latest release tag"
  
  info "Latest release: $tag_name"

  # Construct direct download URL
  local download_url="https://github.com/${REPO}/releases/download/${tag_name}/${asset_name}"
  info "Downloading from: $download_url"
  
  curl -sSL -o "$temp_file" "$download_url" || error "Download failed"

  [[ ! -s "$temp_file" ]] && error "Downloaded file is empty"

  chmod +x "$temp_file" || error "Failed to make binary executable"

  # Create install directory if it doesn't exist (especially for Windows)
  if [[ ! -d "$INSTALL_DIR" ]]; then
    if [[ "$OS" == "windows" ]]; then
      mkdir -p "$INSTALL_DIR" 2>/dev/null || {
        warning "Could not create $INSTALL_DIR. Installing to current directory instead."
        INSTALL_DIR="."
      }
    else
      if [[ -w "$(dirname "$INSTALL_DIR")" ]]; then
        mkdir -p "$INSTALL_DIR" || error "Failed to create $INSTALL_DIR"
      else
        sudo mkdir -p "$INSTALL_DIR" || error "Failed to create $INSTALL_DIR with sudo"
      fi
    fi
  fi

  # Install to appropriate directory
  local target_path="$INSTALL_DIR/$BINARY_NAME"
  
  if [[ -w "$INSTALL_DIR" ]]; then
    mv "$temp_file" "$target_path" || error "Failed to install to $target_path"
    success "$BINARY_NAME installed to $target_path"
  else
    if [[ "$OS" == "windows" ]]; then
      # On Windows, try alternative locations
      local alt_dirs=("$HOME/bin" "$HOME" ".")
      local installed=false
      
      for alt_dir in "${alt_dirs[@]}"; do
        if [[ -w "$alt_dir" ]]; then
          mkdir -p "$alt_dir" 2>/dev/null
          if mv "$temp_file" "$alt_dir/$BINARY_NAME" 2>/dev/null; then
            success "$BINARY_NAME installed to $alt_dir/$BINARY_NAME"
            warning "Add $alt_dir to your PATH to use bump from anywhere"
            installed=true
            break
          fi
        fi
      done
      
      if [[ "$installed" == "false" ]]; then
        warning "Could not install to system directory. Binary saved as: $temp_file"
        warning "Please manually move it to a directory in your PATH"
      fi
    else
      info "Requesting elevation to install to $INSTALL_DIR"
      sudo mv "$temp_file" "$target_path" || error "Failed to install to $target_path with sudo"
      success "$BINARY_NAME installed to $target_path"
    fi
  fi
}

verify_installation() {
  local version
  
  # Try different ways to find and run the binary
  local binary_paths=("$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME")
  local found=false
  
  for binary_path in "${binary_paths[@]}"; do
    if command -v "$binary_path" >/dev/null 2>&1; then
      version=$("$binary_path" --version 2>/dev/null) || version="unknown"
      success "Installation complete! Version: $version"
      success "Binary location: $(command -v "$binary_path")"
      found=true
      break
    fi
  done
  
  if [[ "$found" == "false" ]]; then
    warning "Binary installed but not found in PATH. You may need to:"
    case "$OS" in
      windows)
        warning "1. Restart your terminal/command prompt"
        warning "2. Add the installation directory to your PATH"
        warning "3. Or navigate to the installation directory to run bump"
        ;;
      *)
        warning "1. Restart your terminal"
        warning "2. Add $INSTALL_DIR to your PATH: export PATH=\"$INSTALL_DIR:\$PATH\""
        warning "3. Or run the full path: $INSTALL_DIR/$BINARY_NAME"
        ;;
    esac
  fi
}

main() {
  echo "🚀 Bump Installer - Cross Platform"
  echo "Supports: Linux (amd64/arm64), macOS (amd64/arm64), Windows (amd64/arm64)"
  echo "Downloading from public GitHub releases - no authentication required"
  echo ""
  
  detect_platform
  check_dependencies
  install_bump
  verify_installation
  
  echo ""
  echo "🎉 Installation process completed!"
  case "$OS" in
    windows)
      echo "💡 Windows users: If bump is not in your PATH, try running it from the installation directory"
      ;;
    *)
      echo "💡 Run 'bump --help' to get started!"
      ;;
  esac
}

main "$@"
