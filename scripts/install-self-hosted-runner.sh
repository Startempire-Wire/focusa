#!/usr/bin/env bash
# Install or refresh a self-hosted GitHub Actions runner for Focusa live deploys.
set -euo pipefail

REPO="${FOCUSA_RUNNER_REPO:-Startempire-Wire/focusa}"
RUNNER_USER="${FOCUSA_RUNNER_USER:-github-runner}"
RUNNER_GROUP="${FOCUSA_RUNNER_GROUP:-$RUNNER_USER}"
RUNNER_ROOT="${FOCUSA_RUNNER_ROOT:-/opt/actions-runner-focusa}"
RUNNER_NAME="${FOCUSA_RUNNER_NAME:-$(hostname -s)-focusa-deploy}"
RUNNER_LABELS="${FOCUSA_RUNNER_LABELS:-focusa-deploy,production}"
WORK_FOLDER="${FOCUSA_RUNNER_WORK_FOLDER:-_work}"
RUNNER_VERSION="${FOCUSA_RUNNER_VERSION:-}"
SUDOERS_PATH="/etc/sudoers.d/focusa-github-runner"
DRY_RUN=0

log() { printf '[focusa-runner] %s\n' "$*"; }
die() { printf '[focusa-runner][error] %s\n' "$*" >&2; exit 1; }
usage() {
  cat <<'USAGE'
Usage: scripts/install-self-hosted-runner.sh [--dry-run]

Installs a dedicated self-hosted runner with labels:
  self-hosted, linux, x64, focusa-deploy, production

Requires:
- gh CLI authenticated with repo admin/workflow scope
- root privileges
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1; shift ;;
    --help|-h) usage; exit 0 ;;
    *) die "Unknown argument: $1" ;;
  esac
done

[[ $EUID -eq 0 ]] || die "run as root"
command -v gh >/dev/null 2>&1 || die "gh CLI required"
command -v curl >/dev/null 2>&1 || die "curl required"
command -v tar >/dev/null 2>&1 || die "tar required"

if id "$RUNNER_USER" >/dev/null 2>&1; then
  log "runner user exists: $RUNNER_USER"
else
  log "creating runner user: $RUNNER_USER"
  [[ "$DRY_RUN" -eq 1 ]] || useradd --system --create-home --home-dir "/home/$RUNNER_USER" --shell /bin/bash "$RUNNER_USER"
fi

if [[ -z "$RUNNER_VERSION" ]]; then
  RUNNER_VERSION="$(gh api repos/actions/runner/releases/latest --jq '.tag_name' | sed 's/^v//')"
fi
[[ -n "$RUNNER_VERSION" ]] || die "could not resolve runner version"
ARCHIVE="actions-runner-linux-x64-${RUNNER_VERSION}.tar.gz"
URL="https://github.com/actions/runner/releases/download/v${RUNNER_VERSION}/${ARCHIVE}"

log "repo=$REPO"
log "runner_root=$RUNNER_ROOT"
log "runner_name=$RUNNER_NAME"
log "runner_labels=$RUNNER_LABELS"
log "runner_version=$RUNNER_VERSION"

if [[ "$DRY_RUN" -eq 1 ]]; then
  exit 0
fi

mkdir -p "$RUNNER_ROOT"
chown -R "$RUNNER_USER:$RUNNER_GROUP" "$RUNNER_ROOT"

if [[ -f "$RUNNER_ROOT/.runner" ]]; then
  log "existing runner config detected; keeping root and refreshing service/files"
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
log "downloading $URL"
curl -fsSL "$URL" -o "$TMP_DIR/$ARCHIVE"
tar -xzf "$TMP_DIR/$ARCHIVE" -C "$RUNNER_ROOT"
chown -R "$RUNNER_USER:$RUNNER_GROUP" "$RUNNER_ROOT"

REG_TOKEN="$(gh api -X POST repos/$REPO/actions/runners/registration-token --jq '.token')"
[[ -n "$REG_TOKEN" ]] || die "failed to obtain registration token"

sudo -u "$RUNNER_USER" bash -lc "cd '$RUNNER_ROOT' && ./config.sh --url 'https://github.com/$REPO' --token '$REG_TOKEN' --name '$RUNNER_NAME' --labels '$RUNNER_LABELS' --work '$WORK_FOLDER' --replace --unattended"

cat > "$SUDOERS_PATH" <<EOF
Defaults:${RUNNER_USER} !requiretty
${RUNNER_USER} ALL=(root) NOPASSWD: /bin/bash ${RUNNER_ROOT}/_work/focusa/focusa/scripts/install-daemon.sh *, /bin/bash ${RUNNER_ROOT}/_work/focusa/focusa/scripts/safe-disk-cleanup.sh *
EOF
chmod 440 "$SUDOERS_PATH"

cd "$RUNNER_ROOT"
./svc.sh install "$RUNNER_USER"
./svc.sh start

systemctl is-active --quiet "actions.runner.${REPO//\//-}.${RUNNER_NAME}.service" || true
log "runner install complete"
