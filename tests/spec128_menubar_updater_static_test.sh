#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CONF="$ROOT/apps/menubar/src-tauri/tauri.conf.json"
CARGO="$ROOT/apps/menubar/src-tauri/Cargo.toml"
RUST="$ROOT/apps/menubar/src-tauri/src/main.rs"
CAP="$ROOT/apps/menubar/src-tauri/capabilities/default.json"
PKG="$ROOT/apps/menubar/package.json"
UPDATER="$ROOT/apps/menubar/src/lib/updater.ts"
LAYOUT="$ROOT/apps/menubar/src/routes/+layout.svelte"
SETTINGS="$ROOT/apps/menubar/src/lib/components/Settings.svelte"
RELEASE="$ROOT/.github/workflows/release.yml"
CI="$ROOT/.github/workflows/ci.yml"
SIGNING_PROOF="$ROOT/.github/workflows/tauri-updater-signing-proof.yml"
BETA_INSTALLER="$ROOT/scripts/install-focusa-menubar-beta.sh"

fail() { echo "FAIL: $*" >&2; exit 1; }

jq -e '.bundle.createUpdaterArtifacts == true' "$CONF" >/dev/null || fail "Tauri updater artifacts disabled"
jq -e '.plugins.updater.pubkey | type == "string" and length > 80' "$CONF" >/dev/null || fail "Tauri updater public key missing"
jq -e '.plugins.updater.endpoints | any(startswith("https://") and endswith("latest.json"))' "$CONF" >/dev/null || fail "HTTPS updater endpoint missing"
jq -e '.permissions | index("updater:default") and index("process:allow-restart")' "$CAP" >/dev/null || fail "updater/relaunch capabilities missing"
jq -e '.dependencies["@tauri-apps/plugin-updater"] and .dependencies["@tauri-apps/plugin-process"]' "$PKG" >/dev/null || fail "frontend updater dependencies missing"
rg -q 'tauri-plugin-updater' "$CARGO" || fail "Rust updater dependency missing"
rg -q 'tauri-plugin-process' "$CARGO" || fail "Rust process dependency missing"
rg -q 'tauri_plugin_updater::Builder::new\(\)\.build\(\)' "$RUST" || fail "Rust updater plugin not registered"
rg -q 'tauri_plugin_process::init\(\)' "$RUST" || fail "Rust process plugin not registered"
rg -q 'auto_apply_allowed|parts\?\.menubar|downloadAndInstall|await relaunch\(\)' "$UPDATER" || fail "policy-gated download/install/relaunch flow missing"
rg -q 'Signed update failed safely' "$UPDATER" || fail "safe signed-update failure result missing"
rg -q 'startAutomaticMenubarUpdate' "$LAYOUT" || fail "automatic startup updater missing"
rg -q 'Check for update|Install and relaunch|SIGNED UPDATES' "$SETTINGS" || fail "manual settings control missing"
rg -q 'secrets\.TAURI_SIGNING_PRIVATE_KEY' "$RELEASE" || fail "release private-key secret wiring missing"
rg -q 'secrets\.TAURI_SIGNING_PRIVATE_KEY_PASSWORD' "$RELEASE" || fail "release key-password secret wiring missing"
rg -q 'uploadUpdaterJson: true' "$RELEASE" || fail "release updater JSON upload missing"
rg -q 'APPLE_CERTIFICATE_BASE64 APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY APPLE_API_KEY_ID APPLE_API_ISSUER_ID APPLE_API_KEY_P8 APPLE_TEAM_ID' "$RELEASE" || fail "mandatory Apple signing/notarization preflight missing"
if rg -q 'UNNOTARIZED-PREVIEW' "$RELEASE"; then fail "release still allows unnotarized updater artifacts"; fi
rg -q 'tauri signer sign' "$SIGNING_PROOF" || fail "secret-backed updater signing proof missing"
rg -q 'Ed25519PublicKey.*verify' "$SIGNING_PROOF" || fail "updater Ed25519 verification missing"
rg -q 'public_raw\[2:10\].*signature_raw\[2:10\]' "$SIGNING_PROOF" || fail "updater key-ID verification missing"
rg -q 'createUpdaterArtifacts.*false' "$CI" || fail "non-release CI must not require private updater signing material"

if rg -n 'BEGIN (OPENSSH |RSA |EC )?PRIVATE KEY|untrusted comment: encrypted secret key' "$ROOT/apps" "$ROOT/.github" --glob '!package-lock.json'; then
  fail "private signing material found in tracked surfaces"
fi

test -x "$BETA_INSTALLER" || fail "pre-license beta installer missing or not executable"
bash -n "$BETA_INSTALLER" || fail "pre-license beta installer syntax invalid"
for marker in \
  'FOCUSA_MACOS_RELEASE_MODE' \
  'beta_ad_hoc' \
  'production_notarized' \
  'Signature=adhoc' \
  'install-focusa-menubar-beta.sh'; do
  rg -q "$marker" "$RELEASE" || fail "release workflow missing pre-license marker: $marker"
done
for marker in \
  'issues?state=open&per_page=100' \
  'has("pull_request") | not' \
  'startswith("release-gate:")'; do
  rg -Fq "$marker" "$RELEASE" || fail "release workflow missing generic open-issue gate contract: $marker"
done
for marker in \
  'pre-license macOS beta' \
  'FOCUSA_BETA_ACCEPT' \
  'com.focusa.menubar' \
  'codesign --verify --deep --strict' \
  'com.apple.quarantine' \
  'previous.app' \
  'Tauri updater key'; do
  rg -q "$marker" "$BETA_INSTALLER" || fail "beta installer missing trust/rollback marker: $marker"
done
rg -q 'Signature=adhoc' "$CI" || fail "ordinary macOS CI does not prove ad-hoc bundle integrity"
rg -q 'MENUBAR_RELEASE_MODE' "$UPDATER" "$SETTINGS" || fail "menubar UI does not disclose pre-license beta mode"
rg -q 'not Apple-notarized' "$SETTINGS" || fail "menubar settings omit beta notarization warning"
if awk '/^  tauri-build:/{in_job=1} /^  rust-release:/{in_job=0} in_job{print}' "$RELEASE" | rg -q 'continue-on-error: true'; then
  fail "mandatory menubar release job remains optional"
fi

echo "PASS: Spec128 signed menubar automatic updater, pre-license beta bootstrap, and production notarization boundary present"
