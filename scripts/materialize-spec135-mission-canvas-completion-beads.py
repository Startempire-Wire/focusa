#!/usr/bin/env python3
"""Materialize the approved Spec 135 Mission Canvas completion DAG into Beads JSONL.

The script is deterministic and non-destructive: superseded focusa-mc-full*
records are retained as closed history, while the new focusa-mc2 hierarchy is
created with explicit parent-child and blocking dependencies.
"""
from __future__ import annotations

import argparse
import json
import os
import sqlite3
import tempfile
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
GRAPH_PATH = ROOT / "docs/contracts/spec135-mission-canvas-completion-dag.v2.json"
DEFAULT_JSONL = ROOT / ".beads/issues.jsonl"
DEFAULT_DB = ROOT / ".beads/beads.db"
GENERATED_PATH = ROOT / "docs/contracts/spec135-mission-canvas-beads-materialization.v2.jsonl"
STAMP = "2026-07-30T07:00:00Z"
ACTOR = "pi-spec135-materializer"
ROOT_ID = "focusa-mc2"
SUPERSEDED_PREFIX = "focusa-mc-full"
LEGACY_ISSUE_TYPE_MAP = {"security": "bug", "improvement": "feature"}
LEGACY_STATUS_MAP = {"deferred": "blocked"}


def bead_id_for_phase(phase_id: str) -> str:
    return f"{ROOT_ID}.{int(phase_id[1:]) + 1}"


def bead_id_for_node(node: dict[str, Any]) -> str:
    phase_number = int(node["phase"][1:]) + 1
    return f"{ROOT_ID}.{phase_number}.{int(node['sequence']):03d}"


def dependency(issue_id: str, depends_on_id: str, kind: str) -> dict[str, Any]:
    return {
        "issue_id": issue_id,
        "depends_on_id": depends_on_id,
        "type": kind,
        "created_at": STAMP,
        "created_by": ACTOR,
    }


def root_record(graph: dict[str, Any]) -> dict[str, Any]:
    return {
        "id": ROOT_ID,
        "title": "EPIC: Complete Spec 135 Mission Canvas rich adaptive Pi-extension UI",
        "description": (
            "Operator-approved execution epic generated from "
            "docs/contracts/spec135-mission-canvas-completion-dag.v2.json. "
            "Mission Canvas is a portable macOS/Windows/Linux Pi-extension rich "
            "workspace driven by canonical ResolvedWorkspaceProjection. Terminal "
            "Canvas is fallback only. Adaptive composition omits ineligible or empty "
            "contributions before geometry and forbids dead chrome."
        ),
        "design": (
            f"Graph digest {graph['graph_digest_sha256']}; "
            f"{graph['task_count_excluding_gates']} tasks, {graph['node_count']} nodes, "
            f"{graph['edge_count']} dependency edges, {graph['phase_count']} phases."
        ),
        "acceptance_criteria": (
            "Every DAG phase gate passes from implementation and runtime evidence; "
            "all Spec 135 through 135K closure artifacts are regenerated truthfully; "
            "the installed Pi extension passes macOS, Windows, and Linux runtime proof."
        ),
        "status": "open",
        "priority": 0,
        "issue_type": "epic",
        "created_at": STAMP,
        "updated_at": STAMP,
        "external_ref": "spec135-dag:v2",
        "source_repo": ".",
        "labels": ["spec135", "mission-canvas", "adaptive-composition", "rich-host", "operator-approved"],
    }


def phase_record(graph: dict[str, Any], phase: dict[str, Any]) -> dict[str, Any]:
    issue_id = bead_id_for_phase(phase["id"])
    return {
        "id": issue_id,
        "title": f"{phase['id']}: {phase['title']}",
        "description": phase["purpose"],
        "design": f"Requirement refs: {', '.join(phase['requirement_refs'])}",
        "acceptance_criteria": f"The {phase['id']} phase gate passes with evidence before the next phase begins.",
        "status": "open",
        "priority": 0 if phase["id"] in {"P00", "P01", "P02", "P03", "P04"} else 1,
        "issue_type": "epic",
        "created_at": STAMP,
        "updated_at": STAMP,
        "external_ref": f"spec135-dag:{phase['id']}",
        "source_repo": ".",
        "labels": ["spec135", "mission-canvas", "completion-phase", phase["id"].lower()],
        "dependencies": [dependency(issue_id, ROOT_ID, "parent-child")],
    }


def node_record(graph: dict[str, Any], node: dict[str, Any], node_ids: dict[str, str]) -> dict[str, Any]:
    issue_id = node_ids[node["id"]]
    phase_id = bead_id_for_phase(node["phase"])
    dependencies = [dependency(issue_id, phase_id, "parent-child")]
    dependencies.extend(dependency(issue_id, node_ids[dep], "blocks") for dep in node["depends_on"])
    targets = "\n".join(f"- {target}" for target in node["targets"]) or "- no direct file target; evidence gate"
    refs = ", ".join(node["requirement_refs"]) or "none"
    description = (
        f"DAG node: {node['id']}\n"
        f"Phase: {node['phase']}\n"
        f"Wave: {node['wave']}\n"
        f"Requirement refs: {refs}\n"
        f"Proof class: {node['proof_class']}\n"
        f"Targets:\n{targets}\n\n"
        f"Drift gate: {node['drift_gate']}"
    )
    labels = [
        "spec135",
        "mission-canvas",
        "completion-dag-v2",
        node["phase"].lower(),
        f"proof:{node['proof_class']}",
    ]
    if node["proof_class"] == "gate":
        labels.append("dependency-gate")
    return {
        "id": issue_id,
        "title": f"{node['id']}: {node['title']}",
        "description": description,
        "design": f"Generated from graph digest {graph['graph_digest_sha256']}; sequence {node['sequence']}.",
        "acceptance_criteria": node["expected_result"],
        "status": "open",
        "priority": 0 if node["phase"] in {"P00", "P01", "P02"} else 1,
        "issue_type": "task",
        "created_at": STAMP,
        "updated_at": STAMP,
        "external_ref": f"spec135-dag:{node['id']}",
        "source_repo": ".",
        "labels": labels,
        "dependencies": dependencies,
    }


def normalize_legacy_record(record: dict[str, Any]) -> dict[str, Any]:
    """Migrate unsupported provider enums without erasing their original meaning."""
    updated = dict(record)
    labels = list(updated.get("labels") or [])
    migrated = False
    issue_type = str(updated.get("issue_type") or "task")
    if issue_type in LEGACY_ISSUE_TYPE_MAP:
        legacy_label = f"legacy-type:{issue_type}"
        if legacy_label not in labels:
            labels.append(legacy_label)
        updated["issue_type"] = LEGACY_ISSUE_TYPE_MAP[issue_type]
        migrated = True
    status = str(updated.get("status") or "open")
    if status in LEGACY_STATUS_MAP:
        legacy_label = f"legacy-status:{status}"
        if legacy_label not in labels:
            labels.append(legacy_label)
        updated["status"] = LEGACY_STATUS_MAP[status]
        migrated = True
    if migrated:
        updated["labels"] = labels
    return updated


def supersede(record: dict[str, Any]) -> dict[str, Any]:
    updated = dict(record)
    labels = list(updated.get("labels") or [])
    for label in ("superseded", "superseded-by:focusa-mc2", "spec135-historical"):
        if label not in labels:
            labels.append(label)
    updated["labels"] = labels
    updated["superseded_by"] = ROOT_ID
    if updated.get("status") != "closed":
        updated["status"] = "closed"
        updated["closed_at"] = STAMP
        updated["close_reason"] = (
            "Superseded without deletion by operator-approved adaptive-composition "
            "completion DAG focusa-mc2 (commit 7b780693). Existing implementation and "
            "history remain evidence; execution moves to dependency-linked v2 tasks."
        )
        updated["updated_at"] = STAMP
    return updated


def build_records(graph: dict[str, Any]) -> list[dict[str, Any]]:
    node_ids = {node["id"]: bead_id_for_node(node) for node in graph["nodes"]}
    records = [root_record(graph)]
    records.extend(phase_record(graph, phase) for phase in graph["phases"])
    records.extend(node_record(graph, node, node_ids) for node in graph["nodes"])
    return records


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def dump_jsonl(records: list[dict[str, Any]]) -> str:
    return "".join(json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n" for record in records)


def merge_jsonl(existing: list[dict[str, Any]], generated: list[dict[str, Any]]) -> list[dict[str, Any]]:
    generated_by_id = {record["id"]: record for record in generated}
    merged: list[dict[str, Any]] = []
    seen: set[str] = set()
    mutable_fields = (
        "status",
        "assignee",
        "notes",
        "updated_at",
        "closed_at",
        "close_reason",
        "superseded_by",
        "source_repo",
    )
    for record in existing:
        record = normalize_legacy_record(record)
        issue_id = record["id"]
        if issue_id.startswith(SUPERSEDED_PREFIX):
            record = supersede(record)
        if issue_id in generated_by_id:
            refreshed = dict(generated_by_id[issue_id])
            for field in mutable_fields:
                if field in record:
                    refreshed[field] = record[field]
                elif field == "source_repo":
                    refreshed.pop(field, None)
            labels = list(record.get("labels") or [])
            labels.extend(label for label in refreshed.get("labels") or [] if label not in labels)
            refreshed["labels"] = labels
            record = refreshed
        merged.append(record)
        seen.add(issue_id)
    merged.extend(record for record in generated if record["id"] not in seen)
    return merged


def validate(graph: dict[str, Any], generated: list[dict[str, Any]], merged: list[dict[str, Any]]) -> None:
    expected = 1 + graph["phase_count"] + graph["node_count"]
    assert len(generated) == expected
    ids = [record["id"] for record in generated]
    assert len(ids) == len(set(ids))
    known = {record["id"] for record in merged}
    for record in generated:
        assert record["status"] == "open"
        for dep in record.get("dependencies") or []:
            assert dep["issue_id"] == record["id"]
            assert dep["depends_on_id"] in known
    superseded = [record for record in merged if record["id"].startswith(SUPERSEDED_PREFIX)]
    assert superseded
    assert all(record["status"] == "closed" and record.get("superseded_by") == ROOT_ID for record in superseded)
    assert graph["status"] in {"operator_approved_p00_execution", "operator_approved"}


def atomic_write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=path.name + ".", dir=path.parent)
    try:
        with os.fdopen(fd, "w") as handle:
            handle.write(text)
        os.replace(tmp, path)
    finally:
        if os.path.exists(tmp):
            os.unlink(tmp)


def merged_text_preserving_unchanged_lines(path: Path, merged: list[dict[str, Any]]) -> str:
    """Keep historical JSONL bytes stable and rewrite only semantically changed/new records."""
    desired = {record["id"]: record for record in merged}
    output: list[str] = []
    seen: set[str] = set()
    for raw in path.read_text().splitlines(keepends=True):
        current = json.loads(raw)
        issue_id = current["id"]
        seen.add(issue_id)
        if current == desired[issue_id]:
            output.append(raw)
        else:
            output.append(json.dumps(desired[issue_id], ensure_ascii=False, separators=(",", ":")) + "\n")
    output.extend(
        json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n"
        for record in merged
        if record["id"] not in seen
    )
    return "".join(output)


def sync_db(db_path: Path, records: list[dict[str, Any]]) -> None:
    """Project canonical targeted JSONL records into the local Beads SQLite cache.

    Beads JSONL remains source of truth. This bounded transaction is a recovery
    path for the current importer, whose per-edge cycle checks exceed practical
    runtime for the 1,014-edge graph.
    """
    targeted = [
        record
        for record in records
        if record["id"] == ROOT_ID
        or record["id"].startswith(ROOT_ID + ".")
        or record["id"].startswith(SUPERSEDED_PREFIX)
    ]
    connection = sqlite3.connect(db_path)
    try:
        connection.execute("PRAGMA foreign_keys = ON")
        connection.execute("BEGIN IMMEDIATE")
        # This database is a disposable projection of provider-owned JSONL. The
        # completion DAG can renumber generated issue ids while retaining stable
        # external refs, so upsert-by-id alone can violate external_ref uniqueness.
        # Rebuild only the bounded Spec 135 projection and let FK cascades clear
        # its derived cache rows before inserting the canonical JSONL records.
        stale_ids = [
            row[0]
            for row in connection.execute(
                "SELECT id FROM issues WHERE id = ? OR id LIKE ? OR id LIKE ?",
                (ROOT_ID, f"{ROOT_ID}.%", f"{SUPERSEDED_PREFIX}%"),
            )
        ]
        connection.executemany("DELETE FROM issues WHERE id = ?", ((issue_id,) for issue_id in stale_ids))
        for record in targeted:
            values = (
                record["id"],
                record.get("title", ""),
                record.get("description", ""),
                record.get("design", ""),
                record.get("acceptance_criteria", ""),
                record.get("status", "open"),
                int(record.get("priority", 2)),
                record.get("issue_type", "task"),
                record.get("created_at", STAMP),
                record.get("updated_at", STAMP),
                record.get("closed_at"),
                record.get("external_ref"),
                record.get("source_repo", "."),
                record.get("close_reason", ""),
                record.get("superseded_by", ""),
            )
            connection.execute(
                """
                INSERT INTO issues (
                    id, title, description, design, acceptance_criteria, status,
                    priority, issue_type, created_at, updated_at, closed_at,
                    external_ref, source_repo, close_reason, superseded_by
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    title=excluded.title,
                    description=excluded.description,
                    design=excluded.design,
                    acceptance_criteria=excluded.acceptance_criteria,
                    status=excluded.status,
                    priority=excluded.priority,
                    issue_type=excluded.issue_type,
                    updated_at=excluded.updated_at,
                    closed_at=excluded.closed_at,
                    external_ref=excluded.external_ref,
                    source_repo=excluded.source_repo,
                    close_reason=excluded.close_reason,
                    superseded_by=excluded.superseded_by
                """,
                values,
            )
        # Dependencies may point forward in DAG order; all issue rows must exist
        # before restoring labels and edges under immediate SQLite FK checks.
        for record in targeted:
            connection.execute("DELETE FROM labels WHERE issue_id = ?", (record["id"],))
            for label in record.get("labels") or []:
                connection.execute("INSERT OR IGNORE INTO labels(issue_id, label) VALUES (?, ?)", (record["id"], label))
            connection.execute("DELETE FROM dependencies WHERE issue_id = ?", (record["id"],))
            for dep in record.get("dependencies") or []:
                connection.execute(
                    """
                    INSERT OR IGNORE INTO dependencies(issue_id, depends_on_id, type, created_at, created_by)
                    VALUES (?, ?, ?, ?, ?)
                    """,
                    (dep["issue_id"], dep["depends_on_id"], dep["type"], dep.get("created_at", STAMP), dep.get("created_by", ACTOR)),
                )
        connection.commit()
        generated_count = connection.execute(
            "SELECT count(*) FROM issues WHERE id = ? OR id LIKE ?", (ROOT_ID, ROOT_ID + ".%")
        ).fetchone()[0]
        dependency_count = connection.execute(
            "SELECT count(*) FROM dependencies WHERE issue_id = ? OR issue_id LIKE ?", (ROOT_ID, ROOT_ID + ".%")
        ).fetchone()[0]
        open_superseded = connection.execute(
            "SELECT count(*) FROM issues WHERE id LIKE ? AND status != 'closed'", (SUPERSEDED_PREFIX + "%",)
        ).fetchone()[0]
        expected_generated = 1 + len(json.loads(GRAPH_PATH.read_text())["phases"]) + len(json.loads(GRAPH_PATH.read_text())["nodes"])
        expected_dependencies = sum(
            len(record.get("dependencies") or [])
            for record in targeted
            if record["id"] == ROOT_ID or record["id"].startswith(ROOT_ID + ".")
        )
        assert generated_count == expected_generated, (generated_count, expected_generated)
        assert dependency_count == expected_dependencies, (dependency_count, expected_dependencies)
        assert open_superseded == 0, open_superseded
        print(f"Synchronized Beads cache: issues={generated_count} dependencies={dependency_count} open_superseded={open_superseded}")
    finally:
        connection.close()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--jsonl", type=Path, default=DEFAULT_JSONL)
    parser.add_argument("--db", type=Path, default=DEFAULT_DB)
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--sync-db", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    graph = json.loads(GRAPH_PATH.read_text())
    generated = build_records(graph)
    existing = load_jsonl(args.jsonl)
    merged = merge_jsonl(existing, generated)
    validate(graph, generated, merged)
    generated_text = dump_jsonl(generated)
    merged_text = merged_text_preserving_unchanged_lines(args.jsonl, merged)
    if args.check:
        assert GENERATED_PATH.read_text() == generated_text, f"stale generated Beads materialization: {GENERATED_PATH}"
        assert existing == merged, f"stale Beads JSONL semantics: {args.jsonl}"
        if args.sync_db:
            sync_db(args.db, merged)
        print(f"Spec 135 Beads materialization: PASS ({len(generated)} generated, {len(merged)} total)")
        return
    if not args.apply:
        print(f"Would generate {len(generated)} records and merge {len(merged)} total records; pass --apply to write.")
        return
    atomic_write(GENERATED_PATH, generated_text)
    atomic_write(args.jsonl, merged_text)
    if args.sync_db:
        sync_db(args.db, merged)
    print(f"Generated {GENERATED_PATH.relative_to(ROOT)} ({len(generated)} records)")
    print(f"Updated {args.jsonl.relative_to(ROOT)} ({len(merged)} total records)")


if __name__ == "__main__":
    main()
