#!/usr/bin/env bash
# DRY failure classifier for release-path self-heal workflows.
# Output is shell-safe KEY=value pairs consumed by GitHub Actions jobs.
set -euo pipefail

log_file="${1:-/dev/stdin}"

failure_class="unknown_process_failure"
retry_policy="rerun_once"
plain_language_error="Transient or unknown release-path failure; one bounded rerun is allowed."

if grep -qE 'cargo clippy|clippy::|needless_borrow|derivable_impls|private_interfaces' "$log_file"; then
  failure_class="ci_clippy_failure"
  retry_policy="hard_failure_no_rerun"
  plain_language_error="Blocked: clippy found a deterministic code issue. Patch code, then let GitHub CI run again."
elif grep -qE 'cargo test|test result: FAILED|panicked at|assertion `left == right` failed' "$log_file"; then
  failure_class="ci_test_failure"
  retry_policy="hard_failure_no_rerun"
  plain_language_error="Blocked: tests failed deterministically. Patch code, then let GitHub CI run again."
elif grep -qE 'Rust Binaries|error\[E[0-9]+\]|could not compile|could not find `unix` in `os`|failed to resolve' "$log_file"; then
  failure_class="release_cross_target_compile_failure"
  retry_policy="hard_failure_no_rerun"
  plain_language_error="Blocked: release matrix hit a deterministic cross-target compile failure. Patch portability, then push; do not rerun the full matrix."
elif grep -qE 'release deploy automation static test|static proof|workflow name missing|static guard' "$log_file"; then
  failure_class="release_static_proof_failure"
  retry_policy="hard_failure_no_rerun"
  plain_language_error="Blocked: release static proof failed. Patch the workflow/spec guard, then let GitHub CI run again."
elif grep -qE 'deploy_health timeout|/v1/health|health check failed' "$log_file"; then
  failure_class="deploy_health_failure"
  retry_policy="rerun_once"
  plain_language_error="Deploy health failed; one bounded redeploy is allowed, then inspect service health."
elif grep -qE 'Killed|oom|out of memory|No space left on device|runner.*lost|The operation was canceled' "$log_file"; then
  failure_class="runner_resource_failure"
  retry_policy="rerun_once"
  plain_language_error="Runner/resource failure detected; one bounded rerun is allowed."
elif grep -qE 'failed to determine base repo|not a git repository|gh run rerun|gh workflow run' "$log_file"; then
  failure_class="auto_heal_process_error"
  retry_policy="hard_failure_no_rerun"
  plain_language_error="Blocked: self-heal process error. Patch the self-heal workflow before retrying."
elif grep -qE 'HTTP 5[0-9][0-9]|connection reset|timed out|TLS|rate limit|upload.*failed|artifact.*failed' "$log_file"; then
  failure_class="transient_github_or_network_failure"
  retry_policy="rerun_once"
  plain_language_error="Transient GitHub/network/upload failure detected; one bounded rerun is allowed."
fi

printf 'failure_class=%s\n' "$failure_class"
printf 'retry_policy=%s\n' "$retry_policy"
printf 'plain_language_error=%s\n' "$plain_language_error"
