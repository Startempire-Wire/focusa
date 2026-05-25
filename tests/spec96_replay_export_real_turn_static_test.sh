#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
REPLAY="${ROOT_DIR}/crates/focusa-core/src/replay/mod.rs"

if rg -n 'placeholder - would extract' "$REPLAY" >/dev/null; then
  echo "✗ FAIL: replay SFT export still emits placeholder text" >&2
  exit 1
fi

if rg -n 'turn\.raw_user_input|CltPayload::Interaction|role == "assistant"|metadata.*turn_id' "$REPLAY" >/dev/null; then
  echo "✓ PASS: replay SFT export derives instruction/response from active turn and CLT assistant content"
else
  echo "✗ FAIL: replay SFT export lacks real turn/CLT extraction" >&2
  exit 1
fi

if rg -n 'test_export_sft_uses_real_turn_and_clt_content' "$REPLAY" >/dev/null; then
  echo "✓ PASS: replay SFT real-content regression test exists"
else
  echo "✗ FAIL: replay SFT real-content regression test missing" >&2
  exit 1
fi

echo "SPEC96 Replay export real-turn static test: PASS"
