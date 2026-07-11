#!/usr/bin/env bash
# Spec 112 / focusa-ux2qx.14 — production-function archive smoke test.
# Exercises the exact install_pi_extension() function from install-focusa.sh.

set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
FIXTURE="$(mktemp -d)"
trap 'rm -rf "$FIXTURE"' EXIT

mkdir -p "$FIXTURE/release/apps" "$FIXTURE/fake-bin"
git archive --format=tar HEAD apps/pi-extension | tar -xf - -C "$FIXTURE/release"
tar -C "$FIXTURE/release/apps" -czf "$FIXTURE/focusa-pi-extension-vtest.tar.gz" pi-extension
ARCHIVE="$FIXTURE/focusa-pi-extension-vtest.tar.gz"
ARCHIVE_HASH="$(sha256sum "$ARCHIVE" | awk '{print $1}')"
printf '%s  %s\n' "$ARCHIVE_HASH" "$(basename "$ARCHIVE")" > "$FIXTURE/SHA256SUMS"

cat > "$FIXTURE/fake-bin/pi" <<'SH'
#!/usr/bin/env bash
exit 0
SH
cat > "$FIXTURE/fake-bin/npm" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == *"install --omit=dev --ignore-scripts"* ]]
mkdir -p node_modules
printf 'staged\n' > node_modules/.focusa-smoke
SH
cat > "$FIXTURE/fake-bin/curl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
: "${TEST_ARCHIVE:?}"
: "${TEST_CURL_CALLED:?}"
printf 'called\n' >> "$TEST_CURL_CALLED"
out=""
while [[ $# -gt 0 ]]; do
    if [[ "$1" == "-o" ]]; then
        out="$2"
        shift 2
    else
        shift
    fi
done
[[ -n "$out" ]]
cp "$TEST_ARCHIVE" "$out"
SH
chmod +x "$FIXTURE/fake-bin/pi" "$FIXTURE/fake-bin/npm" "$FIXTURE/fake-bin/curl"

FUNCTION_FILE="$FIXTURE/install-pi-extension-function.sh"
sed -n '/# BEGIN PI_EXTENSION_INSTALL_FUNCTION/,/# END PI_EXTENSION_INSTALL_FUNCTION/p' \
    scripts/install-focusa.sh > "$FUNCTION_FILE"
# shellcheck source=/dev/null
source "$FUNCTION_FILE"

log() { printf 'LOG: %s\n' "$*"; }
warn() { printf 'WARN: %s\n' "$*" >&2; }
export PATH="$FIXTURE/fake-bin:$PATH"
export TEST_ARCHIVE="$ARCHIVE"
export TEST_CURL_CALLED="$FIXTURE/curl-called"
HOME="$FIXTURE/home"
GITHUB_REPO="Startempire-Wire/focusa"
RELEASE_TAG="vtest"

# Case 1: verified archive stages dependencies and atomically replaces prior install.
TMP="$FIXTURE/case-install/tmp"
FOCUSA_PI_EXT_DIR="$FIXTURE/case-install/extensions"
CHECKSUM_MANIFEST="$FIXTURE/SHA256SUMS"
DRY_RUN=0
mkdir -p "$TMP" "$FOCUSA_PI_EXT_DIR/focusa"
printf 'prior\n' > "$FOCUSA_PI_EXT_DIR/focusa/prior-marker"
install_pi_extension
[[ -f "$FOCUSA_PI_EXT_DIR/focusa/package.json" ]]
[[ -f "$FOCUSA_PI_EXT_DIR/focusa/node_modules/.focusa-smoke" ]]
[[ ! -e "$FOCUSA_PI_EXT_DIR/focusa/prior-marker" ]]

# Case 2: dry-run performs no download and leaves the destination unchanged.
: > "$TEST_CURL_CALLED"
TMP="$FIXTURE/case-dry/tmp"
FOCUSA_PI_EXT_DIR="$FIXTURE/case-dry/extensions"
DRY_RUN=1
mkdir -p "$TMP" "$FOCUSA_PI_EXT_DIR/focusa"
printf 'prior\n' > "$FOCUSA_PI_EXT_DIR/focusa/prior-marker"
install_pi_extension
[[ -f "$FOCUSA_PI_EXT_DIR/focusa/prior-marker" ]]
[[ ! -s "$TEST_CURL_CALLED" ]]

# Case 3: checksum mismatch rejects the archive and preserves prior install.
TMP="$FIXTURE/case-mismatch/tmp"
FOCUSA_PI_EXT_DIR="$FIXTURE/case-mismatch/extensions"
CHECKSUM_MANIFEST="$FIXTURE/BAD-SHA256SUMS"
DRY_RUN=0
mkdir -p "$TMP" "$FOCUSA_PI_EXT_DIR/focusa"
printf '%064d  %s\n' 0 "$(basename "$ARCHIVE")" > "$CHECKSUM_MANIFEST"
printf 'prior\n' > "$FOCUSA_PI_EXT_DIR/focusa/prior-marker"
install_pi_extension
[[ -f "$FOCUSA_PI_EXT_DIR/focusa/prior-marker" ]]
[[ ! -e "$FOCUSA_PI_EXT_DIR/focusa/package.json" ]]

echo "PASS: verified Pi extension archive install is dry-run-safe, checksum-gated, and atomic"
