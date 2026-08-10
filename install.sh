#!/bin/sh
# Adapted from upstream's website/install.sh.
#
# Upstream resolves binaries through https://herdr.dev/latest.json, which tracks
# upstream releases. This fork publishes its own assets, so the URL and checksum
# come from the fork's GitHub releases instead. Everything else is unchanged.
set -eu

BIN="herdr"
REPO="hexxt-git/herdr"
RELEASE_URL="https://github.com/${REPO}/releases/latest/download"
INSTALL_DIR="${HERDR_INSTALL_DIR:-$HOME/.local/bin}"

main() {
    echo ""
    echo "      ,ww"
    echo "     wWWWWWWW_)  herdr installer"
    echo "     \`WWWWWW'    github.com/${REPO}"
    echo "      II  II"
    echo ""

    # detect platform
    OS="$(uname -s)"
    case "$OS" in
        Linux)  os="linux" ;;
        Darwin) os="macos" ;;
        *)      err "unsupported OS: $OS" ;;
    esac

    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        *)              err "unsupported architecture: $ARCH" ;;
    esac

    log "detected ${os}/${arch}"

    # check dependencies
    need curl
    need awk

    ASSET="${BIN}-${os}-${arch}"
    URL="${RELEASE_URL}/${ASSET}"

    # the release publishes a SHA256SUMS asset alongside the binaries
    log "fetching latest release checksums..."
    SUMS="$(curl -fsSL --retry 3 --connect-timeout 10 --max-time 20 "${RELEASE_URL}/SHA256SUMS")" \
        || err "can't reach ${RELEASE_URL}/SHA256SUMS. Please try again later; GitHub might be down. Who let the sheeps out? baaa."
    SHA256="$(printf '%s\n' "$SUMS" | awk -v asset="$ASSET" '$2 == asset { print $1; exit }')"
    VERSION="$(curl -fsSLI -o /dev/null -w '%{url_effective}' \
        "https://github.com/${REPO}/releases/latest" 2>/dev/null | awk -F'/tag/' '{ print $2 }')" || VERSION=""

    if [ "${#SHA256}" -ne 64 ]; then
        err "release checksums do not include a valid SHA-256 for ${ASSET}"
    fi
    if ! printf '%s\n' "$SHA256" | awk '/[^0-9A-Fa-f]/ { exit 1 }'; then
        err "release checksums do not include a valid SHA-256 for ${ASSET}"
    fi
    SHA256="$(printf '%s\n' "$SHA256" | awk '{ print tolower($0) }')"

    if command -v sha256sum >/dev/null 2>&1; then
        SHA256_TOOL="sha256sum"
    elif command -v shasum >/dev/null 2>&1; then
        SHA256_TOOL="shasum"
    elif command -v openssl >/dev/null 2>&1; then
        SHA256_TOOL="openssl"
    else
        err "SHA-256 verification requires sha256sum, shasum, or openssl"
    fi

    if [ -n "$VERSION" ]; then
        log "downloading ${VERSION}..."
    else
        log "downloading latest release..."
    fi
    TMP="$(mktemp -d)"
    trap 'rm -rf "$TMP"' EXIT

    if ! curl -fsSL --retry 3 --connect-timeout 10 --max-time 120 "$URL" -o "${TMP}/${BIN}"; then
        err "download failed from ${URL}"
    fi

    case "$SHA256_TOOL" in
        sha256sum) ACTUAL_SHA256="$(sha256sum < "${TMP}/${BIN}" | awk '{ print $1 }')" ;;
        shasum)    ACTUAL_SHA256="$(shasum -a 256 < "${TMP}/${BIN}" | awk '{ print $1 }')" ;;
        openssl)   ACTUAL_SHA256="$(openssl dgst -sha256 < "${TMP}/${BIN}" | awk '{ print $NF }')" ;;
    esac
    if [ "$ACTUAL_SHA256" != "$SHA256" ]; then
        err "downloaded Herdr checksum did not match"
    fi

    # install
    mkdir -p "$INSTALL_DIR"
    mv "${TMP}/${BIN}" "${INSTALL_DIR}/${BIN}"
    chmod +x "${INSTALL_DIR}/${BIN}"

    log "installed ${BIN} to ${INSTALL_DIR}/${BIN}"

    # check PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo ""
            warn "${INSTALL_DIR} is not in your PATH"
            echo "  add it to your shell config:"
            echo ""
            echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            ;;
    esac

    # verify
    if command -v "$BIN" >/dev/null 2>&1; then
        echo ""
        log "ready. run 'herdr' to get started."
    fi

    echo ""
}

log()  { printf '  \033[32m>\033[0m %s\n' "$1"; }
warn() { printf '  \033[33m!\033[0m %s\n' "$1"; }
err()  { printf '  \033[31m✗\033[0m %s\n' "$1" >&2; exit 1; }

need() {
    if ! command -v "$1" >/dev/null 2>&1; then
        err "requires '$1' — install it first, or download a binary manually from https://github.com/${REPO}/releases/latest"
    fi
}

main "$@"
