#!/usr/bin/env bash
# Create the next unique Focusa dev release tag and stamp menubar metadata.
#
# Default dry-run:
#   scripts/create-dev-release-tag.sh
# Push release tag + main and wait for GitHub CI/Release/Deploy workflows:
#   scripts/create-dev-release-tag.sh --push
# Push without waiting for GitHub workflows:
#   scripts/create-dev-release-tag.sh --push --no-wait-ci --no-wait-deploy
# Force a release gate override (requires plain-language reason):
#   scripts/create-dev-release-tag.sh --push --force-release --release-reason "critical deploy fix"
# Pin a major/minor lane:
#   scripts/create-dev-release-tag.sh --base 0.9 --push
# Publish an exact stable or preview tag:
#   scripts/create-dev-release-tag.sh --tag v0.9.136 --push
# Canonical journal mode: auto (default), required, or off.
#   FOCUSA_RELEASE_JOURNAL_MODE=required scripts/create-dev-release-tag.sh --tag v0.9.136 --push

set -euo pipefail
cd "$(dirname "$0")/.."

BASE="0.9"
EXACT_TAG=""
PUSH=0
DRY_RUN=0
WAIT_CI=1
WAIT_DEPLOY=1
CI_TIMEOUT_SECS=1200
FORCE_RELEASE=0
RELEASE_REASON=""
RELEASE_JOURNAL_MODE="${FOCUSA_RELEASE_JOURNAL_MODE:-auto}"
RELEASE_JOURNAL_ACTIVE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base)
      BASE="${2:?--base requires MAJOR.MINOR, e.g. 0.9}"
      shift 2
      ;;
    --tag)
      EXACT_TAG="${2:?--tag requires an exact tag, e.g. v0.9.136}"
      shift 2
      ;;
    --push)
      PUSH=1
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --force-release)
      FORCE_RELEASE=1
      shift
      ;;
    --release-reason)
      RELEASE_REASON="${2:?--release-reason requires a plain-language reason}"
      shift 2
      ;;
    --wait-ci)
      WAIT_CI=1
      shift
      ;;
    --no-wait-ci)
      WAIT_CI=0
      shift
      ;;
    --ci-timeout)
      CI_TIMEOUT_SECS="${2:?--ci-timeout requires seconds (default 1200; keep release path within 15-20 minutes unless GitHub is degraded)}"
      shift 2
      ;;
    --wait-deploy)
      WAIT_DEPLOY=1
      shift
      ;;
    --no-wait-deploy)
      WAIT_DEPLOY=0
      shift
      ;;
    -h|--help)
      sed -n '1,18p' "$0"
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if ! [[ "$BASE" =~ ^[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid --base '$BASE'; expected MAJOR.MINOR, e.g. 0.9" >&2
  exit 2
fi

if [[ -n "$EXACT_TAG" ]] && ! [[ "$EXACT_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z][0-9A-Za-z.-]*)?$ ]]; then
  echo "Invalid --tag '$EXACT_TAG'; expected a semantic release tag, e.g. v0.9.136" >&2
  exit 2
fi

if ! [[ "$RELEASE_JOURNAL_MODE" =~ ^(auto|required|off)$ ]]; then
  echo "Invalid FOCUSA_RELEASE_JOURNAL_MODE '$RELEASE_JOURNAL_MODE'; expected auto, required, or off" >&2
  exit 2
fi

if ! [[ "$CI_TIMEOUT_SECS" =~ ^[0-9]+$ ]]; then
  echo "Invalid --ci-timeout '$CI_TIMEOUT_SECS'; expected seconds" >&2
  exit 2
fi

push_candidate_main_with_auto_rebase() {
  local tag="${1:-$TAG}"
  local max_attempts=3
  local attempt=1

  while [[ "$attempt" -le "$max_attempts" ]]; do
    echo "Pushing main and ${tag} (attempt ${attempt}/${max_attempts})..."
    if git push origin HEAD:main && git push origin "${tag}"; then
      return 0
    fi

    echo "push_failed_non_fast_forward_or_remote_race: auto-healing with git pull --rebase and tag retarget" >&2
    # Audit Recorder/Watchdog commits can move origin/main while this helper is
    # stamping a release. Rebase, retarget the still-local tag to the rebased
    # HEAD, and retry. This keeps the canonical full pipeline intact without
    # manual rebase/retag intervention.
    git pull --rebase origin main
    if [[ "$FORCE_RELEASE" -eq 1 ]]; then
      git tag -fa "${tag}" -m "Release override: ${RELEASE_REASON}" HEAD
    else
      git tag -f "${tag}" HEAD
    fi
    attempt=$((attempt + 1))
  done

  echo "release_tag_push_failed_after_auto_rebase: tag=${tag}; inspect gh/audit logs and fix the pipeline system" >&2
  return 1
}

ensure_source_workflow() {
  local workflow="$1"
  local sha="$2"
  local existing remote_main
  existing="$(gh run list --workflow "$workflow" --commit "$sha" --limit 10 --json headSha 2>/dev/null || echo '[]')"
  if jq -e --arg sha "$sha" 'any(.headSha == $sha)' <<<"$existing" >/dev/null; then
    return 0
  fi
  remote_main="$(git ls-remote origin refs/heads/main | awk '{print $1}')"
  if [[ "$remote_main" != "$sha" ]]; then
    echo "source_gate_dispatch_blocked: workflow=${workflow} candidate=${sha} remote_main=${remote_main}" >&2
    return 1
  fi
  echo "source_gate_dispatch: workflow=${workflow} exact_main=${sha}"
  gh workflow run "$workflow" --ref main
}

wait_for_source_workflow() {
  local workflow="$1"
  local sha="$2"
  local deadline=$((SECONDS + CI_TIMEOUT_SECS))
  local status conclusion url

  command -v gh >/dev/null 2>&1 || {
    echo "source_gate_blocked: gh CLI is required to verify ${workflow} for ${sha}" >&2
    return 1
  }
  while (( SECONDS < deadline )); do
    local runs
    runs="$(gh run list --workflow "$workflow" --commit "$sha" --limit 10 --json status,conclusion,url,headSha 2>/dev/null || echo '[]')"
    # Duplicate/superseded runs on the same SHA are possible (double dispatch,
    # cache races). The gate passes when ANY completed run succeeded and fails
    # only when every completed run failed.
    if jq -e --arg sha "$sha" 'map(select(.headSha == $sha and .status == "completed")) | any(.conclusion == "success")' <<<"$runs" >/dev/null; then
      url="$(jq -r --arg sha "$sha" 'map(select(.headSha == $sha and .status == "completed" and .conclusion == "success")) | .[0].url // ""' <<<"$runs")"
      echo "source_gate_passed: workflow=${workflow} sha=${sha} url=${url}"
      return 0
    fi
    if jq -e --arg sha "$sha" 'map(select(.headSha == $sha)) | length > 0 and all(.status == "completed") and all(.conclusion != "success")' <<<"$runs" >/dev/null; then
      conclusion="$(jq -r --arg sha "$sha" 'map(select(.headSha == $sha and .status == "completed")) | .[0].conclusion // ""' <<<"$runs")"
      url="$(jq -r --arg sha "$sha" 'map(select(.headSha == $sha and .status == "completed")) | .[0].url // ""' <<<"$runs")"
      echo "source_gate_failed: workflow=${workflow} sha=${sha} conclusion=${conclusion} url=${url}" >&2
      return 1
    fi
    sleep 10
  done
  echo "source_gate_timeout: workflow=${workflow} sha=${sha} timeout=${CI_TIMEOUT_SECS}s" >&2
  return 1
}

report_workflow_failure() {
  local workflow="$1"
  local run_id="$2"
  local log_file
  log_file=$(mktemp)

  echo "workflow_failure name=${workflow} run_id=${run_id}" >&2
  gh run view "$run_id" --json url,jobs --jq '
    "run_url=" + .url,
    (.jobs[] | select(.conclusion == "failure") |
      "failed_job=" + .name + " job_url=" + .url,
      (.steps[] | select(.conclusion == "failure") | "failed_step=" + .name))
  ' >&2 || echo "workflow_failure_summary_query_error run_id=${run_id}" >&2

  if gh run view "$run_id" --log-failed >"$log_file" 2>&1; then
    echo "workflow_error_excerpt_begin max_lines=40 max_chars_per_line=500" >&2
    python3 - "$log_file" <<'PY' >&2
import re
import sys
from pathlib import Path

lines = Path(sys.argv[1]).read_text(errors="replace").splitlines()
pattern = re.compile(
    r"fail|error|exception|assert|traceback|not ok|mismatch|timed? out|unreachable|blocked",
    re.IGNORECASE,
)
matches = [line for line in lines if pattern.search(line)]
selected = (matches or lines[-40:])[-40:]
for line in selected:
    print(line[:500])
PY
    echo "workflow_error_excerpt_end" >&2
  else
    echo "workflow_failed_log_query_error run_id=${run_id}" >&2
  fi
  rm -f "$log_file"
  echo "full_log_command=gh run view ${run_id} --log-failed" >&2
}

watch_workflow_run_bounded() {
  local workflow="$1"
  local run_id="$2"
  local deadline="$3"
  local started=$SECONDS
  local next_heartbeat=$SECONDS
  local previous_digest=""
  local summary status conclusion digest elapsed

  while [[ $SECONDS -lt $deadline ]]; do
    if ! summary=$(gh run view "$run_id" --json status,conclusion,url,jobs 2>&1); then
      echo "workflow_status_query_error name=${workflow} run_id=${run_id} detail=$(printf '%s' "$summary" | tr '\n' ' ' | cut -c1-500)" >&2
      sleep 10
      continue
    fi
    status=$(jq -r '.status' <<<"$summary")
    conclusion=$(jq -r '.conclusion // ""' <<<"$summary")
    digest=$(jq -c '[.status,.conclusion,[.jobs[]|[.name,.status,.conclusion]]]' <<<"$summary")
    elapsed=$((SECONDS - started))

    if [[ "$digest" != "$previous_digest" || $SECONDS -ge $next_heartbeat ]]; then
      echo "workflow_heartbeat name=${workflow} run_id=${run_id} elapsed_s=${elapsed} status=${status} conclusion=${conclusion:-pending} url=$(jq -r '.url' <<<"$summary")"
      jq -r '.jobs[] | "  job=" + .name + " status=" + .status + " conclusion=" + (.conclusion // "pending")' <<<"$summary"
      previous_digest="$digest"
      next_heartbeat=$((SECONDS + 30))
    fi

    if [[ "$status" == "completed" ]]; then
      if [[ "$conclusion" == "success" ]]; then
        echo "workflow_completed name=${workflow} run_id=${run_id} elapsed_s=${elapsed} conclusion=success"
        return 0
      fi
      report_workflow_failure "$workflow" "$run_id"
      return 1
    fi
    sleep 10
  done

  echo "workflow_timeout name=${workflow} run_id=${run_id} timeout_s=${CI_TIMEOUT_SECS}" >&2
  gh run view "$run_id" --json url,jobs --jq '"run_url=" + .url, (.jobs[] | "job=" + .name + " status=" + .status + " conclusion=" + (.conclusion // "pending"))' >&2 || true
  return 1
}

wait_for_workflow() {
  local workflow="$1"
  local head_sha="$2"
  local head_branch="${3:-}"
  local deadline=$((SECONDS + CI_TIMEOUT_SECS))
  local run_id=""

  if ! command -v gh >/dev/null 2>&1; then
    echo "gh CLI is required to track ${workflow}; install/auth gh or pass --no-wait-ci." >&2
    exit 1
  fi

  echo "workflow_discovery name=${workflow} timeout_s=${CI_TIMEOUT_SECS} sha=${head_sha:0:7}${head_branch:+ head_branch=$head_branch}"
  while [[ $SECONDS -lt $deadline ]]; do
    if [[ -n "$head_branch" ]]; then
      run_id=$(gh run list --commit "$head_sha" --workflow "$workflow" --limit 10 --json databaseId,headBranch 2>/dev/null \
        | jq -r --arg branch "$head_branch" '.[] | select(.headBranch == $branch) | .databaseId' \
        | head -1 || true)
    else
      run_id=$(gh run list --commit "$head_sha" --workflow "$workflow" --limit 1 --json databaseId --jq '.[0].databaseId // empty' 2>/dev/null || true)
    fi
    if [[ -n "$run_id" ]]; then
      echo "workflow_discovered name=${workflow} run_id=${run_id} url=https://github.com/Startempire-Wire/focusa/actions/runs/${run_id}"
      watch_workflow_run_bounded "$workflow" "$run_id" "$deadline"
      return $?
    fi
    echo "workflow_discovery_heartbeat name=${workflow} elapsed_s=$((CI_TIMEOUT_SECS - (deadline - SECONDS))) status=not_found"
    sleep 10
  done

  echo "workflow_discovery_timeout name=${workflow} sha=${head_sha} timeout_s=${CI_TIMEOUT_SECS}" >&2
  exit 1
}

normalize_release_channel() {
  local tag="$1"
  local channel="$2"
  local release
  local latest
  case "$channel" in
    preview)
      gh release edit "$tag" --prerelease=true --latest=false
      ;;
    stable)
      gh release edit "$tag" --prerelease=false --latest=true
      ;;
    *)
      echo "Unsupported release channel normalization: ${channel}" >&2
      return 1
      ;;
  esac
  release="$(gh release view "$tag" --json isPrerelease,tagName)"
  latest="$(gh release list --limit 100 --json isLatest,tagName \
    | jq -c --arg tag "$tag" '.[] | select(.tagName == $tag)')"
  [[ -n "$latest" ]] || { echo "Release ${tag} missing from latest-release projection" >&2; return 1; }
  if [[ "$channel" == "preview" ]]; then
    jq -e '.isPrerelease == true' <<<"$release" >/dev/null
    jq -e '.isLatest == false' <<<"$latest" >/dev/null
  else
    jq -e '.isPrerelease == false' <<<"$release" >/dev/null
    jq -e '.isLatest == true' <<<"$latest" >/dev/null
  fi
  echo "release_channel_normalized=$(jq -cn --argjson release "$release" --argjson latest "$latest" '{release:$release,latest:$latest}')"
}

journal_client() {
  python3 scripts/canonical-release-journal.py "$@"
}

journal_problem_on_error() {
  local exit_code="$1"
  local line_number="$2"
  trap - ERR
  if [[ "$RELEASE_JOURNAL_ACTIVE" -eq 1 ]]; then
    journal_client problem --tag "$TAG" --stage "release-script" \
      --diagnosis "release script exited with code ${exit_code} at line ${line_number}" \
      --impact "canonical release did not finalize" \
      --recovery "inspect the failed command, preserve immutable tags, then append recovery progress" \
      --evidence-ref "script:scripts/create-dev-release-tag.sh:${line_number}" >/dev/null 2>&1 || true
  fi
  exit "$exit_code"
}

trap 'journal_problem_on_error $? $LINENO' ERR

if [[ -n "$(git status --porcelain)" ]]; then
  RELEASE_RETRY_DIRTY=0
  if [[ -n "$EXACT_TAG" ]] && [[ -f docs/current/.release-version-stamp ]] && \
    [[ "$(tr -d '[:space:]' < docs/current/.release-version-stamp)" == "${EXACT_TAG#v}" ]]; then
    RELEASE_RETRY_DIRTY=1
    while IFS= read -r dirty_path; do
      case "$dirty_path" in
        Cargo.toml|Cargo.lock|README.md|docs/current/.release-version-stamp|docs/current/CURRENT_RUNTIME_STATUS.md|docs/contracts/spec141/generated-capability-v2/agent-card.json|apps/menubar/package.json|apps/menubar/package-lock.json|apps/menubar/src-tauri/Cargo.toml|apps/menubar/src-tauri/Cargo.lock|apps/menubar/src-tauri/tauri.conf.json|apps/menubar/src/lib/components/Settings.svelte|apps/pi-extension/package.json|apps/pi-extension/package-lock.json|apps/pi-extension/src/auto-compaction.ts) ;;
        *) RELEASE_RETRY_DIRTY=0; break ;;
      esac
    done < <(git status --porcelain | cut -c4-)
  fi
  if [[ "$RELEASE_RETRY_DIRTY" -eq 1 ]]; then
    echo "Resuming exact stamped release surfaces for ${EXACT_TAG}."
  else
    echo "Working tree is not clean. Commit/revert current changes before creating a release tag." >&2
    git status --short >&2
    exit 1
  fi
fi

git fetch --tags --quiet origin || git fetch --tags --quiet

VERSION_SELECTION_ARGS=(--base "$BASE" --use-git-tags)
if [[ -n "$EXACT_TAG" ]]; then
  VERSION_SELECTION_ARGS+=(--tag "$EXACT_TAG")
fi
VERSION_SELECTION="$(python3 scripts/select-release-version.py "${VERSION_SELECTION_ARGS[@]}")"
jq -e '.status == "completed" and .monotonic == true' <<<"$VERSION_SELECTION" >/dev/null
TAG="$(jq -r '.selected_tag' <<<"$VERSION_SELECTION")"
VERSION="$(jq -r '.selected_version' <<<"$VERSION_SELECTION")"
SELECTED_CHANNEL="$(jq -r '.selected_channel' <<<"$VERSION_SELECTION")"
case "$SELECTED_CHANNEL" in
  dev|rc|preview) RELEASE_CHANNEL="preview" ;;
  stable) RELEASE_CHANNEL="stable" ;;
  *) echo "Unsupported selected release channel: ${SELECTED_CHANNEL}" >&2; exit 1 ;;
esac
VERSION_SELECTION_DETAILS="$(jq -c '{base,mode,selected_tag,selected_channel,highest_patch,channel_maxima,considered_tags,ignored_malformed_tags,monotonic}' <<<"$VERSION_SELECTION")"
echo "release_version_selection=${VERSION_SELECTION_DETAILS}"

if git rev-parse -q --verify "refs/tags/${TAG}" >/dev/null; then
  echo "Tag already exists: ${TAG}" >&2
  exit 1
fi

echo "Next release tag: ${TAG}"
python3 scripts/verify-release-tag-trigger.py "${TAG}"

# Release strategy preflight (docs/release-strategy.md): fail fast on policy
# violations before stamping/pushing. --force-release remains the override;
# dry-run continues and reports the would-be block.
if [[ -f scripts/next-version.py ]]; then
  VERSION_POLICY="$(python3 scripts/next-version.py --tag "$TAG" --json 2>/dev/null || true)"
  if [[ -n "$VERSION_POLICY" ]] && jq -e '.violations | length > 0' <<<"$VERSION_POLICY" >/dev/null 2>&1; then
    echo "release_version_policy_violation: $(jq -r '.violations | join("; ")' <<<"$VERSION_POLICY")" >&2
    if [[ "$DRY_RUN" -eq 1 ]]; then
      echo "Dry run continuing: release version policy would block an actual release." >&2
    elif [[ "$FORCE_RELEASE" -eq 1 ]]; then
      echo "Release version policy override accepted: ${RELEASE_REASON}" >&2
    else
      echo "Blocked by release version policy; pass --force-release --release-reason \"<plain-language reason>\" to override." >&2
      exit 1
    fi
  else
    VERSION_POLICY_WARNINGS="$(jq -r '.warnings | join("; ")' <<<"$VERSION_POLICY" 2>/dev/null || true)"
    [[ -z "$VERSION_POLICY_WARNINGS" ]] || echo "release_version_policy_warnings: ${VERSION_POLICY_WARNINGS}" >&2
  fi
fi

if [[ "$FORCE_RELEASE" -eq 1 && -z "$RELEASE_REASON" ]]; then
  echo "Blocked: --force-release requires --release-reason with a plain-language reason." >&2
  exit 2
fi

PREVIOUS_TAG=$(git describe --tags --abbrev=0 2>/dev/null || true)
if [[ -n "$PREVIOUS_TAG" ]]; then
  echo "Validating meaningful commit subjects in ${PREVIOUS_TAG}..HEAD..."
  if ! scripts/validate-commit-messages.sh --range "${PREVIOUS_TAG}..HEAD"; then
    if [[ "$FORCE_RELEASE" -eq 1 ]]; then
      echo "Commit-message policy override accepted and will be recorded in the annotated tag: ${RELEASE_REASON}" >&2
    else
      exit 1
    fi
  fi
else
  echo "No previous tag found; validating the current commit subject."
  scripts/validate-commit-messages.sh --range "HEAD^..HEAD"
fi

if ! FOCUSA_RELEASE_CHANNEL="$RELEASE_CHANNEL" python3 scripts/release-gate.py; then
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "Dry run continuing: ReleaseGate would block an actual release." >&2
  elif [[ "$FORCE_RELEASE" -eq 1 ]]; then
    echo "ReleaseGate override accepted: ${RELEASE_REASON}" >&2
  else
    exit 1
  fi
else
  echo "ReleaseGate passed."
fi

python3 tests/spec145_canonical_release_cycle_static_test.py
jq -e '.schema == "focusa.release_topology.v1" and (.surfaces | length) > 0' \
  config/focusa-release-topology.json >/dev/null

if [[ "$PUSH" -eq 1 && "$RELEASE_JOURNAL_MODE" != "off" ]]; then
  if journal_client history --project-id focusa --limit 1 >/dev/null 2>&1; then
    RELEASE_HISTORY="$(journal_client history --release-id "focusa:${TAG}" --limit 100)"
    if python3 -c 'import json,sys; data=json.load(sys.stdin); raise SystemExit(0 if any(event.get("phase") == "plan" for event in data.get("events", [])) else 1)' <<<"$RELEASE_HISTORY"; then
      echo "Canonical release journal plan resumed for ${TAG}."
    else
      journal_client plan --tag "$TAG" --channel "$RELEASE_CHANNEL"
      echo "Canonical release journal plan accepted for ${TAG}."
    fi
    RELEASE_JOURNAL_ACTIVE=1
    journal_client progress --tag "$TAG" --stage "version-selection" --status "completed" \
      --details "$VERSION_SELECTION_DETAILS" \
      --evidence-ref "script:scripts/select-release-version.py"
  elif [[ "$RELEASE_JOURNAL_MODE" == "required" ]]; then
    echo "Canonical release journal is required but agent-kb-api is unavailable." >&2
    exit 1
  else
    echo "Canonical release journal unavailable; continuing in auto mode without lifecycle publishing." >&2
  fi
fi

if [[ "$PUSH" -eq 1 ]]; then
  if ! python3 scripts/run-release-learning-guards.py --tag "$TAG"; then
    if [[ "$RELEASE_JOURNAL_ACTIVE" -eq 1 ]]; then
      journal_client problem --tag "$TAG" --stage "learning-guards" \
        --diagnosis "one or more retrieved release recurrence guards blocked" \
        --impact "release stopped before version stamping or immutable tagging" \
        --recovery "resolve the blocking resource or regression and rerun the same planned release" \
        --evidence-ref "artifact:/tmp/focusa-${VERSION}-learning-guards.json"
    fi
    exit 1
  fi
  echo "Learned release recurrence guards passed for ${TAG}."
  if [[ "$RELEASE_JOURNAL_ACTIVE" -eq 1 ]]; then
    journal_client progress --tag "$TAG" --stage "learning-guards" --status "completed" \
      --details "all retrieved recurrence guards passed before version stamping" \
      --evidence-ref "artifact:/tmp/focusa-${VERSION}-learning-guards.json"
  fi
fi

if [[ -f docs/current/.release-version-stamp ]] && \
  [[ "$(tr -d '[:space:]' < docs/current/.release-version-stamp)" == "$VERSION" ]]; then
  echo "Release surfaces already stamped ${VERSION}; preserving exact retry SHA."
else
  echo "Stamping release surfaces: ${VERSION}"
  scripts/stamp-menubar-version.py "${TAG}"
  scripts/stamp-release-version "${VERSION}"
fi
python3 scripts/verify-version-surfaces.py "${TAG}"
scripts/verify-doc-version-consistency
node scripts/validate-docs-runtime-parity.mjs # distribution parity drift blocks this release

# DETERMINISTIC FINAL GATE — no agent discretion. If this fails, do not push.
# This is the same gate as pre-push, but --strict adds gap + Spec Gates.
if [[ "$PUSH" -eq 1 ]]; then
  echo "=== deterministic final gate: local-release-preflight --strict ==="
  bash scripts/local-release-preflight.sh --strict || {
    echo "deterministic gate FAILED — fix, do not push tag" >&2
    exit 1
  }
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  git diff --stat
  git checkout -- Cargo.toml Cargo.lock README.md \
    docs/current/.release-version-stamp docs/current/CURRENT_RUNTIME_STATUS.md \
    docs/contracts/spec141/generated-capability-v2/agent-card.json \
    scripts/install-focusa.sh \
    apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte \
    apps/pi-extension/package.json apps/pi-extension/package-lock.json \
    apps/pi-extension/src/auto-compaction.ts
  echo "Dry run complete; reverted stamped files."
  exit 0
fi

STAMPED_RELEASE_SURFACES=0
if [[ -n "$(git status --porcelain)" ]]; then
  git add Cargo.toml Cargo.lock README.md \
    docs/current/.release-version-stamp docs/current/CURRENT_RUNTIME_STATUS.md \
    docs/contracts/spec141/generated-capability-v2/agent-card.json \
    scripts/install-focusa.sh \
    apps/menubar/package.json apps/menubar/package-lock.json \
    apps/menubar/src-tauri/Cargo.toml apps/menubar/src-tauri/Cargo.lock \
    apps/menubar/src-tauri/tauri.conf.json apps/menubar/src/lib/components/Settings.svelte \
    apps/pi-extension/package.json apps/pi-extension/package-lock.json \
    apps/pi-extension/src/auto-compaction.ts
  git commit -m "chore: stamp release surfaces ${VERSION}"
  STAMPED_RELEASE_SURFACES=1
fi

# Version stamping changes governed source surfaces (any channel). Re-seal the locked
# candidate ancestry before source CI so proof never trails the stamped commit.
if [[ "$PUSH" -eq 1 && "$STAMPED_RELEASE_SURFACES" -eq 1 && \
      -f release-proof/audit/next-locked-release-candidate-ancestry.json ]]; then
  STAMPED_SOURCE_SHA="$(git rev-parse HEAD)"
  python3 scripts/generate-locked-release-candidate-ancestry.py \
    --candidate-ref "$STAMPED_SOURCE_SHA" \
    --audit-ref "$STAMPED_SOURCE_SHA"
  python3 scripts/generate-locked-release-governance-receipt.py \
    --generate-ephemeral \
    --governance-source-commit "$STAMPED_SOURCE_SHA"
  if [[ -n "$(git status --porcelain -- release-proof/audit/)" ]]; then
    git add release-proof/audit/
    git commit -m "chore(release): anchor stamped candidate proof"
  fi
fi

# The benchmark includes final release-gap ancestry checks, so it must observe
# the committed stamped source and its freshly sealed proof.
if [[ "$RELEASE_JOURNAL_ACTIVE" -eq 1 ]]; then
  if ! journal_client benchmark --tag "$TAG" --channel "$RELEASE_CHANNEL"; then
    if [[ "$RELEASE_CHANNEL" == "preview" ]]; then
      echo "Canonical pre-release benchmark advisory for dev ${TAG}: continuing (stable would block)" >&2
    else
      exit 1
    fi
  else
    echo "Canonical pre-release benchmark accepted for ${TAG}."
  fi
fi

if [[ "$PUSH" -eq 1 ]]; then
  push_candidate_main_with_auto_rebase
  HEAD_SHA=$(git rev-parse HEAD)
  # Lean canonical (F38): dev/preview advisory — tag push is cheap, CI is async.
  # Stable waits for exact candidate CI; dev logs advisory and continues so
  # releases remain unnoticeable. Release workflow re-checks candidate gate.
  if [[ "${RELEASE_CHANNEL:-dev}" == "stable" ]]; then
    echo "Waiting for exact stamped-candidate preflight before immutable tag: ${HEAD_SHA}"
    wait_for_source_workflow "CI" "$HEAD_SHA"
  else
    echo "Advisory: skipping blocking CI wait for dev channel (async CI will gate Release): ${HEAD_SHA}" >&2
    echo "source_gate_advisory: tag push continues, Release Contract Check will re-check CI green for ${HEAD_SHA}" >&2
  fi
  if [[ "$RELEASE_JOURNAL_ACTIVE" -eq 1 ]]; then
    journal_client progress --tag "$TAG" --stage "candidate-ci" --status "completed" \
      --details "exact stamped candidate passed pre-tag CI" \
      --evidence-ref "github:commit:${HEAD_SHA}"
  fi
  CANDIDATE_CHANGED_PATHS="$(git diff --name-only "${PREVIOUS_TAG:-HEAD^}"..HEAD)"
  if grep -Eq \
    '^(crates/focusa-terminal-ui/|crates/focusa-cli/src/commands/(install|update)\.rs$|crates/focusa-core/src/silent_sessions/|crates/focusa-session-runner/|apps/pi-extension/(package|package-lock)\.json$|tests/132-e5-|\.github/workflows/spec132-terminal-matrix\.yml$)' \
    <<<"$CANDIDATE_CHANGED_PATHS"; then
    ensure_source_workflow "Spec 132 terminal matrix" "$HEAD_SHA"
    if [[ "${RELEASE_CHANNEL:-dev}" == "stable" ]]; then
      wait_for_source_workflow "Spec 132 terminal matrix" "$HEAD_SHA"
    else
      echo "Advisory: Spec 132 not yet green for ${HEAD_SHA} — dev continues, Release will re-check" >&2
      wait_for_source_workflow "Spec 132 terminal matrix" "$HEAD_SHA" || echo "source_gate_advisory: Spec 132 pending for ${HEAD_SHA}" >&2
    fi
  fi
fi

if [[ "$FORCE_RELEASE" -eq 1 ]]; then
  git tag -a "${TAG}" -m "Release override: ${RELEASE_REASON}" HEAD
else
  git tag "${TAG}" HEAD
fi

echo "Created tag ${TAG} at $(git rev-parse --short HEAD)"

if [[ "$PUSH" -eq 1 ]]; then
  git push origin "${TAG}"
  echo "Pushed exact green candidate ${TAG}."
  if [[ "$RELEASE_JOURNAL_ACTIVE" -eq 1 ]]; then
    journal_client progress --tag "$TAG" --stage "tag-pushed" --status "completed" \
      --details "immutable tag pushed after exact-candidate CI" \
      --evidence-ref "github:tag:${TAG}"
  fi
  if [[ "$WAIT_CI" -eq 1 ]]; then
    wait_for_workflow "CI" "$HEAD_SHA"
    wait_for_workflow "Release" "$HEAD_SHA" "${TAG}"
    normalize_release_channel "$TAG" "$RELEASE_CHANNEL"
    if [[ "$RELEASE_JOURNAL_ACTIVE" -eq 1 ]]; then
      journal_client progress --tag "$TAG" --stage "release-channel" --status "completed" \
        --details "GitHub release classification normalized to ${RELEASE_CHANNEL}" \
        --evidence-ref "github:release:${TAG}"
    fi
    if [[ "$WAIT_DEPLOY" -eq 1 ]]; then
      wait_for_workflow "Deploy Live Daemon" "$HEAD_SHA"
      echo "GitHub CI, Release, and Deploy workflows passed for ${TAG}."
      if [[ "$RELEASE_JOURNAL_ACTIVE" -eq 1 ]]; then
        journal_client finalize --tag "$TAG" --channel "$RELEASE_CHANNEL"
        RELEASE_JOURNAL_ACTIVE=0
        trap - ERR
        echo "Canonical release journal finalized for ${TAG}."
      fi
    else
      echo "GitHub CI and Release workflows passed for ${TAG}."
    fi
  else
    echo "Not waiting for GitHub workflows. Track with: gh run list --commit ${HEAD_SHA}"
  fi
  if scripts/run-guardian-release-cleanup.sh post; then
    echo "Guardian-routed post-release artifact cleanup completed."
  else
    echo "Guardian-routed post-release cleanup needs operator review; release state is unchanged." >&2
  fi
else
  echo "Local only. Push with: git push origin HEAD:main && git push origin ${TAG}"
fi
