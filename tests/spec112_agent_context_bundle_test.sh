#!/usr/bin/env bash
# Spec 112 / focusa-wo3q — AGENTS.md + skills release/install contract.
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
source "$ROOT/tests/focusa_portable_bin.sh"
FIXTURE="$(mktemp -d)"
trap 'rm -rf "$FIXTURE"' EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }

WORKFLOW="$ROOT/.github/workflows/release.yml"
INSTALL="$ROOT/crates/focusa-cli/src/commands/install.rs"
for marker in \
  'focusa-agent-context-${TAG}.tar.gz' \
  'cp AGENTS.md bundle/focusa-agent-context/AGENTS.md' \
  'cp -R .pi/skills/. bundle/focusa-agent-context/skills/' \
  'focusa-agent-context-*.tar.gz'; do
  grep -qF "$marker" "$WORKFLOW" || fail "release workflow missing: $marker"
done
for marker in \
  'phase_agent_context_download' \
  'install_agent_context_archive' \
  'unsafe agent context archive path' \
  'refusing unverified install' \
  'focusa-agent-context'; do
  grep -qF "$marker" "$INSTALL" || fail "Rust installer missing: $marker"
done

mkdir -p "$FIXTURE/bundle/focusa-agent-context/skills"
cp AGENTS.md "$FIXTURE/bundle/focusa-agent-context/AGENTS.md"
cp -R .pi/skills/. "$FIXTURE/bundle/focusa-agent-context/skills/"
tar -C "$FIXTURE/bundle" -czf "$FIXTURE/focusa-agent-context-vtest.tar.gz" focusa-agent-context
LISTING="$(tar -tzf "$FIXTURE/focusa-agent-context-vtest.tar.gz")"
printf '%s\n' "$LISTING" | grep -q '^focusa-agent-context/AGENTS.md$' \
  || fail "archive missing AGENTS.md"
printf '%s\n' "$LISTING" | grep -q '^focusa-agent-context/skills/.*/SKILL.md$' \
  || fail "archive missing skills/*/SKILL.md"
if printf '%s\n' "$LISTING" | grep -Eq '^/|(^|/)\.\.(/|$)'; then
  fail "archive contains unsafe paths"
fi

cargo build -q -p focusa-cli --bin focusa
BIN="$(focusa_resolve_test_cli_binary "$ROOT")"
HOME="$FIXTURE/home" XDG_CONFIG_HOME="$FIXTURE/config" XDG_DATA_HOME="$FIXTURE/data" \
  "$BIN" install --target=linux --dry-run --json > "$FIXTURE/plan.json"
jq -e '
  (.assets_planned|any(.name=="focusa-agent-context" and .triple=="all")) and
  (.first_install_walkthrough_v1.agent_integrations|any(
    .agent=="focusa-agent-context" and
    (.config_path|endswith("/.focusa/agent-context")) and
    .integrated==false
  ))
' "$FIXTURE/plan.json" >/dev/null \
  || { cat "$FIXTURE/plan.json" >&2; fail "dry-run omits agent context plan/integration"; }

echo "PASS: release and Rust install surfaces bundle verified AGENTS.md plus skills for first session"
