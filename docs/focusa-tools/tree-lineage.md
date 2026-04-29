# Focusa Tree, Lineage, and Snapshot Tools

Use these tools for recoverable state, lineage-aware reasoning, safe comparisons, and rollback planning.

Each tool below includes what it does, when to use it, and one concrete example. Examples use Pi tool-call style; adapt parameter syntax to the active harness.

## `focusa_tree_head`

Reads current lineage head; start here when branch/head context matters.

Example:

```text
focusa_tree_head
```

## `focusa_tree_path`

Looks up ancestry for a CLT node without guessing from transcript.

Example:

```text
focusa_tree_path clt_node_id="019dd..."
```

## `focusa_tree_snapshot_state`

Creates a recoverable state snapshot before risky work.

Example:

```text
focusa_tree_snapshot_state snapshot_reason="before README rewrite"
```

## `focusa_tree_restore_state`

Restores a saved snapshot; use only for intentional rollback/recovery.

Example:

```text
focusa_tree_restore_state snapshot_id="snap-123" restore_mode="merge"
```

## `focusa_tree_diff_context`

Compares two snapshots safely.

Example:

```text
focusa_tree_diff_context from_snapshot_id="snap-old" to_snapshot_id="snap-new"
```

## `focusa_tree_recent_snapshots`

Finds recent snapshot IDs before diff/restore.

Example:

```text
focusa_tree_recent_snapshots limit=5
```

## `focusa_tree_snapshot_compare_latest`

Creates a snapshot and compares with latest/baseline in one step.

Example:

```text
focusa_tree_snapshot_compare_latest snapshot_reason="after docs update"
```

## `focusa_lineage_tree`

Fetches lineage tree for tree-aware reasoning.

Example:

```text
focusa_lineage_tree max_nodes=100
```

## `focusa_li_tree_extract`

Extracts decision/constraint/risk/reflection candidates from lineage.

Example:

```text
focusa_li_tree_extract max_candidates=12
```
