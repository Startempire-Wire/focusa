#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: run-cancellation-safe-cross.sh <cross-subcommand> [arguments...]" >&2
  exit 64
}

(($# > 0)) || usage
[[ "${GITHUB_RUN_ID:-}" =~ ^[0-9]+$ ]] || {
  echo "GITHUB_RUN_ID must be a numeric provider run identity" >&2
  exit 65
}
[[ "${GITHUB_RUN_ATTEMPT:-}" =~ ^[0-9]+$ ]] || {
  echo "GITHUB_RUN_ATTEMPT must be a numeric provider attempt identity" >&2
  exit 66
}
[[ "${GITHUB_JOB:-}" =~ ^[A-Za-z0-9_.-]+$ ]] || {
  echo "GITHUB_JOB must contain only label-safe characters" >&2
  exit 67
}

target=""
args=("$@")
for ((index = 0; index < ${#args[@]}; index++)); do
  case "${args[$index]}" in
    --target)
      ((index + 1 < ${#args[@]})) || usage
      target="${args[$((index + 1))]}"
      ;;
    --target=*) target="${args[$index]#--target=}" ;;
  esac
done
[[ "$target" =~ ^[A-Za-z0-9_.-]+$ ]] || {
  echo "cross invocation requires one label-safe --target" >&2
  exit 68
}

engine="${CROSS_CONTAINER_ENGINE:-}"
if [[ -z "$engine" ]]; then
  if command -v docker >/dev/null 2>&1; then
    engine="docker"
  elif command -v podman >/dev/null 2>&1; then
    engine="podman"
  else
    echo "neither docker nor podman is available" >&2
    exit 69
  fi
fi
[[ "$engine" =~ ^[A-Za-z0-9_.+-]+$ ]] && command -v "$engine" >/dev/null 2>&1 || {
  echo "CROSS_CONTAINER_ENGINE must name one executable" >&2
  exit 70
}
command -v cross >/dev/null 2>&1 || {
  echo "cross is not installed" >&2
  exit 71
}

label_run="focusa.github.run_id=$GITHUB_RUN_ID"
label_attempt="focusa.github.run_attempt=$GITHUB_RUN_ATTEMPT"
label_job="focusa.github.job=$GITHUB_JOB"
label_target="focusa.cross.target=$target"
owned_labels="--label=$label_run --label=$label_attempt --label=$label_job --label=$label_target"
export CROSS_CONTAINER_ENGINE="$engine"
export CROSS_CONTAINER_OPTS="${CROSS_CONTAINER_OPTS:+$CROSS_CONTAINER_OPTS }$owned_labels"

child_pid=""
cleanup_started=0
cleanup_owned_containers() {
  local original_status=$?
  local cleanup_status=0
  local id observed ps_output remaining_output
  local -a ids=()

  ((cleanup_started == 0)) || return "$original_status"
  cleanup_started=1
  trap - EXIT INT TERM HUP
  set +e

  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill -TERM -- "-$child_pid" 2>/dev/null
  fi

  ps_output="$(
    "$engine" ps -aq \
      --filter "label=$label_run" \
      --filter "label=$label_attempt" \
      --filter "label=$label_job" \
      --filter "label=$label_target"
  )"
  if (($? != 0)); then
    echo "failed to inventory exact job-owned cross containers" >&2
    cleanup_status=75
  elif [[ -n "$ps_output" ]]; then
    mapfile -t ids <<<"$ps_output"
  fi
  for id in "${ids[@]}"; do
    [[ -n "$id" ]] || continue
    if ! observed="$(
      "$engine" inspect \
        --format '{{ index .Config.Labels "focusa.github.run_id" }}|{{ index .Config.Labels "focusa.github.run_attempt" }}|{{ index .Config.Labels "focusa.github.job" }}|{{ index .Config.Labels "focusa.cross.target" }}' \
        "$id" 2>/dev/null
    )"; then
      if "$engine" inspect "$id" >/dev/null 2>&1; then
        echo "failed to inspect exact job-owned container identity: id=$id" >&2
        cleanup_status=77
      fi
      continue
    fi
    if [[ "$observed" != "$GITHUB_RUN_ID|$GITHUB_RUN_ATTEMPT|$GITHUB_JOB|$target" ]]; then
      echo "refusing container with mismatched exact identity: id=$id labels=$observed" >&2
      cleanup_status=72
      continue
    fi
    if ! "$engine" rm -f "$id" >/dev/null; then
      if "$engine" inspect "$id" >/dev/null 2>&1; then
        echo "failed to remove exact job-owned cross container: id=$id" >&2
        cleanup_status=73
      fi
    fi
  done

  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill -KILL -- "-$child_pid" 2>/dev/null
  fi
  [[ -z "$child_pid" ]] || wait "$child_pid" 2>/dev/null

  remaining_output="$(
    "$engine" ps -aq \
      --filter "label=$label_run" \
      --filter "label=$label_attempt" \
      --filter "label=$label_job" \
      --filter "label=$label_target"
  )"
  if (($? != 0)); then
    echo "failed to verify exact job-owned cross container cleanup" >&2
    cleanup_status=76
  elif [[ -n "$remaining_output" ]]; then
    echo "exact job-owned cross container residue remains: $remaining_output" >&2
    cleanup_status=74
  fi

  if ((cleanup_status != 0)); then
    exit "$cleanup_status"
  fi
  exit "$original_status"
}
trap cleanup_owned_containers EXIT
trap 'exit 143' TERM HUP
trap 'exit 130' INT

setsid --wait cross "$@" &
child_pid=$!
wait "$child_pid"
status=$?
child_pid=""
exit "$status"
