#!/bin/bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
CLI_MAIN="${ROOT_DIR}/crates/focusa-cli/src/main.rs"
CLI_MOD="${ROOT_DIR}/crates/focusa-cli/src/commands/mod.rs"
PROJECT="${ROOT_DIR}/crates/focusa-cli/src/commands/project.rs"
TRAJECTORY="${ROOT_DIR}/crates/focusa-cli/src/commands/trajectory.rs"
TRAVERSE="${ROOT_DIR}/crates/focusa-cli/src/commands/traverse.rs"
RESOURCE="${ROOT_DIR}/crates/focusa-cli/src/commands/resource.rs"
CLI_DOC="${ROOT_DIR}/docs/current/CLI_REFERENCE_CURRENT.md"

if rg -n 'Project\(commands::project::ProjectCmd\)|Resource\(commands::resource::ResourceCmd\)|Trajectory\(commands::trajectory::TrajectoryCmd\)|Traverse\(commands::traverse::TraverseCmd\)' "$CLI_MAIN" >/dev/null && rg -n 'pub mod project;|pub mod resource;|pub mod trajectory;|pub mod traverse;' "$CLI_MOD" >/dev/null; then
  echo "✓ PASS: CLI registers Spec96 project/resource/trajectory/traverse domains"
else
  echo "✗ FAIL: CLI command domains missing from main/mod" >&2
  exit 1
fi

if rg -n '/v1/project/identity|/v1/project/verify|ProjectCmd' "$PROJECT" >/dev/null; then
  echo "✓ PASS: project CLI maps to ProjectIdentity APIs"
else
  echo "✗ FAIL: project CLI parity missing" >&2
  exit 1
fi

if rg -n '/v1/trajectory/view|/v1/trajectory/define-goal|/v1/trajectory/assess|/v1/trajectory/propose-workpoint|/v1/trajectory/checkpoint|/v1/trajectory/resume|TrajectoryCmd' "$TRAJECTORY" >/dev/null; then
  echo "✓ PASS: trajectory CLI maps to full Trajectory API set"
else
  echo "✗ FAIL: trajectory CLI parity missing" >&2
  exit 1
fi

if rg -n '/v1/traverse|/v1/traverse/verify-tags|include_payload|include_rehydrate_refs|tag_mode|budget_tokens|TraverseCmd' "$TRAVERSE" >/dev/null; then
  echo "✓ PASS: traverse CLI maps to Spec96 bounded traversal schema"
else
  echo "✗ FAIL: traverse CLI parity missing" >&2
  exit 1
fi

if rg -n '/v1/resource/mode|activate_lowmem|deactivate_lowmem|set_mode|ResourceCmd' "$RESOURCE" >/dev/null; then
  echo "✓ PASS: resource CLI maps to ResourceMode API"
else
  echo "✗ FAIL: resource CLI parity missing" >&2
  exit 1
fi

if rg -n 'project .*Project identity|trajectory .*Trajectory Projection|traverse .*bounded|resource .*ResourceMode|focusa trajectory view|focusa traverse read|focusa resource status|focusa project identity' "$CLI_DOC" >/dev/null; then
  echo "✓ PASS: CLI docs advertise Spec96 parity commands"
else
  echo "✗ FAIL: CLI docs missing Spec96 parity commands" >&2
  exit 1
fi

echo "SPEC96 CLI parity static test: PASS"
