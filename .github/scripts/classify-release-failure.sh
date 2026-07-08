#!/usr/bin/env bash
# Compatibility wrapper for the DRY Focusa release-path classifier.
# Expected class keys emitted by the classifier (shared contract):
# ci_clippy_failure, ci_test_failure, release_cross_target_compile_failure,
# release_static_proof_failure, deploy_health_failure,
# runner_resource_failure, auto_heal_process_error,
# transient_github_or_network_failure, unknown_process_failure,
# hard_failure_no_rerun, plain_language_error.
# Workflows consume shell-safe KEY=value lines; agents/tests can call the
# Python classifier directly for structured JSON.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
exec python3 "$ROOT_DIR/scripts/classify-ci-failure.py" --format env "${1:--}"
