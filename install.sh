#!/bin/sh
# Install the latest herdr build from this fork's GitHub releases.
#
# Upstream's website/install.sh resolves binaries through herdr.dev/latest.json,
# which tracks upstream releases. This fork publishes its own release assets, so
# it pulls them straight from GitHub instead.
set -eu

REPO="hexxt-git/herdr"
INSTALL_DIR="${HERDR_INSTALL_DIR:-$HOME/.local/bin}"

err() { echo "error: $*" >&2; exit 1; }

case "$(uname -s)" in
    Linux)  os=linux ;;
    Darwin) os=macos ;;
    *) err "unsupported OS $(uname -s). On Windows, download the zip from https://github.com/$REPO/releases/latest" ;;
esac

case "$(uname -m)" in
    x86_64|amd64) arch=x86_64 ;;
    arm64|aarch64) arch=aarch64 ;;
    *) err "unsupported architecture $(uname -m)" ;;
esac

command -v curl >/dev/null 2>&1 || err "curl is required"

asset="herdr-${os}-${arch}"
url="https://github.com/$REPO/releases/latest/download/$asset"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

echo "downloading $asset ..."
curl -fsSL "$url" -o "$tmp" || err "download failed: $url"

mkdir -p "$INSTALL_DIR"
chmod 755 "$tmp"
mv "$tmp" "$INSTALL_DIR/herdr"
trap - EXIT

echo "installed herdr to $INSTALL_DIR/herdr"
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "note: $INSTALL_DIR is not on your PATH; add it to your shell profile." ;;
esac
