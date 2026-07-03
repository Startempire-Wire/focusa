#!/usr/bin/env bash
# ============================================================================
# Focusa Installer — Bash bootstrapper (Spec 112 §15A.4)
# Source: https://install.focusa.dev/focusa
# This is a thin bootstrapper. Real install logic lives in
# crates/focusa-cli/src/commands/install.rs (\`focusa install --target=auto\`).
# ============================================================================
set -euo pipefail

CHANNEL="${CHANNEL:-stable}"; DRY_RUN="${DRY_RUN:-0}"; EVAL="${EVAL:-0}"
LICENSE_KEY=""; TARGET="auto"; GITHUB_REPO="Startempire-Wire/focusa"
for arg in "$@"; do
  case "$arg" in
    --dry-run)          DRY_RUN=1 ;;
    --eval)             EVAL=1 ;;
    --target=*)         TARGET="${arg#--target=}" ;;
    --channel=*)        CHANNEL="${arg#--channel=}" ;;
    --license-key=*)    LICENSE_KEY="${arg#--license-key=}" ;;
    --github-repo=*)    GITHUB_REPO="${arg#--github-repo=}" ;;
    --help|-h)          sed -n '2,12p' "$0"; exit 0 ;;
    *) echo "unknown arg: $arg" >&2; exit 64 ;;
  esac
done

command -v curl >/dev/null || { echo "curl is required" >&2; exit 1; }
command -v sha256sum >/dev/null || command -v shasum >/dev/null || { echo "sha256sum required" >&2; exit 1; }

HOST_OS=$(uname -s); HOST_ARCH=$(uname -m)
[ "$HOST_OS" = "Linux" ] || [ "$HOST_OS" = "Darwin" ] \
  || { echo "unsupported OS: $HOST_OS (use install-focusa.ps1 on Windows)" >&2; exit 1; }
case "$HOST_ARCH" in x86_64|aarch64) ;; *) echo "unsupported arch: $HOST_ARCH" >&2; exit 1;; esac

case "$CHANNEL" in
  stable)  TAG="v0.9.54-dev" ;;
  preview) TAG="v0.9.55-dev-preview" ;;
  nightly) TAG="v0.9.55-dev-nightly" ;;
  *) echo "unknown channel: $CHANNEL" >&2; exit 1 ;;
esac

# Resolve asset URL via GH release API
ASSET="focusa-${TAG}-${HOST_OS}-${HOST_ARCH}"
MANIFEST=$(curl -sSL "https://api.github.com/repos/${GITHUB_REPO}/releases/tags/${TAG}") \
  || { echo "release fetch failed" >&2; exit 1; }
ASSET_URL=$(printf '%s' "$MANIFEST" \
  | python3 -c "import json,sys; r=json.load(sys.stdin); print(next((a['browser_download_url'] for a in r.get('assets',[]) if a['name'].startswith('${ASSET}')), ''))" \
  2>/dev/null) \
  || ASSET_URL=""
[ -n "$ASSET_URL" ] || { echo "asset ${ASSET} not in ${TAG}" >&2; exit 1; }

BIN_DIR="${HOME}/.focusa/bin"
mkdir -p "$BIN_DIR"
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
curl -sSL "$ASSET_URL" -o "$TMP/focusa" || { echo "download failed" >&2; exit 1; }
chmod +x "$TMP/focusa"

# SHA256SUMS verify (best-effort)
SHA=$(curl -sSL "https://github.com/${GITHUB_REPO}/releases/download/${TAG}/SHA256SUMS.txt" 2>/dev/null \
  | awk -v n="${ASSET}" '$2 == n {print $1}')
if [ -n "$SHA" ]; then
  ACT=$(sha256sum "$TMP/focusa" | awk '{print $1}')
  [ "$ACT" = "$SHA" ] || { echo "checksum mismatch" >&2; exit 1; }
fi

mv "$TMP/focusa" "$BIN_DIR/focusa"

ARGS=(install --target="$TARGET")
[ "$DRY_RUN" = 1 ] && ARGS+=(--dry-run)
[ "$EVAL" = 1 ] && ARGS+=(--eval)
[ -n "$LICENSE_KEY" ] && ARGS+=(--license-key="$LICENSE_KEY")
[ "$CHANNEL" != "stable" ] && ARGS+=(--channel="$CHANNEL")
ARGS+=(--github-repo="$GITHUB_REPO")

exec "$BIN_DIR/focusa" "${ARGS[@]}"
