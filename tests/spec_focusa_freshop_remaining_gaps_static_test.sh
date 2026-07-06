#!/usr/bin/env bash
# Static guard for fresh-operator dry-run remaining gaps 4-9.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
fail() { echo "✗ FAIL: $*" >&2; exit 1; }
pass() { echo "✓ PASS: $*"; }

# Gap 4: README quickstart
grep -nF -- "Quickstart (60 seconds)" "$ROOT_DIR/README.md" >/dev/null || fail "README quickstart block missing"
grep -nF -- "focusa init --quickstart" "$ROOT_DIR/README.md" >/dev/null || fail "README quickstart must reference focusa init --quickstart"
pass "gap #4 README quickstart present"

# Gap 5: focusa init --quickstart + ensure_dir_all in onboard
[[ -f "$ROOT_DIR/crates/focusa-cli/src/commands/init.rs" ]] || fail "focusa init command missing"
grep -nF -- "pub quickstart: bool" "$ROOT_DIR/crates/focusa-cli/src/commands/init.rs" >/dev/null || fail "focusa init missing --quickstart flag"
grep -nF -- "create_dir_all" "$ROOT_DIR/crates/focusa-cli/src/commands/init.rs" >/dev/null || fail "focusa init missing create_dir_all"
grep -nF -- "create_dir_all" "$ROOT_DIR/crates/focusa-cli/src/commands/onboard.rs" >/dev/null || fail "onboard missing create_dir_all for fresh --project-root"
grep -nF -- "Init(commands::init::InitArgs)" "$ROOT_DIR/crates/focusa-cli/src/main.rs" >/dev/null || fail "focusa init not wired into Commands enum"
pass "gap #5 focusa init --quickstart and onboard ensure_dir_all wired"

# Gap 6: /v1/about
grep -nF -- "/v1/about" "$ROOT_DIR/crates/focusa-api/src/routes/health.rs" >/dev/null || fail "/v1/about route missing in health.rs"
grep -nF -- "fn about" "$ROOT_DIR/crates/focusa-api/src/routes/health.rs" >/dev/null || fail "about handler missing"
pass "gap #6 /v1/about endpoint present"

# Gap 7+8: ASCII intros and interactive prompts
[[ -f "$ROOT_DIR/crates/focusa-cli/src/commands/intro.rs" ]] || fail "intro.rs missing"
grep -nF -- "FOCUSA_WORDMARK" "$ROOT_DIR/crates/focusa-cli/src/commands/intro.rs" >/dev/null || fail "intro.rs missing wordmark"
grep -nF -- "render_wordmark" "$ROOT_DIR/crates/focusa-cli/src/commands/intro.rs" >/dev/null || fail "intro.rs missing render_wordmark"
grep -nF -- "render_help_banner" "$ROOT_DIR/crates/focusa-cli/src/commands/intro.rs" >/dev/null || fail "intro.rs missing render_help_banner"
grep -nF -- "render_about_banner" "$ROOT_DIR/crates/focusa-cli/src/commands/intro.rs" >/dev/null || fail "intro.rs missing render_about_banner"
grep -nF -- "render_onboard_banner" "$ROOT_DIR/crates/focusa-cli/src/commands/intro.rs" >/dev/null || fail "intro.rs missing render_onboard_banner"
grep -nF -- "detect_prompt_intent" "$ROOT_DIR/crates/focusa-cli/src/commands/intro.rs" >/dev/null || fail "intro.rs missing detect_prompt_intent"
grep -nF -- "pick_scope_intent" "$ROOT_DIR/crates/focusa-cli/src/commands/intro.rs" >/dev/null || fail "intro.rs missing pick_scope_intent"
grep -nF -- "render_about_banner" "$ROOT_DIR/crates/focusa-cli/src/commands/about.rs" >/dev/null || fail "focusa about must render about_banner"
grep -nF -- "render_help_banner" "$ROOT_DIR/crates/focusa-cli/src/main.rs" >/dev/null || fail "focusa --help must render help banner"
grep -nF -- "render_onboard_banner" "$ROOT_DIR/crates/focusa-cli/src/commands/onboard.rs" >/dev/null || fail "focusa onboard must render onboard banner"
pass "gap #7+#8 ASCII intros and interactive prompts wired"

# Gap 9: binary drift guard
[[ -x "$ROOT_DIR/scripts/check-fresh-binary.sh" ]] || fail "binary drift guard script missing or not executable"
grep -nF -- "scope" "$ROOT_DIR/scripts/check-fresh-binary.sh" >/dev/null || fail "binary drift guard does not check --scope marker"
grep -nF -- "tui" "$ROOT_DIR/scripts/check-fresh-binary.sh" || grep -nF -- "Tui" "$ROOT_DIR/scripts/check-fresh-binary.sh" >/dev/null || fail "binary drift guard does not check tui marker"
pass "gap #9 binary drift guard installed"

echo "focusa fresh-operator remaining gaps 4-9 test: PASS"