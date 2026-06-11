#!/usr/bin/env bash
#
# Bump installer — Linux / macOS / WSL. Always installs (or updates) the latest release.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/launchfirestorm/bump/main/install/get_bump.sh | bash
#
set -euo pipefail

REPO="launchfirestorm/bump"

# Auth token (optional, avoids rate limits)
TOKEN="${GITHUB_TOKEN:-${GH_TOKEN:-}}"
auth_header=()
[[ -n "$TOKEN" ]] && auth_header=(-H "Authorization: Bearer ${TOKEN}")

info()    { echo -e "\033[0;34m[INFO]\033[0m $1"; }
success() { echo -e "\033[0;32m[SUCCESS]\033[0m $1"; }
die()     { echo -e "\033[0;31m[ERROR]\033[0m $1"; exit 1; }

# Detect platform
case "$(uname -m)-$(uname -s | cut -d- -f1)" in
  x86_64-Linux)  os=linux;  arch=amd64 ;;
  aarch64-Linux) os=linux;  arch=arm64 ;;
  x86_64-Darwin) os=macos;  arch=amd64 ;;
  arm64-Darwin)  os=macos;  arch=arm64 ;;
  *) die "Unsupported platform: $(uname -m)-$(uname -s). On Windows use install/get_bump.ps1." ;;
esac
info "Platform: ${os}/${arch}"

# Resolve install target — update in place if bump already exists
if command -v bump >/dev/null 2>&1; then
  target="$(command -v bump)"
  info "Updating existing bump at ${target} ($(bump --version 2>/dev/null || echo '?'))"
else
  target="/usr/local/bin/bump"
fi
install_dir="$(dirname "$target")"

# Latest release tag
tag=$(curl -fsSL "${auth_header[@]}" -H "Accept: application/vnd.github+json" -H "User-Agent: bump-install" \
  "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)
[[ -n "$tag" ]] || die "Failed to resolve latest release"
info "Latest release: ${tag}"

# Download
url="https://github.com/${REPO}/releases/download/${tag}/bump-${os}-${arch}"
info "Downloading: ${url}"
tmp=$(mktemp)
curl -fsSL "${auth_header[@]}" -o "$tmp" "$url" || die "Download failed"
[[ -s "$tmp" ]] || die "Downloaded file is empty"
chmod +x "$tmp"

if [[ -w "$install_dir" ]]; then # if writable, install directly
  [[ -d "$install_dir" ]] || mkdir -p "$install_dir"
  mv "$tmp" "$target"
elif command -v sudo >/dev/null 2>&1; then
  [[ -d "$install_dir" ]] || sudo mkdir -p "$install_dir"
  sudo mv "$tmp" "$target"
else
  die "Cannot write to ${install_dir} — not root and sudo is not available"
fi

success "bump installed to ${target}"
