#!/usr/bin/env bash
# Compatibility adapter for the canonical Rust installer.
#
# This script intentionally performs no binary promotion, process signalling,
# systemd mutation, health retry, or rollback. Those lifecycle decisions belong
# to `focusa install --system-install`, which installs one verified release set
# and settles the system unit, process identity, health, and rollback together.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=scripts/lib/release-version.sh
source "$ROOT_DIR/scripts/lib/release-version.sh"

INSTALL_ROOT="${FOCUSA_INSTALL_ROOT:-/usr/local}"
BIN_NAME="${FOCUSA_BIN_NAME:-focusa-daemon}"
SERVICE_NAME="${FOCUSA_SERVICE_NAME:-focusa-daemon}"
STATE_DIR="${FOCUSA_STATE_DIR:-${INSTALL_ROOT}/lib/focusa}"
HEALTH_URL="${FOCUSA_DAEMON_URL:-http://127.0.0.1:8787/v1/health}"
EXPECTED_VERSION="${FOCUSA_EXPECTED_VERSION:-}"
BINARY=""
NO_RESTART=0
NO_VERIFY=0
REQUIRE_SERVICE=0

log() { printf '[focusa-deploy-adapter] %s\n' "$*"; }
die() { printf '[focusa-deploy-adapter][error] %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Usage: scripts/install-daemon.sh [options]

Compatibility options:
  --binary PATH            Verify this release-asset identity before delegation.
  --expected-version VER   Install this exact immutable release version.
  --install-root PATH      Must remain /usr/local.
  --bin-name NAME          Must remain focusa-daemon.
  --service-name NAME      Must remain focusa-daemon.
  --health-url URL         Forward the canonical daemon health URL.
  --require-service        Require the Rust installer to activate systemd.
  --no-restart             Promote without service registration/restart.
  --no-verify              Rejected: canonical verification cannot be bypassed.
  --help                   Show this help.

The adapter downloads no product artifact itself and never kills by process
name. The signed full release, system unit, one state root, exact process,
health, and rollback are owned by `focusa install --system-install`.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --binary) BINARY="${2:?--binary requires a path}"; shift 2 ;;
    --expected-version) EXPECTED_VERSION="${2:?--expected-version requires a version}"; shift 2 ;;
    --install-root) INSTALL_ROOT="${2:?--install-root requires a path}"; shift 2 ;;
    --bin-name) BIN_NAME="${2:?--bin-name requires a value}"; shift 2 ;;
    --service-name) SERVICE_NAME="${2:?--service-name requires a value}"; shift 2 ;;
    --health-url) HEALTH_URL="${2:?--health-url requires a value}"; shift 2 ;;
    --require-service) REQUIRE_SERVICE=1; shift ;;
    --no-restart) NO_RESTART=1; shift ;;
    --no-verify) NO_VERIFY=1; shift ;;
    --help|-h) usage; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
done

[[ "$INSTALL_ROOT" == /usr/local ]] \
  || die "noncanonical install root rejected: $INSTALL_ROOT (expected /usr/local)"
[[ "$STATE_DIR" == /usr/local/lib/focusa ]] \
  || die "noncanonical state root rejected: $STATE_DIR (expected /usr/local/lib/focusa)"
[[ "$BIN_NAME" == focusa-daemon ]] \
  || die "noncanonical daemon name rejected: $BIN_NAME"
[[ "$SERVICE_NAME" == focusa-daemon ]] \
  || die "noncanonical service name rejected: $SERVICE_NAME"
[[ "$NO_VERIFY" == 0 ]] \
  || die "--no-verify is unsupported; signed release, exact process, and health verification are mandatory"
[[ "$REQUIRE_SERVICE" == 0 || "$NO_RESTART" == 0 ]] \
  || die "--require-service conflicts with --no-restart"

if [[ -n "$BINARY" ]]; then
  [[ -f "$BINARY" && -x "$BINARY" ]] \
    || die "candidate daemon asset is missing or not executable: $BINARY"
  asset_version="$(release_version_from_asset_name "$BINARY" "$EXPECTED_VERSION")"
  [[ -n "$asset_version" ]] \
    || die "candidate daemon path does not identify a canonical release asset: $BINARY"
  if [[ -n "$EXPECTED_VERSION" && "$asset_version" != "$EXPECTED_VERSION" ]]; then
    die "candidate daemon version mismatch: expected $EXPECTED_VERSION, asset identifies $asset_version"
  fi
  EXPECTED_VERSION="${EXPECTED_VERSION:-$asset_version}"
fi
[[ -n "$EXPECTED_VERSION" ]] \
  || die "--expected-version or a canonical --binary asset name is required"

TAG="${FOCUSA_GITHUB_TAG:-v${EXPECTED_VERSION}}"
[[ "$TAG" == "v${EXPECTED_VERSION}" ]] \
  || die "release identity mismatch: tag=$TAG expected=v${EXPECTED_VERSION}"
case "$EXPECTED_VERSION" in
  *-nightly*) CHANNEL=nightly ;;
  *-dev*|*-rc*) CHANNEL=preview ;;
  *) CHANNEL=stable ;;
esac

BOOTSTRAP="$ROOT_DIR/scripts/install-focusa.sh"
[[ -x "$BOOTSTRAP" ]] || die "canonical Rust bootstrap adapter is missing: $BOOTSTRAP"
ARGS=(
  --target=linux
  --channel="$CHANNEL"
  --release-tag="$TAG"
  --github-repo="${FOCUSA_GITHUB_REPOSITORY:-Startempire-Wire/focusa}"
  --accept-license
  --install-dependencies
  --assume-yes
  --system-install
)
[[ "$NO_RESTART" == 0 ]] || ARGS+=(--no-service)

export FOCUSA_DAEMON_URL="$HEALTH_URL"
export FOCUSA_RELEASE_TAG="$TAG"
log "delegating exact full-release lifecycle to Rust: tag=$TAG service=$([[ $NO_RESTART == 0 ]] && printf activate || printf skip)"
exec "$BOOTSTRAP" "${ARGS[@]}"
