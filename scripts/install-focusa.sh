#!/usr/bin/env bash
# ============================================================================
# Focusa Installer — Bash bootstrapper (Spec 112 §15A.4)
#
# SINGLE SOURCE OF TRUTH for the bash install surface.
#   * Served publicly at `https://install.focusa.dev/focusa` (live copy kept
#     byte-identical via `scripts/sync-install-bootstrapper.sh`; parity
#     enforced by `scripts/verify-bootstrapper-parity.sh` in CI).
#   * In-repo copy at `scripts/install-focusa.sh` is the canonical source.
#
# Behavior:
#   * Pre-flight (this script): target detection, channel-aware release
#     selection, license validate via the real license registry
#     (https://wpuiai.com), cosign-verified SHA256SUMS, license.json write,
#     and download of the `focusa` bootstrapper binary.
#   * Install (Rust orchestrator): `exec focusa install --target=auto`,
#     which owns asset downloads for focusa-daemon and focusa-tui, symlink
#     placement, service rendering (systemd user unit on Linux, launchd on
#     macOS), PATH automation + rc-file edit, atomic stash + rollback,
#     smoke test (`focusa --version`), and the first-install walkthrough
#     card. See crates/focusa-cli/src/commands/install.rs.
#
# URLs:
#   * `install.focusa.dev/*` is a public-facing install facade and is
#     preserved as-is for marketing and `curl | bash` UX.
#   * API calls (license registry, GitHub releases, asset CDN) point at
#     absolute real-backend URLs (https://wpuiai.com and
#     https://github.com), never the install facade.
# ============================================================================
set -euo pipefail

# ----------------------------------------------------------------------------
# Defaults — override via env or flags.
# ----------------------------------------------------------------------------
GITHUB_REPO="${GITHUB_REPO:-Startempire-Wire/focusa}"
# Real license registry backend. Operator rule (2026-07-07): preserve
# install.focusa.dev facades; API calls use absolute real-backend URLs.
LICENSE_REGISTRY="${LICENSE_REGISTRY:-https://wpuiai.com}"
LICENSE_VALIDATE_PATH="${LICENSE_VALIDATE_PATH:-/wp-json/wpuiai-ai-cloud/v1/license/validate}"
# License authority — the operator of record for Focusa licenses.
# Source of truth: docs/SPEC_118_LICENSING.md + docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md.
# Operator-facing page: https://install.focusa.dev/license
LICENSE_AUTHORITY_NAME="Wirebot / Phil Overacity LLC"
LICENSE_AUTHORITY_URL="https://wpuiai.com"
LICENSE_AUTHORITY_DOC="https://install.focusa.dev/license"
LICENSE_AUTHORITY_SUPPORT="https://focusa.dev/support"
CHANNEL="${CHANNEL:-stable}"
DRY_RUN="${DRY_RUN:-0}"
EVAL="${EVAL:-0}"
ACCEPT_LICENSE="${ACCEPT_LICENSE:-0}"
NO_SERVICE="${NO_SERVICE:-0}"
FORCE="${FORCE:-0}"
UNINSTALL="${UNINSTALL:-0}"
LICENSE_KEY="${FOCUSA_LICENSE_KEY:-${LICENSE_KEY:-${WPUIAI_LICENSE_KEY:-}}}"
# Customer email for receipt and reissue contact (Spec 118 §6).
LICENSE_EMAIL="${FOCUSA_LICENSE_EMAIL:-${LICENSE_EMAIL:-}}"
TARGET="auto"
BIN_DIR="${HOME}/.focusa/bin"
STATE_DIR="${HOME}/.focusa/state"
CONFIG_DIR="${HOME}/.focusa/config"
LIBEXEC_DIR="${HOME}/.focusa/libexec"
LICENSE_DIR="${HOME}/.config/focusa"
LICENSE_FILE="${LICENSE_DIR}/license.json"
LICENSE_AUTHORITY_FILE="${LICENSE_DIR}/license_authority.json"
LICENSE_RECEIPT_FILE="${LICENSE_DIR}/license_receipt.json"
INSTALL_LOG_FILE="${STATE_DIR}/installs.jsonl"
MAX_CANDIDATES="${MAX_CANDIDATES:-20}"
# Optional local release fixture used by installer lifecycle tests.
# Production remains on GitHub when this is unset.
RELEASE_BASE_URL="${FOCUSA_RELEASE_BASE_URL:-}"

# Preserve pre-existing installations/configuration. The bootstrapper may
# create these paths before handing off to Rust; on a failed clean install
# they must not be left behind as misleading partial state.
PREEXISTING_INSTALL_ROOT=0
PREEXISTING_LICENSE_DIR=0
[ -e "${HOME}/.focusa" ] && PREEXISTING_INSTALL_ROOT=1
[ -e "$LICENSE_DIR" ] && PREEXISTING_LICENSE_DIR=1
BOOTSTRAP_SUCCESS=0

usage() {
  cat <<USAGE
Usage: install-focusa.sh [options]

Options:
  --dry-run                print the install plan; do not write anything
  --eval                   install in eval mode (no license key required)
  --target=auto|linux|darwin|windows-x64|windows-arm64
  --channel=stable|preview|nightly
  --github-repo=OWNER/REPO  override asset host (default: ${GITHUB_REPO})
  --registry=URL           override license registry URL (default: ${LICENSE_REGISTRY})
  --license-key=KEY        commercial install with the given key
  --email=EMAIL            customer email for receipt + reissue (commercial)
  --accept-license         accept BSL 1.1 terms without prompting
  --no-service             skip systemd user unit / launchd registration
  --force                  allow downgrade or overwriting an existing install
  --uninstall              remove an existing install; succeeds if already removed
  --help                   print this help

Environment overrides (lower precedence than flags):
  CHANNEL, DRY_RUN, EVAL, ACCEPT_LICENSE, NO_SERVICE, FORCE,
  LICENSE_KEY, FOCUSA_LICENSE_KEY, LICENSE_KEY, WPUIAI_LICENSE_KEY,
  LICENSE_EMAIL, FOCUSA_LICENSE_EMAIL, EMAIL,
  GITHUB_REPO, LICENSE_REGISTRY, FOCUSA_LICENSE_REGISTRY

Exit codes:
   0 success / dry-run printed
   1 generic failure
   2 license validation failed
  64 unknown argument
  65 required tool missing
  66 unsupported platform
  67 release list missing partial assets
  68 checksum or signature mismatch
USAGE
}

# ----------------------------------------------------------------------------
# Argument parsing.
# ----------------------------------------------------------------------------
for arg in "$@"; do
  case "$arg" in
    --dry-run)            DRY_RUN=1 ;;
    --eval)               EVAL=1 ;;
    --accept-license)     ACCEPT_LICENSE=1 ;;
    --no-service)         NO_SERVICE=1 ;;
    --force)              FORCE=1 ;;
    --uninstall)          UNINSTALL=1 ;;
    --target=*)           TARGET="${arg#--target=}" ;;
    --channel=*)          CHANNEL="${arg#--channel=}" ;;
    --github-repo=*)      GITHUB_REPO="${arg#--github-repo=}" ;;
    --registry=*)         LICENSE_REGISTRY="${arg#--registry=}" ;;
    --license-key=*)      LICENSE_KEY="${arg#--license-key=}" ;;
    --email=*)            LICENSE_EMAIL="${arg#--email=}" ;;
    --help|-h)            usage; exit 0 ;;
    *) printf '[focusa-install] unknown arg: %s\n' "$arg" >&2; usage >&2; exit 64 ;;
  esac
done

# Allow FOCUSA_LICENSE_REGISTRY env override too (matches Rust install.rs).
LICENSE_REGISTRY="${FOCUSA_LICENSE_REGISTRY:-$LICENSE_REGISTRY}"

log()  { printf '\033[1;34m[focusa-install]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[focusa-install]\033[0m %s\n' "$*" >&2; }
err()  { printf '\033[1;31m[focusa-install]\033[0m %s\n' "$*" >&2; }
die()  { err "$@"; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

# ----------------------------------------------------------------------------
# Pre-flight: required tools.
# ----------------------------------------------------------------------------
have curl   || { err "curl is required. recovery_hint: install curl, then retry."; exit 65; }
have python3 || { err "python3 is required. recovery_hint: install python3, then retry."; exit 65; }
have sha256sum || have shasum || { err "sha256sum (or shasum) is required."; exit 65; }

# GitHub/CDN HTTP/2 resets must not turn an otherwise valid install into a
# partial transaction. HTTP/1.1 plus bounded retries is portable on supported
# macOS/Linux curl versions.
curl_resilient() {
  curl --http1.1 --retry 5 --retry-all-errors --retry-delay 2 --connect-timeout 20 "$@"
}

# Idempotent public uninstall entrypoint. Never creates install/license state.
if [ "$UNINSTALL" = 1 ]; then
  uninstall_status=0
  if [ -x "$BIN_DIR/focusa" ]; then
    "$BIN_DIR/focusa" uninstall --yes || uninstall_status=$?
  else
    log "Focusa binaries are already removed."
  fi
  # The shell bootstrapper owns Pi extension installation, so it also owns
  # symmetric removal. Limit deletion to the exact managed extension path.
  PI_MANAGED_DEST="${FOCUSA_PI_EXT_DIR:-${HOME}/.pi/agent/extensions}/focusa"
  if [ -e "$PI_MANAGED_DEST" ]; then
    rm -rf "$PI_MANAGED_DEST"
    log "removed Focusa Pi extension: $PI_MANAGED_DEST"
  fi
  [ "$uninstall_status" -eq 0 ] || exit "$uninstall_status"
  log "Focusa uninstall complete."
  exit 0
fi

# ----------------------------------------------------------------------------
# Pre-flight: BSL 1.1 acceptance gate. Commercial installs must accept
# the BSL terms. Eval is permitted without acceptance.
# ----------------------------------------------------------------------------
bsl_summary() {
  cat <<BSL
Focusa is source-available under the Business Source License 1.1 (BSL 1.1).

  Permitted without a license key:
    - Personal, educational, and non-commercial local use
    - Evaluation on real projects (--eval mode)
    - Reading and studying the source tree

  NOT permitted without a commercial license:
    - Production deployments
    - Hosted services that bill customers
    - Client delivery, consulting, or agency work
    - Team or company use that materially supports commercial operations
    - Embedding in a commercial product
    - Redistribution or resale

  Full terms:  https://focusa.dev/LICENSE  (BSL 1.1)
  Pricing:     https://install.focusa.dev/license      (Operator Lifetime $697)
  Authority:   ${LICENSE_AUTHORITY_NAME}  (${LICENSE_AUTHORITY_URL})
BSL
}

if [ "$EVAL" = 0 ] && [ -z "$LICENSE_KEY" ] && [ "$ACCEPT_LICENSE" = 0 ]; then
  err "Commercial install requires --accept-license or a --license-key."
  err "Use --eval to install in evaluation mode (BSL 1.1 non-commercial use)."
  err "license authority: ${LICENSE_AUTHORITY_NAME} <${LICENSE_AUTHORITY_URL}>"
  exit 64
fi

# ----------------------------------------------------------------------------
# Pre-flight: existing install / license check (anti-rollback + idempotency).
# * Refuse to downgrade unless --force.
# * Refuse to overwrite a different-key license unless --force.
# * Migrate legacy license.json (customer_email: null) on read.
# ----------------------------------------------------------------------------
INSTALLED_VERSION_FILE="${STATE_DIR}/installed_version"
INSTALLED_VERSION=""
[ -s "$INSTALLED_VERSION_FILE" ] && INSTALLED_VERSION="$(cat "$INSTALLED_VERSION_FILE" 2>/dev/null || true)"

migrate_legacy_license() {
  [ -f "$LICENSE_FILE" ] || return 0
  python3 - "$LICENSE_FILE" <<'PY' 2>/dev/null || true
import json, os, sys
path = sys.argv[1]
try:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
except Exception:
    sys.exit(0)
changed = False
# Spec §5.1: customer_email is nullable in the daemon parser; the legacy
# live installer wrote literal null. Normalize to "" so future activations
# see a consistent shape.
if "customer_email" in data and data["customer_email"] is None:
    data["customer_email"] = ""
    changed = True
# Add registry_authority metadata if missing.
if "registry_authority" not in data:
    data["registry_authority"] = {
        "name": "Wirebot / Phil Overacity LLC",
        "url": "https://wpuiai.com",
        "doc": "https://install.focusa.dev/license",
    }
    changed = True
if changed:
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)
        f.write("\n")
    os.chmod(tmp, 0o600)
    os.replace(tmp, path)
PY
}

# ----------------------------------------------------------------------------
# Pre-flight: write the license authority record. Written BEFORE validation
# so the operator can see at a glance which authority governs this install,
# even if the network call fails.
# ----------------------------------------------------------------------------
write_license_authority() {
  mkdir -p "$LICENSE_DIR"
  cat > "$LICENSE_AUTHORITY_FILE" <<JSON
{
  "name": "${LICENSE_AUTHORITY_NAME}",
  "url": "${LICENSE_AUTHORITY_URL}",
  "doc": "${LICENSE_AUTHORITY_DOC}",
  "support": "${LICENSE_AUTHORITY_SUPPORT}",
  "registry_url": "${LICENSE_REGISTRY}",
  "validate_path": "${LICENSE_VALIDATE_PATH}",
  "spec_refs": [
    "docs/SPEC_118_LICENSING.md",
    "docs/SPEC_119_LIFETIME_TO_RECURRING_TRANSITION.md"
  ],
  "written_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "channel": "${CHANNEL}",
  "target": "${TARGET}"
}
JSON
  chmod 600 "$LICENSE_AUTHORITY_FILE"
}

write_license_receipt() {
  local tier="$1" status="$2" expires="$3" customer_email="$4" eval_flag="$5"
  mkdir -p "$LICENSE_DIR"
  python3 - "$LICENSE_RECEIPT_FILE" "$tier" "$status" "$expires" \
                     "$customer_email" "$eval_flag" "$LICENSE_AUTHORITY_NAME" \
                     "$LICENSE_AUTHORITY_URL" <<'PY'
import json, os, sys
from datetime import datetime, timezone
(path, tier, status, expires, customer_email, eval_flag,
 authority_name, authority_url) = sys.argv[1:]
ef = (eval_flag or "").strip().lower()
is_eval = ef in ("true", "1", "yes")
data = {
    "issued_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "tier": tier,
    "status": status,
    "expires_at": expires or None,
    "customer_email": customer_email or None,
    "authority": {"name": authority_name, "url": authority_url},
    "eval": is_eval,
    "note": "Save this receipt with your purchase confirmation. "
            "It is the only durable local record of which authority "
            "and tier issued this license."
}
tmp = path + ".tmp"
with open(tmp, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
os.chmod(tmp, 0o600)
os.replace(tmp, path)
PY
  chmod 600 "$LICENSE_RECEIPT_FILE"
  log "wrote receipt to ${LICENSE_RECEIPT_FILE}"
}

# ----------------------------------------------------------------------------
# Pre-flight: platform detection → target triple.
# ----------------------------------------------------------------------------
HOST_OS=$(uname -s); HOST_ARCH=$(uname -m)
# Rosetta 2 correction: if running x86_64 on an Apple Silicon host, prefer arm64.
if [ "$HOST_OS" = "Darwin" ] && [ "$HOST_ARCH" = "x86_64" ]; then
  if sysctl -n hw.optional.arm64 2>/dev/null | grep -q '^1$'; then
    HOST_ARCH="aarch64"
    log "detected native arm64 host via sysctl (Rosetta corrected x86_64 -> aarch64)"
  fi
fi
case "$HOST_OS" in
  Linux|Darwin) ;;
  MINGW*|MSYS*|CYGWIN*)
    err "Windows native installs must use https://install.focusa.dev/focusa.ps1 in PowerShell."
    exit 66 ;;
  *) err "unsupported OS: $HOST_OS"; exit 66 ;;
esac
case "$HOST_ARCH" in x86_64|aarch64|arm64) ;; *) err "unsupported arch: $HOST_ARCH"; exit 66 ;; esac

if [ "$TARGET" = "auto" ]; then
  case "$HOST_OS-$HOST_ARCH" in
    # Static musl is the portable default for older production glibc hosts.
    Linux-x86_64)   TRIPLE="x86_64-unknown-linux-musl" ;;
    Linux-aarch64)  TRIPLE="aarch64-unknown-linux-gnu" ;;
    Darwin-x86_64)  TRIPLE="x86_64-apple-darwin" ;;
    Darwin-arm64|Darwin-aarch64) TRIPLE="aarch64-apple-darwin" ;;
    *) err "unsupported host: $HOST_OS-$HOST_ARCH"; exit 66 ;;
  esac
  TARGET="$TRIPLE"
  # Map host OS to Rust InstallTarget enum variant.
  case "$HOST_OS" in
    Linux)   RUST_TARGET="linux" ;;
    Darwin)  RUST_TARGET="darwin" ;;
    Windows) RUST_TARGET="windows-x64" ;;
  esac
fi

# ----------------------------------------------------------------------------
# Channel → release-tag pattern.
# ----------------------------------------------------------------------------
case "$CHANNEL" in
  stable)  TAG_PATTERN='v[0-9]+\.[0-9]+\.[0-9]+' ;;
  preview) TAG_PATTERN='v[0-9]+\.[0-9]+\.[0-9]+-(dev|rc)(\..*)?' ;;
  dev)     TAG_PATTERN='v[0-9]+\.[0-9]+\.[0-9]+-dev' ;;
  nightly) TAG_PATTERN='v[0-9]+\.[0-9]+\.[0-9]+-nightly\..*' ;;
  *) err "unknown channel: $CHANNEL"; exit 1 ;;
esac

# ----------------------------------------------------------------------------
# Scratch tmpdir for the install transaction. Cleaned up on exit.
# ----------------------------------------------------------------------------
TMP="$(mktemp -d "${TMPDIR:-/tmp}/focusa-install.XXXXXX")"
if [ "$PREEXISTING_LICENSE_DIR" = 1 ]; then
  cp -a "$LICENSE_DIR" "$TMP/license-dir.before"
fi
cleanup_bootstrap_failure() {
  local status=$?
  if [ "$status" -ne 0 ] && [ "$BOOTSTRAP_SUCCESS" != 1 ]; then
    [ "$PREEXISTING_INSTALL_ROOT" = 1 ] || rm -rf "${HOME}/.focusa"
    if [ "$PREEXISTING_LICENSE_DIR" = 1 ]; then
      rm -rf "$LICENSE_DIR"
      cp -a "$TMP/license-dir.before" "$LICENSE_DIR"
    else
      rm -rf "$LICENSE_DIR"
    fi
    err "install failed; restored exact pre-bootstrap state"
  fi
  rm -rf "$TMP"
}
trap cleanup_bootstrap_failure EXIT

# ----------------------------------------------------------------------------
# Discover the latest COMPLETE release for this target. A complete release
# must ship all three binaries (focusa, focusa-daemon, focusa-tui) for the
# target triple. Partial releases are skipped automatically.
# ----------------------------------------------------------------------------
log "fetching release list (channel=${CHANNEL} target=${TARGET})"
RELEASES_FILE="$TMP/releases.json"
if [ -z "$RELEASE_BASE_URL" ]; then
  curl_resilient -fsSL "https://api.github.com/repos/${GITHUB_REPO}/releases?per_page=${MAX_CANDIDATES}" \
    -o "$RELEASES_FILE" \
    || die "failed to fetch release list from GitHub"
fi

pick_complete_release() {
  python3 - "$RELEASES_FILE" "$TAG_PATTERN" "$TARGET" "$MAX_CANDIDATES" <<'PY'
import json, re, sys
path, pattern, triple, max_n = sys.argv[1:]
with open(path, "r", encoding="utf-8") as f:
    data = json.load(f)
pat_re = re.compile("^(?:" + pattern + ")$")
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

if [ -n "$RELEASE_BASE_URL" ] && [ -n "${FOCUSA_RELEASE_TAG:-}" ]; then
  SELECTED="${FOCUSA_RELEASE_TAG}"$'\t'"${RELEASE_BASE_URL%/}/focusa-${FOCUSA_RELEASE_TAG}-${TARGET}"
else
  SELECTED="$(pick_complete_release)"
fi
if [ -z "$SELECTED" ]; then
  err "no complete release for channel='${CHANNEL}' target='${TARGET}'"
  err "recovery_hint: a complete release ships focusa + focusa-daemon + focusa-tui for ${TARGET}."
  exit 67
fi
RELEASE_TAG="${SELECTED%%	*}"
ASSET_URL="${SELECTED##*	}"
[ -n "$RELEASE_TAG" ] && [ -n "$ASSET_URL" ] || die "could not parse selected release"
log "selected: tag=${RELEASE_TAG} target=${TARGET}"

# ----------------------------------------------------------------------------
# Pre-flight: license phase.
#   * --eval: write a self-signed eval license.json with offline grace.
#   * --license-key: POST to the real license registry; on valid response,
#     write the daemon-compatible license.json with offline_valid_until
#     +7 days so the operator can install offline for one week.
#   * Neither: refused above (BSL acceptance gate).
# ----------------------------------------------------------------------------
key_hash() { printf '%s' "$1" | sha256sum 2>/dev/null | awk '{print $1}' || \
             printf '%s' "$1" | shasum -a 256 | awk '{print $1}'; }

# Compute offline_valid_until = now + 7 days (ISO 8601 UTC).
offline_until() {
  python3 - <<'PY'
from datetime import datetime, timedelta, timezone
print((datetime.now(timezone.utc) + timedelta(days=7)).strftime("%Y-%m-%dT%H:%M:%SZ"))
PY
}

write_license_json() {
  local key_hash_val="$1" key_prefix_val="$2" product="$3" tier="$4" \
        status="$5" commercial="$6" features="$7" expires="$8" \
        activated="$9" eval_flag="${10}"
  mkdir -p "$LICENSE_DIR"
  OFFLINE="$(offline_until)"
  python3 - "$LICENSE_FILE" "$key_hash_val" "$key_prefix_val" "$product" "$tier" \
                  "$status" "$commercial" "$features" "$expires" "$activated" \
                  "$eval_flag" "$OFFLINE" "$LICENSE_REGISTRY" <<'PY'
import json, os, sys
(path, key_hash, key_prefix, product, tier, status,
 commercial, features, expires, activated, eval_flag,
 offline_until, registry_url) = sys.argv[1:]
data = {
    "key_hash": key_hash,
    "key_prefix": key_prefix,
    "product": product,
    "tier": tier,
    "status": status,
    "commercial_use": json.loads(commercial.lower() if commercial.lower() in ("true","false") else commercial),
    "customer_email": None,
    "features": json.loads(features or "[]"),
    "expires_at": expires or None,
    "offline_valid_until": offline_until,
    "registry_url": registry_url,
    "activated_at": activated or None,
    "eval": json.loads(eval_flag.lower() if eval_flag.lower() in ("true","false") else eval_flag),
}
tmp = path + ".tmp"
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(tmp, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
os.chmod(tmp, 0o600)
os.replace(tmp, path)
PY
  chmod 600 "$LICENSE_FILE"
  log "wrote license state to ${LICENSE_FILE}"
}

if [ -n "$LICENSE_KEY" ]; then
  log "validating license against ${LICENSE_REGISTRY}${LICENSE_VALIDATE_PATH}"
  VALIDATE_RESP="$(
    curl_resilient -sS -X POST \
      -H "Content-Type: application/json" \
      -H "X-License-Key: $LICENSE_KEY" \
      -d "{\"license_key\":\"$LICENSE_KEY\"}" \
      "${LICENSE_REGISTRY}${LICENSE_VALIDATE_PATH}" \
      2>/dev/null || true
  )"
  if ! printf '%s' "$VALIDATE_RESP" | grep -q '"valid"[[:space:]]*:[[:space:]]*true'; then
    err "license validation failed."
    err "response: ${VALIDATE_RESP:-(empty)}"
    err "purchase/manage license: https://install.focusa.dev/license"
    err "recovery_hint: re-run with --eval, or fix the key, or check the registry URL."
    err "license authority: ${LICENSE_AUTHORITY_NAME} <${LICENSE_AUTHORITY_URL}>"
    exit 2
  fi
  json_get() { python3 -c "import json,sys;print(json.loads(sys.stdin.read()).get(\"$1\", \"\"))" ; }

  # Operator rule (2026-07-07): dev_mode responses exist for testing and must
  # not hinder transactions. A real commercial buyer must have a real
  # license row in the registry; the registry returns status="active" in
  # that case. status="dev_mode" means "this is a test fixture, not a
  # purchase". Downgrade to eval so the install succeeds without
  # silently granting commercial privileges.
  RESP_STATUS="$(printf '%s' "$VALIDATE_RESP" | json_get status)"
  RESP_TIER="$(printf '%s' "$VALIDATE_RESP" | json_get tier)"
  if [ "$RESP_STATUS" = "dev_mode" ] && [ "${FOCUSA_REQUIRE_REAL_LICENSE:-0}" = 1 ]; then
    err "registry returned status=dev_mode for a license key."
    err "this key did not resolve to a real license row."
    err "recovery_hint: remove FOCUSA_REQUIRE_REAL_LICENSE or purchase at ${LICENSE_AUTHORITY_URL}/buy."
    exit 2
  fi
  if [ "$RESP_STATUS" = "dev_mode" ]; then
    warn "registry returned status=dev_mode; this is a TEST FIXTURE, not a real purchase."
    warn "installing in EVAL mode (tier=evaluation, commercial_use=false)."
    warn "to use this key commercially, complete the purchase at ${LICENSE_AUTHORITY_URL}/buy."
    log "switching to --eval semantics for the rest of this install"
    LICENSE_KEY=""          # so downstream branches treat this as eval
    EVAL=1
  fi
  KH="$(key_hash "$LICENSE_KEY")"
  KP="${LICENSE_KEY:0:16}"
  PRODUCT="$(printf '%s' "$VALIDATE_RESP" | json_get product)"; PRODUCT="${PRODUCT:-focusa}"
  TIER="$(printf '%s' "$VALIDATE_RESP" | json_get tier)"; TIER="${TIER:-operator}"
  STATUS="$(printf '%s' "$VALIDATE_RESP" | json_get status)"; STATUS="${STATUS:-active}"
  COMMERCIAL="$(printf '%s' "$VALIDATE_RESP" | json_get commercial_use)"; COMMERCIAL="${COMMERCIAL:-true}"
  FEATURES="$(printf '%s' "$VALIDATE_RESP" | json_get features)"; FEATURES="${FEATURES:-[]}"
  EXPIRES="$(printf '%s' "$VALIDATE_RESP" | json_get expires_at)"
  ACTIVATED="$(printf '%s' "$VALIDATE_RESP" | json_get activated_at)"
  # Customer email: prefer registry response, then --email flag, then empty.
  RESP_EMAIL="$(printf '%s' "$VALIDATE_RESP" | json_get customer_email)"
  CUSTOMER_EMAIL="${RESP_EMAIL:-${LICENSE_EMAIL:-}}"
  log "license valid: tier=${TIER}"
  write_license_authority
  write_license_json "$KH" "$KP" "$PRODUCT" "$TIER" "$STATUS" "$COMMERCIAL" \
                     "$FEATURES" "$EXPIRES" "$ACTIVATED" "false"
  write_license_receipt "$TIER" "$STATUS" "$EXPIRES" "$CUSTOMER_EMAIL" "false"
elif [ "$EVAL" = 1 ]; then
  log "eval mode: writing self-signed license.json with 7-day offline grace"
  KH="eval"
  KP="eval-$(date -u +%Y%m%d)"
  write_license_authority
  write_license_json "$KH" "$KP" "focusa" "evaluation" "active" "false" \
                     '["daemon","tui","cli"]' "" "" "true"
  write_license_receipt "evaluation" "active" "" "" "true"
else
  # Should be unreachable (BSL gate above). Surface a clear error.
  err "no license key provided and --eval not set. pass --eval or --license-key."
  exit 64
fi

# ----------------------------------------------------------------------------
# Migrate any pre-existing legacy license.json (customer_email: null shape)
# so the daemon parser accepts it.
# ----------------------------------------------------------------------------
migrate_legacy_license

# ----------------------------------------------------------------------------
# Download focusa bootstrapper binary for this target triple.
# ----------------------------------------------------------------------------
ASSET_FOCUSA="focusa-${RELEASE_TAG}-${TARGET}"
log "downloading ${ASSET_FOCUSA}"

curl_resilient -fsSL "$ASSET_URL" -o "$TMP/focusa" || die "download failed: $ASSET_URL"
chmod +x "$TMP/focusa"

# ----------------------------------------------------------------------------
# Verify SHA256SUMS if available. Prefer cosign-signed manifest, fall back
# to plain SHA256SUMS, fall back to skip-with-warning.
# ----------------------------------------------------------------------------
release_asset_url() {
  local name="$1"
  if [ -n "$RELEASE_BASE_URL" ]; then
    printf '%s/%s' "${RELEASE_BASE_URL%/}" "$name"
  else
    printf 'https://github.com/%s/releases/download/%s/%s' "$GITHUB_REPO" "$RELEASE_TAG" "$name"
  fi
}

CHECKSUM_MANIFEST=""
for sha_path in SHA256SUMS.txt SHA256SUMS; do
  if curl_resilient -fsSL "$(release_asset_url "$sha_path")" \
       -o "$TMP/$sha_path" 2>/dev/null; then
    CHECKSUM_MANIFEST="$TMP/$sha_path"
    break
  fi
done

verify_signature() {
  local manifest="$1"
  [ -s "$manifest" ] || return 1
  if ! have cosign; then
    warn "cosign not installed; falling back to SHA256SUMS-only verification"
    return 1
  fi
  local base sig cert
  base="$(basename "$manifest")"
  sig="$TMP/${base}.sig"; cert="$TMP/${base}.pem"
  if curl_resilient -fsSL "$(release_asset_url "${base}.sig")" -o "$sig" 2>/dev/null \
     && curl_resilient -fsSL "$(release_asset_url "${base}.pem")" -o "$cert" 2>/dev/null; then
    # GitHub release assets store cosign sig/pem as base64. Decode for cosign v3 compatibility.
    if python3 -c "
import base64, sys
for p in sys.argv[1:]:
    data = open(p,'rb').read().strip()
    try: open(p,'wb').write(base64.b64decode(data, validate=True))
    except: pass
" "$sig" "$cert" 2>/dev/null; then
      : # decoded
    fi
    # cosign v3 requires explicit identity + issuer for keyless verification.
    identity="https://github.com/${GITHUB_REPO}/.github/workflows/release.yml@refs/tags/${RELEASE_TAG}"
    issuer="https://token.actions.githubusercontent.com"
    if cosign verify-blob --cert "$cert" --signature "$sig" \
       --certificate-identity "$identity" --certificate-oidc-issuer "$issuer" \
       "$manifest" >/dev/null 2>&1; then
      log "cosign signature verified: ${base}"
      return 0
    fi
    err "cosign signature verification failed for ${base}"
    return 1
  fi
  warn "no cosign .sig/.pem found next to ${base}; skipping signature verify"
  return 1
}

verified_signature=1
if [ -n "$CHECKSUM_MANIFEST" ]; then
  log "checksum manifest: $(basename "$CHECKSUM_MANIFEST")"
  if ! verify_signature "$CHECKSUM_MANIFEST"; then
    verified_signature=0
  fi
  EXPECTED="$(awk -v n="${ASSET_FOCUSA}" '$2 == n {print $1; exit}' "$CHECKSUM_MANIFEST")"
  if [ -z "$EXPECTED" ]; then
    err "no checksum entry for ${ASSET_FOCUSA} in $(basename "$CHECKSUM_MANIFEST")"
    exit 68
  fi
  ACTUAL="$(sha256sum "$TMP/focusa" 2>/dev/null | awk '{print $1}' || shasum -a 256 "$TMP/focusa" 2>/dev/null | awk '{print $1}')"
  [ "$ACTUAL" = "$EXPECTED" ] || { err "checksum mismatch for focusa (expected $EXPECTED, got $ACTUAL)"; exit 68; }
  log "sha256 verified: ${ACTUAL:0:12}…"
else
  warn "no SHA256SUMS asset for ${RELEASE_TAG}; digest verification is incomplete until release signing lands."
fi

sha_ok=0
if [ -n "$CHECKSUM_MANIFEST" ] && [ -s "$CHECKSUM_MANIFEST" ]; then
  sha_ok=1
fi
if [ "$verified_signature" != 1 ]; then
  case "$CHANNEL" in
    stable)
      err "stable install requires valid Cosign signature metadata; SHA256 alone is insufficient"
      exit 68
      ;;
    preview|dev|nightly)
      [ "$sha_ok" = 1 ] || exit 68
      warn "${CHANNEL} channel: SHA256 verified but Cosign metadata is absent; install is preview-only"
      ;;
  esac
fi

# ----------------------------------------------------------------------------
# Place the bootstrapper binary and hand off to the Rust orchestrator.
# ----------------------------------------------------------------------------
mkdir -p "$BIN_DIR" "$STATE_DIR" "$CONFIG_DIR" "$LIBEXEC_DIR"

# Anti-rollback: refuse to downgrade an existing install unless --force.
INSTALLED_VERSION_FILE="${STATE_DIR}/installed_version"
if [ -s "$INSTALLED_VERSION_FILE" ] && [ "$FORCE" != 1 ]; then
  CURRENT_VER="$(cat "$INSTALLED_VERSION_FILE" 2>/dev/null || echo unknown)"
  if python3 - "$CURRENT_VER" "$RELEASE_TAG" <<'PY' 2>/dev/null
import sys
def parse(v):
    s = v.lstrip('v')
    parts = s.split('-')[0].split('.')
    try:
        return tuple(int(p) for p in parts)
    except Exception:
        return None
a, b = sys.argv[1], sys.argv[2]
pa, pb = parse(a), parse(b)
if pa is None or pb is None:
    sys.exit(0)
sys.exit(0 if pb >= pa else 1)
PY
  then
    : # current >= selected; allow
  else
    err "refusing to downgrade: installed=${CURRENT_VER} selected=${RELEASE_TAG}"
    err "recovery_hint: re-run with --force to overwrite, or pick a newer channel."
    exit 68
  fi
fi

# Execute the verified bootstrap binary from transaction scratch. Never
# overwrite the live CLI before the Rust orchestrator has committed all assets.
chmod 0755 "$TMP/focusa"
BOOTSTRAP_BIN="$TMP/focusa"

record_install_success() {
  mkdir -p "$STATE_DIR"
  printf '%s\n' "$RELEASE_TAG" > "$INSTALLED_VERSION_FILE"
  chmod 0644 "$INSTALLED_VERSION_FILE"

  if [ "$DRY_RUN" != 1 ]; then
  install_event_tier="${TIER:-evaluation}"
  install_event_eval="false"
  [ "$EVAL" = 1 ] && install_event_eval="true"
  install_event_channel="$CHANNEL"
  install_event_target="$TARGET"
  install_event_key_hash="$(key_hash "$LICENSE_KEY" 2>/dev/null || echo unknown)"
  install_event_email="${LICENSE_EMAIL:-}"
  install_event_ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  install_event_host="$(uname -n 2>/dev/null || echo unknown)"
  install_event_os="$(uname -s)"
  install_event_arch="$(uname -m)"
  install_event_bin_dir="$BIN_DIR"
  install_event_license_authority="$LICENSE_AUTHORITY_NAME"
  install_event_registry="$LICENSE_REGISTRY"
  {
    printf '{"event":"install","ts":"%s","channel":"%s","target":"%s","tag":"%s","tier":"%s","eval":%s,"key_hash":"%s","customer_email":"%s","host":"%s","os":"%s","arch":"%s","bin_dir":"%s","license_authority":"%s","registry":"%s"}\n' \
      "$install_event_ts" "$install_event_channel" "$install_event_target" "$RELEASE_TAG" \
      "$install_event_tier" "$install_event_eval" "$install_event_key_hash" \
      "$install_event_email" "$install_event_host" "$install_event_os" \
      "$install_event_arch" "$install_event_bin_dir" "$install_event_license_authority" \
      "$install_event_registry"
  } >> "$INSTALL_LOG_FILE"
    chmod 0600 "$INSTALL_LOG_FILE"
  fi
}

# When Pi is installed, install the matching bundled extension beside its
# other extensions. Extension failure never corrupts an existing extension or
# aborts the CLI install, but an unverified/broken extension is never activated.
# Pi extension integration is owned by the Rust orchestrator.


log "handing off to Rust orchestrator: focusa install --target=${TARGET}"
if [ "$DRY_RUN" = 1 ]; then
  log "DRY RUN: would exec verified scratch bootstrap install --target=$RUST_TARGET --github-repo=$GITHUB_REPO ..."
  exit 0
fi

# Forward every relevant flag; the Rust orchestrator owns the rest.
ARGS=(install --target="$RUST_TARGET" --github-repo="$GITHUB_REPO")
[ "$EVAL" = 1 ] && ARGS+=(--eval)
[ "$NO_SERVICE" = 1 ] && ARGS+=(--no-service)
[ "$ACCEPT_LICENSE" = 1 ] && ARGS+=(--accept-license)
[ "$CHANNEL" != "stable" ] && ARGS+=(--channel="$CHANNEL")
[ -n "$LICENSE_KEY" ] && ARGS+=(--license-key="$LICENSE_KEY")
# Bind the Rust orchestrator to the exact release selected and verified above.
# Without this, the embedded channel default can drift from the bootstrapper.
export FOCUSA_RELEASE_TAG="$RELEASE_TAG"
[ -z "$RELEASE_BASE_URL" ] || export FOCUSA_RELEASE_BASE_URL="$RELEASE_BASE_URL"
# Run rather than exec so an orchestrator failure reaches the EXIT trap.
# The trap removes only clean-state paths created by this bootstrapper.
if "$BOOTSTRAP_BIN" "${ARGS[@]}"; then
  record_install_success
  BOOTSTRAP_SUCCESS=1
  exit 0
else
  status=$?
  err "Rust install orchestrator failed (exit ${status})"
  exit "$status"
fi