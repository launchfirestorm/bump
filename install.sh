#!/bin/bash
set -e

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
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

check_github_token() {
    [[ -z "$GH_TOKEN" ]] && error "GH_TOKEN required. Set: export GH_TOKEN=your_token"
    info "GitHub private access token found"
}

github_api_call() {
    curl -sSL -H "Authorization: token $GH_TOKEN" "$1"
}

check_os() {
    [[ "$OSTYPE" != "linux-gnu"* ]] && error "Linux only. Current: $OSTYPE"
    info "Linux detected"
}

detect_arch() {
    case $(uname -m) in
        x86_64) ARCH="amd64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        *) error "Unsupported arch: $(uname -m)" ;;
    esac
    info "Architecture: $ARCH"
}

check_dependencies() {
    for cmd in curl sudo; do
        command -v "$cmd" >/dev/null || error "Missing: $cmd"
    done
}

install_bump() {
    local asset_name="bump-linux-musl-${ARCH}"
    local temp_file=$(mktemp)
    
    info "Downloading bump..."
    
    local release_data=$(github_api_call "https://api.github.com/repos/${REPO}/releases/latest")
    local asset_id
    
    if command -v jq >/dev/null 2>&1; then
        asset_id=$(echo "$release_data" | jq -r ".assets[] | select(.name==\"$asset_name\") | .id")
    else
        asset_id=$(echo "$release_data" | grep -A 2 "\"name\": \"$asset_name\"" | grep '"id":' | sed -E 's/.*"id": ([0-9]+).*/\1/')
    fi
    
    [[ -z "$asset_id" || "$asset_id" == "null" ]] && error "Asset $asset_name not found"
    
    curl -sSL -H "Authorization: token $GH_TOKEN" -H "Accept: application/octet-stream" -o "$temp_file" "https://api.github.com/repos/${REPO}/releases/assets/$asset_id" || error "Download failed"
    
    [[ ! -s "$temp_file" ]] && error "Empty download"
    
    chmod +x "$temp_file"
    
    if [[ -w "/usr/local/bin" ]]; then
        mv "$temp_file" "/usr/local/bin/$BINARY_NAME" || error "Install failed"
    else
        info "Need elevation to write to /usr/local/bin"
        sudo mv "$temp_file" "/usr/local/bin/$BINARY_NAME" || error "Install failed"
    fi
    
    success "bump installed to /usr/local/bin/$BINARY_NAME"
}

verify_installation() {
    local version=$(bump --version 2>/dev/null || echo "unknown")
    success "Installation complete! Version: $version"
}

main() {
    echo "🚀 Bump Installer"
    check_github_token
    check_os
    detect_arch
    check_dependencies
    install_bump
    verify_installation
}

main "$@"
