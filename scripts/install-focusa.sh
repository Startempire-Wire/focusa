#!/usr/bin/env bash
# Focusa verified bootstrapper (Spec 112 §15A; Specs 150A/152, 152E). Product installation and entitlement authority
# belong to the canonical Rust installer; this script only selects and verifies
# that installer, then delegates. Activation intent (e.g. --eval) is forwarded
# to the shared activation client and is never evaluated or issued locally.
set -euo pipefail

# Safe version surface: `--version` prints the installer version and exits 0.
# Never derived from remote/admin state.
FOCUSA_INSTALLER_VERSION="0.9.166"

GITHUB_REPO="${FOCUSA_GITHUB_REPO:-Startempire-Wire/focusa}"
RELEASE_BASE_URL="${FOCUSA_RELEASE_BASE_URL:-}"
RELEASE_TAG="${FOCUSA_RELEASE_TAG:-}"
TARGET_INPUT="auto"
CHANNEL="stable"
DRY_RUN=0
EVAL=0
ACCEPT_LICENSE=0
INSTALL_DEPS=1
ASSUME_YES=0
NO_SERVICE=0
UNINSTALL=0
PURGE_DATA=0

usage() {
  cat <<'USAGE'
Usage: install-focusa.sh [options]

  --dry-run                show the delegation plan without writes or downloads
  --eval                   forward Evaluation intent to the shared activation
                           client; maps to verified-email limited activation
                           (Spec 172 limited-access overlay; authority-issued
                           only, never local)
  --target=TARGET          auto|linux|darwin|windows-x64|windows-arm64
  --channel=CHANNEL        stable|preview|nightly
  --github-repo=OWNER/REPO override release repository
  --release-base-url=URL   override release asset base URL
  --release-tag=TAG        select an exact immutable release
  --accept-license         forward BSL acceptance to the Rust installer
  --install-dependencies   allow Rust installer dependency onboarding
  --no-install-dependencies
  --assume-yes             approve dependency installation
  --no-service             skip service registration
  --uninstall              delegate preserve-by-default uninstall
  --purge-data             purge only with --uninstall and separate confirmation
  -h, --help               show this help

Raw license keys and email addresses are intentionally not accepted. The Rust
# License purchases and reissues happen at the public storefront; the installer
# never touches admin surfaces: https://install.focusa.dev/license (purchase) and
# https://focusa.dev/support (reissue/support).

installer resolves or acquires a signed, node-bound authority lease and safely
presents the device verification URL and user-code handle. Evaluation is
authority-issued only; the bootstrapper never creates local evaluation state.
Spec 172 verified-email limited activation replaces local/self-issued grants:
--eval requests the authority-signed limited-access overlay and no channel can
issue or persist an Evaluation locally.
USAGE
}

log() { printf '\033[1;34m[focusa-install]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[focusa-install]\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31m[focusa-install]\033[0m %s\n' "$*" >&2; exit 1; }
usage_die() { printf '\033[1;31m[focusa-install]\033[0m %s\n' "$*" >&2; exit 64; }
have() { command -v "$1" >/dev/null 2>&1; }

case "${1:-}" in
  --version|-v)
    printf 'focusa-installer %s\n' "$FOCUSA_INSTALLER_VERSION"
    exit 0
    ;;
esac

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --eval) EVAL=1 ;;
    --target=*) TARGET_INPUT="${arg#*=}" ;;
    --channel=*) CHANNEL="${arg#*=}" ;;
    --github-repo=*) GITHUB_REPO="${arg#*=}" ;;
    --release-base-url=*) RELEASE_BASE_URL="${arg#*=}" ;;
    --release-tag=*) RELEASE_TAG="${arg#*=}" ;;
    --accept-license) ACCEPT_LICENSE=1 ;;
    --install-dependencies) INSTALL_DEPS=1 ;;
    --no-install-dependencies) INSTALL_DEPS=0 ;;
    --assume-yes) ASSUME_YES=1 ;;
    --no-service) NO_SERVICE=1 ;;
    --uninstall) UNINSTALL=1 ;;
    --purge-data) PURGE_DATA=1 ;;
    --license-key=*|--email=*|--registry=*)
      die "E_AUTHORITY_RAW_KEY_FORBIDDEN: raw credentials and legacy registry overrides are forbidden; use signed authority device authorization"
      ;;
    -h|--help) usage; exit 0 ;;
    *) usage_die "unknown option" ;;
  esac
done

case "$CHANNEL" in
  stable|preview|nightly) ;;
  *) die "unsupported channel: $CHANNEL" ;;
esac
[ "$PURGE_DATA" = 0 ] || [ "$UNINSTALL" = 1 ] || usage_die "--purge-data requires --uninstall"

if [ "$UNINSTALL" = 1 ]; then
  # Prefer the canonical install location so delegated uninstall runs the
  # preserved binary (Spec 132 public uninstall preservation test).
  focusa_bin="$HOME/.focusa/bin/focusa"
  if [ ! -x "$focusa_bin" ]; then
    have focusa || die "focusa is not installed; recovery: reinstall or invoke the preserved binary directly"
    focusa_bin="$(command -v focusa)"
  fi
  uninstall_args=(uninstall --yes)
  if [ "$PURGE_DATA" = 0 ]; then
    # Preserve by default: --keep-data is the safe posture (Spec 132).
    uninstall_args+=(--keep-data)
  fi
  exec "$focusa_bin" "${uninstall_args[@]}"
fi

OS="$(uname -s 2>/dev/null || printf unknown)"
ARCH="$(uname -m 2>/dev/null || printf unknown)"
case "$TARGET_INPUT" in
  auto)
    case "$OS:$ARCH" in
      Linux:x86_64|Linux:amd64) TRIPLE="x86_64-unknown-linux-musl"; RUST_TARGET="linux" ;;
      Linux:aarch64|Linux:arm64) TRIPLE="aarch64-unknown-linux-gnu"; RUST_TARGET="linux" ;;
      Darwin:x86_64|Darwin:amd64) TRIPLE="x86_64-apple-darwin"; RUST_TARGET="darwin" ;;
      Darwin:arm64|Darwin:aarch64) TRIPLE="aarch64-apple-darwin"; RUST_TARGET="darwin" ;;
      *) die "unsupported bootstrap host: $OS/$ARCH" ;;
    esac
    ;;
  linux)
    RUST_TARGET="linux"
    case "$ARCH" in
      x86_64|amd64) TRIPLE="x86_64-unknown-linux-musl" ;;
      aarch64|arm64) TRIPLE="aarch64-unknown-linux-gnu" ;;
      *) die "unsupported Linux architecture: $ARCH" ;;
    esac
    ;;
  darwin)
    RUST_TARGET="darwin"
    case "$ARCH" in
      x86_64|amd64) TRIPLE="x86_64-apple-darwin" ;;
      aarch64|arm64) TRIPLE="aarch64-apple-darwin" ;;
      *) die "unsupported macOS architecture: $ARCH" ;;
    esac
    ;;
  windows-x64) TRIPLE="x86_64-pc-windows-msvc"; RUST_TARGET="windows-x64" ;;
  windows-arm64) TRIPLE="aarch64-pc-windows-msvc"; RUST_TARGET="windows-arm64" ;;
  *) die "unsupported target: $TARGET_INPUT" ;;
esac
TARGET="$TRIPLE"

if [ "$DRY_RUN" = 1 ]; then
  printf 'Focusa verified bootstrap plan\n'
  printf '  target: %s (%s)\n' "$RUST_TARGET" "$TARGET"
  printf '  channel: %s\n' "$CHANNEL"
  printf '  release: %s\n' "${RELEASE_TAG:-latest-complete}"
  printf '  entitlement: signed authority lease; device authorization if absent\n'
  if [ "$EVAL" = 1 ]; then
    printf '  evaluation: authority-issued only; --eval maps to verified-email limited activation (Spec 172)\n'
  fi
  printf '  mutations: none\n'
  exit 0
fi

have curl || die "curl is required to download the verified Rust bootstrap binary"
have sha256sum || have shasum || die "sha256sum or shasum is required"

CURL_RETRY_ALL_ERRORS=""
if curl --help all 2>/dev/null | grep -q -- '--retry-all-errors'; then
  CURL_RETRY_ALL_ERRORS="--retry-all-errors"
fi
curl_resilient() {
  # shellcheck disable=SC2086
  curl --http1.1 --retry 5 $CURL_RETRY_ALL_ERRORS --retry-delay 2 --connect-timeout 20 "$@"
}

if [ -z "$RELEASE_TAG" ]; then
  [ "$CHANNEL" = stable ] || die "$CHANNEL installs require an explicit immutable --release-tag"
  RELEASE_TAG="$(curl_resilient -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" \
    | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
  [ -n "$RELEASE_TAG" ] || die "could not resolve latest immutable release tag"
fi
case "$CHANNEL" in
  stable) TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+$' ;;
  preview) TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+-(dev|rc)(\..*)?$' ;;
  nightly) TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+-nightly(\..*)?$' ;;
esac
printf '%s\n' "$RELEASE_TAG" | grep -Eq "$TAG_PATTERN" \
  || die "release tag $RELEASE_TAG is not valid for channel $CHANNEL"

release_asset_url() {
  local name="$1"
  if [ -n "$RELEASE_BASE_URL" ]; then
    printf '%s/%s' "${RELEASE_BASE_URL%/}" "$name"
  else
    printf 'https://github.com/%s/releases/download/%s/%s' "$GITHUB_REPO" "$RELEASE_TAG" "$name"
  fi
}

TMP="$(mktemp -d "${TMPDIR:-/tmp}/focusa-bootstrap.XXXXXX")"
cleanup() { rm -rf -- "$TMP"; }
trap cleanup EXIT INT TERM

# Canonical release binaries are immutable-tag qualified. Keep the bootstrap
# lookup identical to release packaging and the signed checksum manifest.
ASSET="focusa-${RELEASE_TAG}-${TRIPLE}"
[ "$RUST_TARGET" != "windows-x64" ] && [ "$RUST_TARGET" != "windows-arm64" ] || ASSET="${ASSET}.exe"
BOOTSTRAP_BIN="$TMP/$ASSET"
CHECKSUM_MANIFEST="$TMP/SHA256SUMS.txt"

log "Downloading $ASSET from $RELEASE_TAG"
curl_resilient -fsSL "$(release_asset_url "$ASSET")" -o "$BOOTSTRAP_BIN" \
  || die "bootstrap download failed; recovery: retry or set --release-base-url to an approved mirror"
curl_resilient -fsSL "$(release_asset_url SHA256SUMS.txt)" -o "$CHECKSUM_MANIFEST" \
  || die "signed release checksum manifest is unavailable"

EXPECTED="$(awk -v asset="$ASSET" '$2 == asset || $2 == "*" asset {print $1; exit}' "$CHECKSUM_MANIFEST")"
[ -n "$EXPECTED" ] || die "checksum manifest does not list $ASSET"
if have sha256sum; then
  ACTUAL="$(sha256sum "$BOOTSTRAP_BIN" | awk '{print $1}')"
else
  ACTUAL="$(shasum -a 256 "$BOOTSTRAP_BIN" | awk '{print $1}')"
fi
[ "$EXPECTED" = "$ACTUAL" ] || die "checksum mismatch for $ASSET"

verify_cosign_manifest() {
  have cosign || return 1
  curl_resilient -fsSL "$(release_asset_url SHA256SUMS.txt.cosign.sig)" -o "$TMP/SHA256SUMS.txt.cosign.sig" || return 1
  curl_resilient -fsSL "$(release_asset_url SHA256SUMS.txt.cosign.pem)" -o "$TMP/SHA256SUMS.txt.cosign.pem" || return 1
  cosign verify-blob --certificate "$TMP/SHA256SUMS.txt.cosign.pem" \
    --signature "$TMP/SHA256SUMS.txt.cosign.sig" "$CHECKSUM_MANIFEST" >/dev/null
}
if verify_cosign_manifest; then
  log "cosign verification succeeded"
elif [ "$CHANNEL" = stable ]; then
  die "stable install requires valid Cosign signature metadata; SHA256 alone is insufficient"
else
  warn "install is preview-only because Cosign verification is unavailable; checksum verification succeeded"
fi
chmod 0755 "$BOOTSTRAP_BIN"

# Presenter-safe handoff: allowlisted flags only; product/price/grant/feature
# and Evaluation decisions stay inside the shared activation client.
ARGS=(install --target="$RUST_TARGET" --channel="$CHANNEL" --github-repo="$GITHUB_REPO")
[ "$EVAL" = 0 ] || ARGS+=(--eval)
[ "$ACCEPT_LICENSE" = 0 ] || ARGS+=(--accept-license)
[ "$INSTALL_DEPS" = 0 ] && ARGS+=(--no-install-dependencies) || ARGS+=(--install-dependencies)
[ "$ASSUME_YES" = 0 ] || ARGS+=(--assume-yes)
[ "$NO_SERVICE" = 0 ] || ARGS+=(--no-service)

BOOTSTRAP_STASH="$TMP/bootstrap-stash"
mkdir -p "$BOOTSTRAP_STASH"
restore_bootstrap_stash() {
  warn "E_INSTALL_INTERRUPTED: Rust installer failed; preserving recovery state and leaving prior installation authoritative"
}

# The Rust installer performs entitlement acquisition before product asset download,
# atomically activates assets, verifies daemon lease parity, and emits recovery hints.
export FOCUSA_RELEASE_TAG="$RELEASE_TAG"
export FOCUSA_RELEASE_BASE_URL="$RELEASE_BASE_URL"
if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then
  rm -rf "$BOOTSTRAP_STASH"
  log "Focusa installation completed through the canonical Rust flow"
else
  status=$?
  restore_bootstrap_stash
  exit "$status"
fi
