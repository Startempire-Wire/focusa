#!/usr/bin/env bash
# ============================================================================
# Focusa Installer — Bash bootstrapper (Spec 112 §15A.4)
# Source: https://install.focusa.dev/focusa
# Real install logic lives in crates/focusa-cli/src/commands/install.rs
# (`focusa install --target=auto`).
#
# Full-release-only policy:
#   The installer queries the GitHub releases list and refuses to download
#   from any tag that does not ship ALL three binaries (focusa,
#   focusa-daemon, focusa-tui) for the detected target triple. Partial
#   releases are skipped automatically; the latest complete release wins.
#
# No version strings are hardcoded here — the script discovers the latest
# complete release for the channel at install time.
# ============================================================================
set -euo pipefail

CHANNEL="${CHANNEL:-stable}"; DRY_RUN="${DRY_RUN:-0}"; EVAL="${EVAL:-0}"
LICENSE_KEY="${FOCUSA_LICENSE_KEY:-${LICENSE_KEY:-}}"
LICENSE_KEY="${LICENSE_KEY:-${WPUIAI_LICENSE_KEY:-}}"
TARGET="auto"; GITHUB_REPO="Startempire-Wire/focusa"
LICENSE_REGISTRY="${LICENSE_REGISTRY:-https://wpuiai.com}"
REQUIRED_ASSETS=(focusa focusa-daemon focusa-tui)
MAX_CANDIDATES="${MAX_CANDIDATES:-20}"   # scan the most-recent N releases
for arg in "$@"; do
  case "$arg" in
    --dry-run)          DRY_RUN=1 ;;
    --eval)             EVAL=1 ;;
    --target=*)         TARGET="${arg#--target=}" ;;
    --channel=*)        CHANNEL="${arg#--channel=}" ;;
    --license-key=*)    LICENSE_KEY="${arg#--license-key=}" ;;
    --github-repo=*)    GITHUB_REPO="${arg#--github-repo=}" ;;
    --max-candidates=*) MAX_CANDIDATES="${arg#--max-candidates=}" ;;
    --help|-h)          sed -n '2,18p' "$0"; exit 0 ;;
    *) echo "unknown arg: $arg" >&2; exit 64 ;;
  esac
done

command -v curl >/dev/null || { echo "curl is required" >&2; exit 1; }
command -v python3 >/dev/null || { echo "python3 is required" >&2; exit 1; }
command -v sha256sum >/dev/null || command -v shasum >/dev/null || { echo "sha256sum required" >&2; exit 1; }

HOST_OS=$(uname -s); HOST_ARCH=$(uname -m)
[ "$HOST_OS" = "Linux" ] || [ "$HOST_OS" = "Darwin" ] \
  || { echo "unsupported OS: $HOST_OS (use install-focusa.ps1 on Windows)" >&2; exit 1; }
case "$HOST_ARCH" in x86_64|aarch64) ;; *) echo "unsupported arch: $HOST_ARCH" >&2; exit 1;; esac

case "$HOST_OS-$HOST_ARCH" in
  Linux-x86_64)  TRIPLE="x86_64-unknown-linux-gnu" ;;
  Linux-aarch64) TRIPLE="aarch64-unknown-linux-gnu" ;;
  Darwin-x86_64) TRIPLE="x86_64-apple-darwin" ;;
  Darwin-arm64|Darwin-aarch64) TRIPLE="aarch64-apple-darwin" ;;
esac

# ---------------------------------------------------------------------------
# Channel → release-tag pattern. The channel picks which tag prefix to look
# for in the release list; the actual selected tag is the latest FULL release
# (all required assets present for our triple) that matches the prefix.
# ---------------------------------------------------------------------------
case "$CHANNEL" in
  stable)
    PATTERN='v'            # match any tag
    ;;
  preview)
    PATTERN='v*-preview'
    ;;
  nightly)
    PATTERN='v*-nightly'
    ;;
  *)
    echo "unknown channel: $CHANNEL" >&2
    exit 1
    ;;
esac

# ---------------------------------------------------------------------------
# Fetch the most-recent N releases (GH API paginates 30 at a time; per_page=30
# + the API returns them newest-first). Iterate, prefer latest matching the
# channel pattern AND shipping all required assets for our triple.
# ---------------------------------------------------------------------------
RELEASES=$(curl -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases?per_page=30" 2>/dev/null) \
  || { echo "[focusa-install] release list fetch failed" >&2; exit 1; }

# Returns tab-separated: <tag>\t<focusa-url> when a release is complete.
# Empty stdout when no complete release matches the channel pattern within
# the first MAX_CANDIDATES.
pick_complete_release() {
  python3 - "$RELEASES" "$PATTERN" "$TRIPLE" "$MAX_CANDIDATES" <<'PY'
import json, re, sys
releases, pattern, triple, max_n = sys.argv[1:]
data = json.loads(releases)
pat_re = re.compile("^" + pattern.replace("*", ".*") + "$")
seen = 0
for rel in data:
    if not rel.get("tag_name"):
        continue
    if not pat_re.match(rel["tag_name"]):
        continue
    seen += 1
    if seen > int(max_n):
        break
    asset_names = {a.get("name", "") for a in rel.get("assets", [])}
    required = [
        f"focusa-{rel['tag_name']}-{triple}",
        f"focusa-daemon-{rel['tag_name']}-{triple}",
        f"focusa-tui-{rel['tag_name']}-{triple}",
    ]
    if all(name in asset_names for name in required):
        focusa_url = next(
            (a["browser_download_url"] for a in rel.get("assets", [])
             if a.get("name") == required[0]),
            "",
        )
        print(f"{rel['tag_name']}\t{focusa_url}")
        break
PY
}

SELECTED=$(pick_complete_release)
if [ -z "$SELECTED" ]; then
  echo "[focusa-install] no complete release for channel='${CHANNEL}' triple='${TRIPLE}' within first ${MAX_CANDIDATES} releases" >&2
  echo "[focusa-install] recovery_hint: a release is complete only when it ships focusa, focusa-daemon, AND focusa-tui for the target triple." >&2
  echo "[focusa-install] recovery_hint: check https://github.com/${GITHUB_REPO}/releases for the latest full release." >&2
  exit 1
fi

RELEASE_TAG=$(printf '%s' "$SELECTED" | cut -f1)
ASSET_URL=$(printf '%s' "$SELECTED" | cut -f2-)
[ -n "$RELEASE_TAG" ] && [ -n "$ASSET_URL" ] \
  || { echo "[focusa-install] could not parse selected release" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Download focusa, verify SHA256SUMS if available.
# ---------------------------------------------------------------------------
BIN_DIR="${HOME}/.focusa/bin"
mkdir -p "$BIN_DIR"
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
curl -fsSL "$ASSET_URL" -o "$TMP/focusa" || { echo "[focusa-install] download failed" >&2; exit 1; }
chmod +x "$TMP/focusa"

ASSET_FOCUSA="focusa-${RELEASE_TAG}-${TRIPLE}"
SHA=""
for sha_path in "SHA256SUMS" "SHA256SUMS.txt"; do
  RAW=$(curl -fsSL "https://github.com/${GITHUB_REPO}/releases/download/${RELEASE_TAG}/${sha_path}" 2>/dev/null || true)
  if [ -n "$RAW" ]; then
    SHA=$(printf '%s' "$RAW" | awk -v n="${ASSET_FOCUSA}" '$2 == n {print $1; exit}')
    [ -n "$SHA" ] && break
  fi
done
if [ -n "$SHA" ]; then
  ACT=$(sha256sum "$TMP/focusa" | awk '{print $1}')
  [ "$ACT" = "$SHA" ] || { echo "[focusa-install] checksum mismatch" >&2; exit 1; }
else
  echo "[focusa-install] warning: SHA256SUMS not available for ${RELEASE_TAG}; skipping verify" >&2
fi

mv "$TMP/focusa" "$BIN_DIR/focusa"

ARGS=(install --target="$TARGET" --version="$RELEASE_TAG" --github-repo="$GITHUB_REPO")
[ "$DRY_RUN" = 1 ] && ARGS+=(--dry-run)
[ "$EVAL" = 1 ] && ARGS+=(--eval)
if [ -n "$LICENSE_KEY" ]; then
  ARGS+=(--license-key="$LICENSE_KEY")
elif [ "$EVAL" != 1 ]; then
  # Default to eval when no license key provided AND --eval was not set,
  # so first-time users get a working install while the operator can
  # promote to a paid key later via `focusa license activate`.
  ARGS+=(--eval)
  echo "[focusa-install] no license key provided; defaulting to --eval mode (install will succeed; activate license later with 'focusa license activate <key>')." >&2
fi
[ "$CHANNEL" != "stable" ] && ARGS+=(--channel="$CHANNEL")

exec "$BIN_DIR/focusa" "${ARGS[@]}"