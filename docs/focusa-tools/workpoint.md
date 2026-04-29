# Focusa Workpoint Tools

Use Workpoint tools when continuity matters: compaction, model switches, forks, release proof, evidence capture, drift recovery, or any moment where transcript memory is no longer trustworthy.

Each tool below includes what it does, when to use it, and one concrete example. Examples use Pi tool-call style; adapt parameter syntax to the active harness.

## `focusa_workpoint_checkpoint`

Creates a reducer-owned continuation checkpoint with mission, active objects, action intent, evidence refs, blockers, next action, and canonical/degraded status. Use before compaction, risky work, model switch, fork, or handoff.

Example:

```text
focusa_workpoint_checkpoint mission="Ship Spec89 docs" current_action="release_verify" verified_evidence=["cargo test -p focusa-api workpoint"] next_action="push docs and verify GitHub" checkpoint_reason="manual" canonical=true
```

## `focusa_workpoint_resume`

Fetches the active WorkpointResumePacket. Use immediately after compaction/resume/overflow/model switch/uncertainty before choosing work.

Example:

```text
focusa_workpoint_resume mode="operator_summary"
```

## `focusa_workpoint_link_evidence`

Links a stable proof ref to the active canonical Workpoint so resume packets carry verification records.

Example:

```text
focusa_workpoint_link_evidence target_ref="release:docs" result="README links verified" evidence_ref="docs/focusa-tools/README.md:1"
```

## `focusa_active_object_resolve`

Resolves likely active object refs from current Workpoint and hints without inventing canonical truth.

Example:

```text
focusa_active_object_resolve hint="apps/pi-extension/src/tools.ts"
```

## `focusa_evidence_capture`

Captures a bounded evidence ref/result and optionally links it to active Workpoint.

Example:

```text
focusa_evidence_capture target_ref="README.md" result="Skill links added" evidence_ref="README.md:Documentation map" attach_to_workpoint=true
```
