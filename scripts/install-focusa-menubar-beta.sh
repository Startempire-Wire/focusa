#!/bin/bash
# Focusa pre-license macOS beta bootstrap.
# Durable boundary: paid Apple notarization is production-only. This installer
# uses official GitHub HTTPS bootstrap trust, ad-hoc code integrity, explicit
# consent, rollback, and Tauri signatures for every subsequent OTA update.
set -euo pipefail

REPOSITORY="${FOCUSA_GITHUB_REPOSITORY:-Startempire-Wire/focusa}"
MANIFEST_URL="${FOCUSA_UPDATE_MANIFEST_URL:-https://github.com/${REPOSITORY}/releases/latest/download/latest.json}"
DESTINATION="${FOCUSA_MENUBAR_DESTINATION:-$HOME/Applications/Focusa.app}"
BACKUP="${DESTINATION%.app}.previous.app"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/focusa-menubar-beta.XXXXXX")"
NEW_APP=""
restore_required=0

cleanup() {
  rm -rf "$TMP"
}
rollback() {
  if [[ "$restore_required" == "1" && -d "$BACKUP" ]]; then
    rm -rf "$DESTINATION"
    mv "$BACKUP" "$DESTINATION"
    open "$DESTINATION" >/dev/null 2>&1 || true
    echo "Focusa beta launch failed; previous app restored." >&2
  fi
  cleanup
}
trap rollback ERR INT TERM
trap cleanup EXIT

case "$(uname -m)" in
  arm64) platform="darwin-aarch64" ;;
  x86_64) platform="darwin-x86_64" ;;
  *) echo "Unsupported Mac architecture: $(uname -m)" >&2; exit 1 ;;
esac

cat >&2 <<'NOTICE'
Focusa pre-license macOS beta

This build is Tauri-updater signed but is NOT Apple-notarized. macOS trust starts
with the official Focusa GitHub release and your explicit consent. The installer
will verify bundle integrity and identity, remove quarantine, retain the previous
app, and restore it if the new app cannot launch.
NOTICE

if [[ "${FOCUSA_BETA_ACCEPT:-0}" != "1" ]]; then
  [[ -r /dev/tty ]] || {
    echo "Interactive consent required; set FOCUSA_BETA_ACCEPT=1 only after reviewing this warning." >&2
    exit 1
  }
  printf 'Type BETA to continue: ' >/dev/tty
  IFS= read -r answer </dev/tty
  [[ "$answer" == "BETA" ]] || { echo "Installation cancelled." >&2; exit 1; }
fi

curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
  "$MANIFEST_URL" -o "$TMP/latest.json"
archive_url="$(/usr/bin/plutil -extract "platforms.${platform}.url" raw -o - "$TMP/latest.json" 2>/dev/null || true)"
[[ "$archive_url" == https://github.com/${REPOSITORY}/releases/download/* ]] || {
  echo "Update manifest returned an untrusted archive URL." >&2
  exit 1
}
curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
  "$archive_url" -o "$TMP/Focusa.app.tar.gz"
mkdir -p "$TMP/extracted"
tar -xzf "$TMP/Focusa.app.tar.gz" -C "$TMP/extracted"
NEW_APP="$(find "$TMP/extracted" -maxdepth 3 -type d -name 'Focusa.app' -print -quit)"
[[ -n "$NEW_APP" && -d "$NEW_APP" ]] || { echo "Release archive does not contain Focusa.app." >&2; exit 1; }

codesign --verify --deep --strict --verbose=2 "$NEW_APP"
identifier="$(codesign -d --verbose=4 "$NEW_APP" 2>&1 | awk -F= '/^Identifier=/{print $2; exit}')"
[[ "$identifier" == "com.focusa.menubar" ]] || {
  echo "Unexpected bundle identifier: ${identifier:-missing}" >&2
  exit 1
}

mkdir -p "$(dirname "$DESTINATION")"
rm -rf "$BACKUP"
if [[ -d "$DESTINATION" ]]; then
  osascript -e 'tell application id "com.focusa.menubar" to quit' >/dev/null 2>&1 || true
  mv "$DESTINATION" "$BACKUP"
  restore_required=1
fi
/usr/bin/ditto "$NEW_APP" "$DESTINATION"
# Quarantine removal occurs only after explicit beta consent and verification.
xattr -dr com.apple.quarantine "$DESTINATION"
codesign --verify --deep --strict --verbose=2 "$DESTINATION"

executable="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleExecutable' "$DESTINATION/Contents/Info.plist")"
open "$DESTINATION"
launched=0
for _ in $(seq 1 30); do
  if pgrep -f "$DESTINATION/Contents/MacOS/$executable" >/dev/null 2>&1; then
    launched=1
    break
  fi
  sleep 0.5
done
[[ "$launched" == "1" ]] || { echo "Installed beta did not launch." >&2; false; }

restore_required=0
echo "Focusa pre-license beta installed at $DESTINATION"
echo "Future OTA artifacts are authenticated by the pinned Tauri updater key."
