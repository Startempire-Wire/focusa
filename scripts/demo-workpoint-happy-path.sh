#!/usr/bin/env bash
set -euo pipefail

# Focusa Operator Preview golden demo:
# onboarding -> project identity -> trajectory -> Workpoint -> evidence -> resume -> drift check -> proof packet.

ROOT="${FOCUSA_PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
OUT_DIR="${FOCUSA_DEMO_OUT_DIR:-$(mktemp -d "${TMPDIR:-/tmp}/focusa-demo-workpoint.XXXXXX")}" 
CONTINUITY_ID="${FOCUSA_DEMO_CONTINUITY_ID:-focusa-demo-$(date +%Y%m%d%H%M%S)}"
TAG="${FOCUSA_DEMO_TAG:-operator-preview-demo}"

focusa_cmd() {
  if [ -n "${FOCUSA_BIN:-}" ]; then
    "${FOCUSA_BIN}" "$@"
  elif command -v focusa >/dev/null 2>&1; then
    focusa "$@"
  elif [ -x "$ROOT/target/release/focusa" ]; then
    "$ROOT/target/release/focusa" "$@"
  elif [ -x "$ROOT/target/debug/focusa" ]; then
    "$ROOT/target/debug/focusa" "$@"
  else
    (cd "$ROOT" && cargo run -q -p focusa-cli --bin focusa -- "$@")
  fi
}

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 127
  }
}

need jq
mkdir -p "$OUT_DIR"

echo "Focusa Operator Preview demo"
echo "  root: $ROOT"
echo "  continuity: $CONTINUITY_ID"
echo "  out: $OUT_DIR"

focusa_cmd start >/dev/null || true
focusa_cmd status --json > "$OUT_DIR/01-status.json"
focusa_cmd project identity --project-root "$ROOT" --json > "$OUT_DIR/02-project-identity.json"

focusa_cmd trajectory define-goal \
  --long-term-goal "Prove Focusa Operator Preview happy path" \
  --desired-end-state "A tester can onboard, checkpoint a Workpoint, link evidence, resume after compaction, and see drift guidance" \
  --short-term-goal "Run the golden Workpoint demo" \
  --current-state "Demo script started" \
  --goal-source operator \
  --operator-confirmed \
  --project-root "$ROOT" \
  --continuity-id "$CONTINUITY_ID" \
  --json > "$OUT_DIR/03-trajectory-define.json"

focusa_cmd workpoint checkpoint \
  --mission "Operator Preview demo: preserve the AI coding session thread" \
  --next-action "Attach evidence, simulate compaction, then resume the Workpoint" \
  --work-item "$TAG" \
  --project-root "$ROOT" \
  --continuity-id "$CONTINUITY_ID" \
  --reason session-start \
  --action-type operator_preview_demo \
  --target-ref scripts/demo-workpoint-happy-path.sh \
  --json > "$OUT_DIR/04-workpoint-checkpoint.json"

WORKPOINT_ID="$(jq -r '.workpoint_id // .workpoint.workpoint_id // .workpoint.id // empty' "$OUT_DIR/04-workpoint-checkpoint.json")"

EVIDENCE_ARGS=(workpoint evidence-link)
if [ -n "$WORKPOINT_ID" ]; then
  EVIDENCE_ARGS+=(--workpoint-id "$WORKPOINT_ID")
fi
EVIDENCE_ARGS+=(
  --target-ref scripts/demo-workpoint-happy-path.sh
  --result "Golden Operator Preview demo reached evidence-link step"
  --evidence-ref "$OUT_DIR/04-workpoint-checkpoint.json"
  --json
)
focusa_cmd "${EVIDENCE_ARGS[@]}" > "$OUT_DIR/05-evidence-link.json"

# This is the compaction/session-switch moment: discard transcript assumptions and ask Focusa for the typed continuation packet.
focusa_cmd workpoint resume \
  --mode compact_prompt \
  --project-root "$ROOT" \
  --continuity-id "$CONTINUITY_ID" \
  --json > "$OUT_DIR/06-workpoint-resume.json"

focusa_cmd workpoint drift-check \
  --latest-action "Attached evidence and resumed the Operator Preview demo Workpoint" \
  --expected-action-type operator_preview_demo \
  --json > "$OUT_DIR/07-drift-check.json"

focusa_cmd trajectory view \
  --project-root "$ROOT" \
  --continuity-id "$CONTINUITY_ID" \
  --mode summary \
  --json > "$OUT_DIR/08-trajectory-view.json"

jq -n \
  --arg generated_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg project_root "$ROOT" \
  --arg continuity_id "$CONTINUITY_ID" \
  --arg workpoint_id "$WORKPOINT_ID" \
  --arg out_dir "$OUT_DIR" \
  '{
    status: "completed",
    summary: "Focusa Operator Preview golden Workpoint demo completed",
    generated_at: $generated_at,
    project_root: $project_root,
    continuity_id: $continuity_id,
    workpoint_id: $workpoint_id,
    artifacts: {
      status: "01-status.json",
      project_identity: "02-project-identity.json",
      trajectory_define: "03-trajectory-define.json",
      workpoint_checkpoint: "04-workpoint-checkpoint.json",
      evidence_link: "05-evidence-link.json",
      workpoint_resume: "06-workpoint-resume.json",
      drift_check: "07-drift-check.json",
      trajectory_view: "08-trajectory-view.json"
    },
    out_dir: $out_dir,
    next_command: "focusa workpoint resume --mode compact_prompt --project-root <root> --continuity-id <id>"
  }' > "$OUT_DIR/proof-packet.json"

echo "Demo proof packet: $OUT_DIR/proof-packet.json"
jq '{status,summary,project_root,continuity_id,workpoint_id,next_command}' "$OUT_DIR/proof-packet.json"
