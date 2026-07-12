#!/usr/bin/env bash
# Spec 110 §16: Optional shell fallback shim for non-Pi environments.
# Provides degraded reminder coverage when Pi hook visibility is unavailable.
# Install guidance only; do not auto-install without operator approval.
#
# Usage:
#   # zsh broad fallback
#   source "$HOME/.local/share/focusa/focusa-pi-shell-reminder.sh"
#
#   # bash fallback
#   source "$HOME/.local/share/focusa/focusa-pi-shell-reminder.sh"

mode="${FOCUSA_PI_AGENT_REMINDER_MODE:-all}"

if [[ "$mode" == "off" ]]; then
  return 0 2>/dev/null || exit 0
fi

if [[ -n "${FOCUSA_PI_SESSION:-}" || -n "${FOCUSA_PROJECT_ROOT:-}" || "$PWD" == *"/focusa"* ]]; then
  echo "Focusa reminder: Prefer focusa_* Pi tools for Focusa daemon/state work." >&2
  echo "Start with: focusa_agent_prompt -> focusa_tool_doctor -> focusa_workpoint_resume -> focusa_evidence_capture." >&2
fi
