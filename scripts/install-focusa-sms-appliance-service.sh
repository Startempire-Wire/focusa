#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
INSTALL_ROOT="${FOCUSA_SMS_INSTALL_ROOT:-/usr/local}"
RUNTIME_USER="${FOCUSA_SMS_RUNTIME_USER:-wirebot}"
RUNTIME_GROUP="${FOCUSA_SMS_RUNTIME_GROUP:-$RUNTIME_USER}"
STATE_DIR="${FOCUSA_SMS_STATE_DIR:-/var/lib/focusa-sms-broker}"
RUNTIME_DIR="${FOCUSA_SMS_RUNTIME_DIR:-/run/focusa-sms-broker}"
ENV_FILE="${FOCUSA_SMS_ENV_FILE:-/etc/focusa/sms-appliance.env}"
POLICY_CREDENTIAL_FILE="${FOCUSA_SMS_POLICY_CREDENTIAL_FILE:-/etc/credstore.encrypted/focusa-sms-provider-policy}"
CHECKPOINT_CREDENTIAL_FILE="${FOCUSA_SMS_CHECKPOINT_CREDENTIAL_FILE:-/etc/credstore.encrypted/focusa-sms-checkpoint-key}"
TOKEN_CREDENTIAL_FILE="${FOCUSA_SMS_TOKEN_CREDENTIAL_FILE:-/etc/credstore.encrypted/focusa-sms-broker-token}"
GRANTS_CREDENTIAL_FILE="${FOCUSA_SMS_GRANTS_CREDENTIAL_FILE:-/etc/credstore.encrypted/focusa-sms-grants}"
TARGETS_CREDENTIAL_FILE="${FOCUSA_SMS_TARGETS_CREDENTIAL_FILE:-/etc/credstore.encrypted/focusa-sms-targets}"
UNIT_PATH="${FOCUSA_SMS_UNIT_PATH:-/etc/systemd/system/focusa-sms-appliance.service}"
MODE="install"
[[ "${1:-}" == "--render" ]] && MODE="render"
[[ "$(id -u)" == 0 || "$MODE" == "render" ]] || { echo "installation requires root" >&2; exit 1; }
command -v zstd >/dev/null || { echo "existing zstd is required" >&2; exit 1; }
command -v node >/dev/null || { echo "existing Node.js is required" >&2; exit 1; }
python3 -c 'import cryptography, websocket' || { echo "existing Python cryptography and websocket modules are required" >&2; exit 1; }
SYSTEMD_CREDS_HELP="$(systemd-creds --help 2>&1)" || { echo "systemd encrypted credentials unavailable" >&2; exit 1; }
[[ "$SYSTEMD_CREDS_HELP" == *encrypt* ]] || { echo "systemd encrypted credentials unsupported" >&2; exit 1; }

for value in "$INSTALL_ROOT" "$STATE_DIR" "$RUNTIME_DIR" "$ENV_FILE" "$CHECKPOINT_CREDENTIAL_FILE" "$TOKEN_CREDENTIAL_FILE" "$GRANTS_CREDENTIAL_FILE" "$TARGETS_CREDENTIAL_FILE" "$POLICY_CREDENTIAL_FILE" "$UNIT_PATH"; do
  [[ "$value" == /* && "$value" != *$'\n'* ]] || { echo "unsafe absolute path" >&2; exit 1; }
done
id "$RUNTIME_USER" >/dev/null 2>&1 || { echo "runtime user unavailable" >&2; exit 1; }
getent group "$RUNTIME_GROUP" >/dev/null 2>&1 || { echo "runtime group unavailable" >&2; exit 1; }
[[ -f "$ENV_FILE" && ! -L "$ENV_FILE" ]] || { echo "owner-managed mode-0600 environment file required" >&2; exit 1; }
[[ "$(stat -c %a "$ENV_FILE")" == "600" && "$(stat -c %u "$ENV_FILE")" == 0 ]] || { echo "environment file must be root-owned mode 0600" >&2; exit 1; }
! grep -q '@[A-Z_][A-Z_]*@' "$ENV_FILE" || { echo "environment file contains unresolved placeholders" >&2; exit 1; }
for credential in "$CHECKPOINT_CREDENTIAL_FILE" "$TOKEN_CREDENTIAL_FILE" "$GRANTS_CREDENTIAL_FILE" "$TARGETS_CREDENTIAL_FILE" "$POLICY_CREDENTIAL_FILE"; do
  [[ -f "$credential" && ! -L "$credential" ]] || { echo "encrypted systemd credential required: $credential" >&2; exit 1; }
  [[ "$(stat -c %u "$credential")" == 0 && "$(stat -c %a "$credential")" =~ ^(400|600)$ ]] || { echo "credential must be root-owned mode 0400/0600: $credential" >&2; exit 1; }
done

tmp="$(mktemp)"
trap 'unlink "$tmp" 2>/dev/null || :' EXIT
sed \
  -e "s|@INSTALL_ROOT@|$INSTALL_ROOT|g" \
  -e "s|@RUNTIME_USER@|$RUNTIME_USER|g" \
  -e "s|@RUNTIME_GROUP@|$RUNTIME_GROUP|g" \
  -e "s|@STATE_DIR@|$STATE_DIR|g" \
  -e "s|@RUNTIME_DIR@|$RUNTIME_DIR|g" \
  -e "s|@ENV_FILE@|$ENV_FILE|g" \
  -e "s|@POLICY_CREDENTIAL_FILE@|$POLICY_CREDENTIAL_FILE|g" \
  -e "s|@CHECKPOINT_CREDENTIAL_FILE@|$CHECKPOINT_CREDENTIAL_FILE|g" \
  -e "s|@TOKEN_CREDENTIAL_FILE@|$TOKEN_CREDENTIAL_FILE|g" \
  -e "s|@GRANTS_CREDENTIAL_FILE@|$GRANTS_CREDENTIAL_FILE|g" \
  -e "s|@TARGETS_CREDENTIAL_FILE@|$TARGETS_CREDENTIAL_FILE|g" \
  "$ROOT/templates/systemd/focusa-sms-appliance.service" >"$tmp"
grep -q 'Restart=always' "$tmp"
grep -q 'WatchdogSec=30s' "$tmp"
grep -q 'LoadCredentialEncrypted=' "$tmp"
grep -q 'UMask=0077' "$tmp"

if [[ "$MODE" == "render" ]]; then
  cat "$tmp"
  exit 0
fi
install -d -m 0755 "$INSTALL_ROOT/libexec/focusa"
install -m 0755 "$ROOT/scripts/focusa-sms-appliance.py" "$INSTALL_ROOT/libexec/focusa/"
install -m 0755 "$ROOT/scripts/focusa-sms-supervisor.py" "$INSTALL_ROOT/libexec/focusa/"
install -m 0755 "$ROOT/scripts/focusa-google-messages-broker.py" "$INSTALL_ROOT/libexec/focusa/"
install -m 0755 "$ROOT/scripts/focusa-sms-ready-probe.mjs" "$INSTALL_ROOT/libexec/focusa/"
install -m 0755 "$ROOT/scripts/provision-focusa-sms-appliance-credentials.py" "$INSTALL_ROOT/libexec/focusa/"
install -m 0644 "$tmp" "$UNIT_PATH"
systemctl daemon-reload
systemctl enable --now focusa-sms-appliance.service
systemctl is-enabled --quiet focusa-sms-appliance.service
systemctl is-active --quiet focusa-sms-appliance.service
