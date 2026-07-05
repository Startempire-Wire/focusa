#!/usr/bin/env bash
# Render a uniform self-heal decision block for GitHub Step Summary.
set -euo pipefail

summary_path="${GITHUB_STEP_SUMMARY:-/dev/stdout}"
surface="${SELF_HEAL_SURFACE:-Self-heal}"
workflow="${SELF_HEAL_WORKFLOW:-unknown}"
run_id="${SELF_HEAL_RUN_ID:-unknown}"
head_sha="${SELF_HEAL_HEAD_SHA:-unknown}"
failure_class="${failure_class:-unknown_process_failure}"
retry_policy="${retry_policy:-unknown}"
deterministic="${deterministic:-unknown}"
safe_to_rerun="${safe_to_rerun_unchanged:-unknown}"
plain_language_error="${plain_language_error:-unknown}"
likely_root_cause="${likely_root_cause:-unknown}"
remediation_template="${remediation_template:-unknown}"
source_refs="${source_refs:-}"
signals="${signals:-}"

decision="unknown_policy_stop"
next_command="inspect failed logs and classifier output"
if [[ "$retry_policy" == "hard_failure_no_rerun" ]]; then
  decision="repair_required_no_rerun"
  next_command="patch the cited source refs, then let GitHub CI run again"
elif [[ "$retry_policy" == "rerun_once" ]]; then
  decision="rerun_once_allowed"
  next_command="rerun failed jobs or redispatch deploy once; if repeated, repair manually"
fi

{
  echo "### Self-heal decision — ${surface}"
  echo ""
  echo "| field | value |"
  echo "| --- | --- |"
  echo "| workflow | \`${workflow}\` |"
  echo "| upstream_run | \`${run_id}\` |"
  echo "| head_sha | \`${head_sha}\` |"
  echo "| failure_class | \`${failure_class}\` |"
  echo "| retry_policy | \`${retry_policy}\` |"
  echo "| deterministic | \`${deterministic}\` |"
  echo "| safe_to_rerun_unchanged | \`${safe_to_rerun}\` |"
  echo "| decision | \`${decision}\` |"
  echo "| source_refs | \`${source_refs:-none}\` |"
  echo "| signals | \`${signals:-none}\` |"
  echo ""
  echo "plain_language_error: ${plain_language_error}"
  echo ""
  echo "likely_root_cause: ${likely_root_cause}"
  echo ""
  echo "remediation: ${remediation_template}"
  echo ""
  echo "next_command: ${next_command}"
} >> "$summary_path"
