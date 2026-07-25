#!/usr/bin/env python3
from pathlib import Path

root = Path(__file__).resolve().parents[1]
source = (root / "crates/focusa-core/src/silent_session_reducer.rs").read_text()
for forbidden in [
    "tokio::", "std::process", "std::fs", "Command::new", "Child", "Pty", "tmux",
    "worktree", "provider request", "sleep(", "retry_backoff",
]:
    assert forbidden not in source, f"pure reducer contains forbidden runtime concern: {forbidden}"
for required in [
    "lifecycle_transition_allowed", "reduce_silent_session", "TypedSessionBlocker",
    "WaitingInputRequiresExplicitFreshObservation", "CompletedRequiresReceiptReadyEvaluation",
    "ProcessExited", "semantic_activity_is_fresh",
]:
    assert required in source, required
print("Spec133 pure reducer contains typed facts and truth guards without runtime side effects: PASS")
