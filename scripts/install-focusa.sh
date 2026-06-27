#!/usr/bin/env bash
# ============================================================================
# Focusa Installer
# Source: https://install.focusa.dev/focusa
# Docs:   https://install.focusa.dev (see "Install" section)
# License: BSL 1.1 — see https://focusa.dev/LICENSE
# ============================================================================
set -euo pipefail

PREFIX="${HOME}/.focusa"
CHANNEL="stable"
VERSION=""
DRY_RUN=0
EVAL=0
LICENSE_KEY=""
UNINSTALL=0
WITH_ENGINE=0
WITH_PI=0
WITH_OPENCLAW=0
ACCEPT_LICENSE=0
NO_SERVICE=0
LICENSE_REGISTRY="https://install.focusa.dev"
GITHUB_REPO="Startempire-Wire/focusa"
PRODUCT="focusa"
TIER="operator"
VALIDATE_RESP=""

log() { printf '\033[1;34m[focusa-install]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[focusa-install]\033[0m %s\n' "$*"; }
err() { printf '\033[1;31m[focusa-install]\033[0m %s\n' "$*" >&2; }
die() { err "$@"; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

usage() {
  cat <<'USAGE'
Usage: bash focusa-install.sh [options]

Options:
  --eval                 Install in evaluation mode; skips commercial license validation
  --license-key <key>    Activate a Focusa license key (focusa_live_xxxxx)
  --prefix <path>        Install prefix (default: ~/.focusa)
  --channel <name>       stable | preview (default: stable)
  --version <tag>        Install explicit GitHub release tag, e.g. v0.9.25-dev
  --dry-run              Show what would happen; do not write
  --uninstall            Remove installed files at --prefix
  --with-engine          Also run the UIAI Engine installer
  --with-pi              Install Pi wrapper if missing
  --with-openclaw        Unsupported until an actual bridge installer exists
  --no-service           Do not install systemd/launchd service
  --accept-license       Confirm commercial BSL terms for paid install
  -h, --help             Show help
USAGE
}

while [ $# -gt 0 ]; do
  case "$1" in
    --eval) EVAL=1 ;;
    --license-key) LICENSE_KEY="${2:-}"; shift ;;
    --prefix) PREFIX="${2:-}"; shift ;;
    --channel) CHANNEL="${2:-}"; shift ;;
    --version) VERSION="${2:-}"; shift ;;
    --dry-run) DRY_RUN=1 ;;
    --uninstall) UNINSTALL=1 ;;
    --with-engine) WITH_ENGINE=1 ;;
    --with-pi) WITH_PI=1 ;;
    --with-openclaw) WITH_OPENCLAW=1 ;;
    --no-service) NO_SERVICE=1 ;;
    --accept-license) ACCEPT_LICENSE=1 ;;
    -h|--help) usage; exit 0 ;;
    *) die "Unknown option: $1" ;;
  esac
  shift
done

run() {
  if [ "$DRY_RUN" -eq 1 ]; then
    log "DRY RUN: $*"
  else
    eval "$@"
  fi
}

fetch() {
  url="$1"; out="$2"
  if have curl; then curl -fL --retry 3 --connect-timeout 15 -o "$out" "$url"; return; fi
  if have wget; then wget -O "$out" "$url"; return; fi
  die "curl or wget is required. recovery_hint: install curl, then rerun https://install.focusa.dev/focusa"
}

fetch_text() {
  url="$1"
  if have curl; then curl -fsSL --retry 3 --connect-timeout 15 "$url"; return; fi
  if have wget; then wget -qO- "$url"; return; fi
  die "curl or wget is required. recovery_hint: install curl, then rerun https://install.focusa.dev/focusa"
}

post_license_validate() {
  [ -n "$LICENSE_KEY" ] || die "Missing --license-key for commercial install."
  if have curl; then
    curl -sS -X POST \
      -H "Content-Type: application/json" \
      -H "X-License-Key: $LICENSE_KEY" \
      -d "{\"license_key\":\"$LICENSE_KEY\"}" \
      "$LICENSE_REGISTRY/wp-json/wpuiai-ai-cloud/v1/license/validate"
    return
  fi
  die "curl is required for license validation against install.focusa.dev. recovery_hint: install curl or use --eval."
}

sha256_file() {
  file="$1"
  if have sha256sum; then sha256sum "$file" | awk '{print $1}'; return; fi
  if have shasum; then shasum -a 256 "$file" | awk '{print $1}'; return; fi
  die "sha256sum or shasum is required for artifact verification. recovery_hint: install coreutils or Perl shasum."
}

sha256_string() {
  value="$1"
  if have sha256sum; then printf '%s' "$value" | sha256sum | awk '{print $1}'; return; fi
  if have shasum; then printf '%s' "$value" | shasum -a 256 | awk '{print $1}'; return; fi
  die "sha256sum or shasum is required for local license hashing."
}

verify_checksum_manifest_signature() {
  manifest="$1"
  sig="$tmp/SHA256SUMS.txt.sig"
  cert="$tmp/SHA256SUMS.txt.pem"
  if fetch "$ASSET_BASE/SHA256SUMS.txt.sig" "$sig" >/dev/null 2>&1 \
    && fetch "$ASSET_BASE/SHA256SUMS.txt.pem" "$cert" >/dev/null 2>&1; then
    if ! have cosign; then
      if [ "${FOCUSA_REQUIRE_COSIGN:-0}" = "1" ]; then
        die "Signed checksum manifest found but cosign is not installed. recovery_hint: install cosign or unset FOCUSA_REQUIRE_COSIGN."
      fi
      warn "Signed checksum manifest found but cosign is not installed; continuing with SHA256 verification only. Set FOCUSA_REQUIRE_COSIGN=1 to fail closed."
      return
    fi
    cosign verify-blob \
      --certificate "$cert" \
      --signature "$sig" \
      --certificate-identity-regexp "https://github.com/Startempire-Wire/focusa/.github/workflows/release.yml@refs/tags/v.*" \
      --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
      "$manifest" >/dev/null
    log "Verified signed checksum manifest with cosign."
  else
    warn "No cosign signature found for SHA256SUMS.txt; checksum verification is unsigned for $TAG."
  fi
}

json_get() {
  key="$1"
  if have python3; then
    python3 -c 'import json,sys; d=json.load(sys.stdin); v=d.get(sys.argv[1], ""); print(json.dumps(v,separators=(",",":")) if isinstance(v,(list,dict)) else ("true" if v is True else "false" if v is False else "" if v is None else v))' "$key" 2>/dev/null || true
  else
    sed -n "s/.*\"$key\"[[:space:]]*:[[:space:]]*\"\([^\"]*\)\".*/\1/p" | head -1
  fi
}

if [ "$UNINSTALL" -eq 1 ]; then
  log "Removing $PREFIX"
  run "rm -rf '$PREFIX'"
  log "Uninstall complete. Remove ~/.config/focusa/license.json manually if desired."
  exit 0
fi

if [ "$WITH_OPENCLAW" -eq 1 ]; then
  die "--with-openclaw is not a supported Focusa installer surface yet; no placeholder installs. recovery_hint: omit --with-openclaw."
fi

if [ "$EVAL" -eq 0 ] && [ "$ACCEPT_LICENSE" -eq 0 ]; then
  cat <<'WARN'

  Focusa is source-available under the Business Source License 1.1.

  Personal, educational, evaluation, and non-commercial local use is permitted.
  Commercial use, hosted services, client delivery, team use, product embedding,
  and redistribution require a paid license from WPUIAI / Startempire Wire.

  See https://focusa.dev/LICENSE and https://focusa.dev/COMMERCIAL.md

  To continue as a personal/evaluation user, re-run with --eval.
  To install for commercial use, run with --accept-license and --license-key.

WARN
  [ -n "$LICENSE_KEY" ] || die "Refusing to install without a license key. Use --eval or pass --license-key + --accept-license."
fi

if [ "$EVAL" -eq 0 ] && [ -z "$LICENSE_KEY" ]; then
  die "Commercial install requires --license-key. Use --eval for evaluation mode."
fi

case "$(uname -s 2>/dev/null || echo unknown)" in
  Darwin)
    arch="$(uname -m)"
    if [ "$arch" = "arm64" ]; then TARGET="aarch64-apple-darwin"; else TARGET="x86_64-apple-darwin"; fi
    ;;
  Linux)
    arch="$(uname -m)"
    case "$arch" in x86_64|amd64) cpu="x86_64" ;; aarch64|arm64) cpu="aarch64" ;; armv7l) cpu="armv7" ;; *) die "Unsupported Linux arch: $arch" ;; esac
    libc="gnu"
    if (ldd --version 2>&1 || true) | grep -qi musl; then libc="musl"; fi
    case "$cpu:$libc" in
      x86_64:gnu) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64:gnu) TARGET="aarch64-unknown-linux-gnu" ;;
      x86_64:musl) TARGET="x86_64-unknown-linux-musl" ;;
      aarch64:musl) TARGET="aarch64-unknown-linux-musl" ;;
      armv7:gnu) TARGET="armv7-unknown-linux-gnueabihf" ;;
      *) die "Unsupported Linux target: $cpu/$libc" ;;
    esac
    ;;
  MINGW*|MSYS*|CYGWIN*) die "Windows native installs must use https://install.focusa.dev/focusa.ps1 in PowerShell." ;;
  *) die "Unsupported OS: $(uname -s 2>/dev/null || echo unknown)" ;;
esac

if [ -z "$VERSION" ]; then
  VERSION="$(fetch_text "https://api.github.com/repos/$GITHUB_REPO/releases?per_page=20" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -1)"
fi
[ -n "$VERSION" ] || die "Could not discover Focusa release version from GitHub. Use --version vX.Y.Z."
TAG="$VERSION"
ASSET_BASE="https://github.com/$GITHUB_REPO/releases/download/$TAG"

log "Plan:"
log "  prefix:      $PREFIX"
log "  channel:     $CHANNEL"
log "  version:     $TAG"
log "  target:      $TARGET"
log "  eval:        $EVAL"
if [ -n "$LICENSE_KEY" ]; then log "  license:     provided"; else log "  license:     (none)"; fi
log "  dry-run:     $DRY_RUN"
log "  with-engine: $WITH_ENGINE"
log "  with-pi:     $WITH_PI"
log "  no-service:  $NO_SERVICE"

if [ "$DRY_RUN" -eq 0 ] && [ "$EVAL" -eq 0 ]; then
  log "Validating license key against $LICENSE_REGISTRY ..."
  VALIDATE_RESP="$(post_license_validate 2>/dev/null || true)"
  echo "$VALIDATE_RESP" | grep -q '"valid"[[:space:]]*:[[:space:]]*true' || {
    err "License validation failed."
    err "Response: ${VALIDATE_RESP:-(empty)}"
    err "Purchase/manage license: https://install.focusa.dev/license"
    exit 2
  }
  TIER="$(printf '%s' "$VALIDATE_RESP" | json_get tier)"; TIER="${TIER:-operator}"
  log "License valid: tier=$TIER"
fi

if [ "$DRY_RUN" -eq 1 ]; then
  log "DRY RUN: would create $PREFIX/bin and download focusa/focusa-daemon/focusa-tui from $ASSET_BASE"
else
  tmp="$(mktemp -d "${TMPDIR:-/tmp}/focusa-install.XXXXXX")"
  trap 'rm -rf "$tmp"' EXIT
  mkdir -p "$PREFIX/bin" "$PREFIX/state" "$PREFIX/config" "$PREFIX/libexec" "$HOME/.config/focusa"

  checksums="$tmp/SHA256SUMS.txt"
  if fetch "$ASSET_BASE/SHA256SUMS.txt" "$checksums" >/dev/null 2>&1 || fetch "$ASSET_BASE/SHA256SUMS" "$checksums" >/dev/null 2>&1; then
    log "Downloaded checksum manifest."
    verify_checksum_manifest_signature "$checksums"
  else
    checksums=""
    warn "No SHA256SUMS asset found for $TAG; digest verification is incomplete until release signing lands."
  fi

  for bin in focusa focusa-daemon focusa-tui; do
    asset="$bin-$TAG-$TARGET"
    out="$tmp/$bin"
    log "Downloading $asset"
    fetch "$ASSET_BASE/$asset" "$out" || die "Missing release asset: $asset. recovery_hint: choose a supported platform/version or wait for matching release asset."
    if [ -n "$checksums" ]; then
      expected="$(awk -v a="$asset" '$2==a || $2=="dist/" a {print $1}' "$checksums" | head -1)"
      [ -n "$expected" ] || die "Checksum missing for $asset in SHA256SUMS."
      actual="$(sha256_file "$out")"
      [ "$expected" = "$actual" ] || die "Checksum mismatch for $asset. recovery_hint: re-download from https://install.focusa.dev/help/security"
    fi
    chmod 0755 "$out"
    mv "$out" "$PREFIX/bin/$bin"
  done

  if ! "$PREFIX/bin/focusa" --version >/tmp/focusa-install-version.out 2>&1; then
    if [ "$TARGET" = "x86_64-unknown-linux-gnu" ]; then
      warn "glibc binary failed on this host; trying musl/static fallback."
      sed -n '1,12p' /tmp/focusa-install-version.out >&2 || true
      fallback_target="x86_64-unknown-linux-musl"
      fallback_ok=1
      for bin in focusa focusa-daemon focusa-tui; do
        asset="$bin-$TAG-$fallback_target"
        out="$tmp/$bin"
        log "Downloading fallback $asset"
        if ! fetch "$ASSET_BASE/$asset" "$out"; then
          fallback_ok=0
          break
        fi
        if [ -n "$checksums" ]; then
          expected="$(awk -v a="$asset" '$2==a || $2=="dist/" a {print $1}' "$checksums" | head -1)"
          [ -n "$expected" ] || die "Checksum missing for $asset in SHA256SUMS."
          actual="$(sha256_file "$out")"
          [ "$expected" = "$actual" ] || die "Checksum mismatch for $asset. recovery_hint: re-download from https://install.focusa.dev/help/security"
        fi
        chmod 0755 "$out"
        mv "$out" "$PREFIX/bin/$bin"
      done
      if [ "$fallback_ok" -eq 1 ] && "$PREFIX/bin/focusa" --version >/tmp/focusa-install-version.out 2>&1; then
        TARGET="$fallback_target"
        log "Using fallback target: $TARGET"
      else
        err "Downloaded focusa binary could not execute on this host."
        sed -n '1,20p' /tmp/focusa-install-version.out >&2 || true
        die "Binary compatibility check failed before license write. recovery_hint: use a compatible release asset, musl/static build, or contact support@focusa.dev."
      fi
    else
      err "Downloaded focusa binary could not execute on this host."
      sed -n '1,20p' /tmp/focusa-install-version.out >&2 || true
      die "Binary compatibility check failed before license write. recovery_hint: use a compatible release asset, musl/static build, or contact support@focusa.dev."
    fi
  fi

  key_hash=""; key_prefix=""; product="focusa"; status="active"; commercial_use="false"; features='[]'; expires_at=""; activated_at=""
  if [ "$EVAL" -eq 1 ]; then
    TIER="evaluation"; features='["daemon","tui","cli"]'
  else
    key_hash="$(sha256_string "$LICENSE_KEY")"
    key_prefix="${LICENSE_KEY:0:16}"
    product="$(printf '%s' "$VALIDATE_RESP" | json_get product)"; product="${product:-focusa}"
    status="$(printf '%s' "$VALIDATE_RESP" | json_get status)"; status="${status:-active}"
    commercial_use="$(printf '%s' "$VALIDATE_RESP" | json_get commercial_use)"; commercial_use="${commercial_use:-false}"
    features="$(printf '%s' "$VALIDATE_RESP" | json_get features)"; features="${features:-[]}"
    expires_at="$(printf '%s' "$VALIDATE_RESP" | json_get expires_at)"
    activated_at="$(printf '%s' "$VALIDATE_RESP" | json_get activated_at)"
  fi
  offline_valid_until="$(date -u -d '+7 days' +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u -v+7d +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date -u +%Y-%m-%dT%H:%M:%SZ)"

  if have python3; then
    python3 - "$HOME/.config/focusa/license.json" "$features" <<PY
import json, os, sys
path=sys.argv[1]
features=json.loads(sys.argv[2] or '[]')
data={
  "key_hash": "$key_hash",
  "key_prefix": "$key_prefix",
  "product": "$product",
  "tier": "$TIER",
  "status": "$status",
  "commercial_use": json.loads("$commercial_use"),
  "customer_email": None,
  "features": features,
  "expires_at": "$expires_at" or None,
  "offline_valid_until": "$offline_valid_until",
  "registry_url": "$LICENSE_REGISTRY",
  "activated_at": "$activated_at" or None,
  "eval": bool(int("$EVAL")),
}
tmp=path+".tmp"
os.makedirs(os.path.dirname(path), exist_ok=True)
with open(tmp,"w",encoding="utf-8") as f:
    json.dump(data,f,indent=2)
    f.write("\n")
os.chmod(tmp,0o600)
os.replace(tmp,path)
PY
  else
    cat > "$HOME/.config/focusa/license.json" <<JSON
{"key_hash":"$key_hash","key_prefix":"$key_prefix","product":"$product","tier":"$TIER","status":"$status","commercial_use":$commercial_use,"customer_email":null,"features":$features,"expires_at":null,"offline_valid_until":"$offline_valid_until","registry_url":"$LICENSE_REGISTRY","activated_at":null,"eval":$([ "$EVAL" -eq 1 ] && echo true || echo false)}
JSON
    chmod 600 "$HOME/.config/focusa/license.json"
  fi
  log "Wrote daemon-compatible license state to \$HOME/.config/focusa/license.json"

  mkdir -p "$HOME/.local/bin"
  ln -sf "$PREFIX/bin/focusa" "$HOME/.local/bin/focusa"
  log "Linked $HOME/.local/bin/focusa -> $PREFIX/bin/focusa"

  if [ "$NO_SERVICE" -eq 0 ]; then
    if [ "$(uname -s)" = "Darwin" ] && have launchctl; then
      mkdir -p "$HOME/Library/LaunchAgents"
      plist="$HOME/Library/LaunchAgents/com.startempire.focusa-daemon.plist"
      cat > "$plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>com.startempire.focusa-daemon</string>
  <key>ProgramArguments</key><array><string>$PREFIX/bin/focusa-daemon</string></array>
  <key>RunAtLoad</key><true/><key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>$PREFIX/state/focusa-daemon.out.log</string>
  <key>StandardErrorPath</key><string>$PREFIX/state/focusa-daemon.err.log</string>
</dict></plist>
PLIST
      launchctl unload "$plist" >/dev/null 2>&1 || true
      launchctl load -w "$plist" || warn "LaunchAgent written but not loaded. recovery_hint: launchctl load -w $plist"
    elif have systemctl && systemctl --user status >/dev/null 2>&1; then
      mkdir -p "$HOME/.config/systemd/user"
      unit="$HOME/.config/systemd/user/focusa-daemon.service"
      cat > "$unit" <<UNIT
[Unit]
Description=Focusa Daemon
After=network-online.target

[Service]
ExecStart=$PREFIX/bin/focusa-daemon
Restart=on-failure
RestartSec=3
WorkingDirectory=$PREFIX

[Install]
WantedBy=default.target
UNIT
      systemctl --user daemon-reload || true
      systemctl --user enable --now focusa-daemon.service || warn "systemd user service written but not started. recovery_hint: systemctl --user enable --now focusa-daemon.service"
    else
      warn "No supported service manager detected; run $PREFIX/bin/focusa-daemon manually or rerun with --no-service."
    fi
  fi
fi

if [ "$WITH_ENGINE" -eq 1 ]; then
  log "Installing UIAI Engine companion (delegating to /engine installer)..."
  if [ "$DRY_RUN" -eq 1 ]; then
    log "DRY RUN: curl -fsSL $LICENSE_REGISTRY/engine | bash -s -- --prefix $PREFIX --no-service"
  else
    fetch "$LICENSE_REGISTRY/engine" /tmp/engine-install.sh
    bash /tmp/engine-install.sh --prefix "$PREFIX" --no-service ${EVAL:+--eval}
  fi
fi

if [ "$WITH_PI" -eq 1 ]; then
  log "Installing Pi wrapper..."
  run "curl -fsSL https://raw.githubusercontent.com/mariozechner/pi-coding-agent/main/install.sh -o /tmp/pi-install.sh"
  run "bash /tmp/pi-install.sh"
fi

if [ "$DRY_RUN" -eq 1 ]; then
  log "DRY RUN: '$PREFIX/bin/focusa' --version"
else
  if ! "$PREFIX/bin/focusa" --version >/tmp/focusa-install-version.out 2>&1; then
    err "Downloaded focusa binary could not execute on this host."
    sed -n '1,20p' /tmp/focusa-install-version.out >&2 || true
    die "Binary compatibility check failed. recovery_hint: use a compatible release asset, musl/static build, or contact support@focusa.dev."
  fi
  cat /tmp/focusa-install-version.out
fi
log "Done."

# Auto-discover a phone-reachable URL for QR-first self-host pairing.
# Operator can override with --public-url or by editing ~/.config/focusa/public-url.
PUBLIC_URL_FILE="$HOME/.config/focusa/public-url"
if [ "$DRY_RUN" -eq 1 ]; then
  log "DRY RUN: would run '$PREFIX/bin/focusa pairing transport-setup' to discover phone-reachable URL."
elif [ -n "${FOCUSA_PUBLIC_URL:-}" ]; then
  log "Using FOCUSA_PUBLIC_URL=$FOCUSA_PUBLIC_URL (from environment)."
  mkdir -p "$(dirname "$PUBLIC_URL_FILE")"
  printf '%s\n' "$FOCUSA_PUBLIC_URL" > "$PUBLIC_URL_FILE"
elif [ -s "$PUBLIC_URL_FILE" ]; then
  log "Using existing public URL from $PUBLIC_URL_FILE: $(cat "$PUBLIC_URL_FILE")"
elif have cloudflared && ! [ "${SKIP_CLOUDFLARED:-0}" = "1" ]; then
  log "Discovering phone-reachable URL via cloudflared quick tunnel…"
  if "$PREFIX/bin/focusa" pairing transport-setup --provider cloudflared --write "$PUBLIC_URL_FILE" \
      >>"$PREFIX/state/focusa-install.out.log" 2>>"$PREFIX/state/focusa-install.err.log"; then
    PUBLIC_URL="$(cat "$PUBLIC_URL_FILE" 2>/dev/null || echo)"
    if [ -n "$PUBLIC_URL" ]; then
      log "Public pairing URL written to $PUBLIC_URL_FILE"
      log "Mac app: open Settings, paste this URL into 'Public pairing URL', then rescan."
    fi
  else
    warn "cloudflared transport-setup failed. recovery_hint: run '$PREFIX/bin/focusa pairing transport-setup' or set FOCUSA_PUBLIC_URL."
  fi
else
  log "No public pairing URL configured. The Mac app will use http://127.0.0.1:8787 (requires LAN/Tailscale to phone)."
  log "To make this server phone-reachable: '$PREFIX/bin/focusa pairing transport-setup' (or set FOCUSA_PUBLIC_URL)."
fi

log "Run: $PREFIX/bin/focusa license status"
log "Docs: https://install.focusa.dev"
