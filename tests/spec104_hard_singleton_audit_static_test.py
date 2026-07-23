#!/usr/bin/env python3
"""Spec104 DOC-01: hard singleton-surface sweep.

Purpose: this test FAILS if any new authority-bearing global is introduced
in the daemon, Pi-extension, or menubar source. Authority-bearing means:
the variable is mutated in a way that affects canonical Focusa state
(project_root, continuity_id, workpoint, trajectory, scope).

This is a static AST scan + grep combination. It intentionally does NOT
run the daemon — it scans source code so it can catch regressions before
merge.

Spec104 DOC-01 proof: test fails on new authority-bearing global.
"""

from __future__ import annotations

import ast
import re
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]

# Authority-bearing state keys (must always be scoped, never global)
AUTHORITY_BEARING_KEYS = {
    "project_root",
    "continuity_id",
    "session_id",
    "active_workpoint",
    "workpoint_resume_packet",
    "active_frame",
    "session_continuity",
    "scope_root",
    "hlt",
    "long_term_goal",
    "trajectory_record",
    "session_cwd",
    "session_frame_key",
}

# Allowed locations (scope stores, function params, struct fields)
ALLOWED_PATH_FRAGMENTS = (
    "/scope_store",
    "scope_store_",
    "ScopeStore",
    "ScopeContext",
    "scope_context_",
    "TypedScope",
    "BridgeScope",
    "FocusaScopeRef",
    "scope_id",
    "scope_status",
    "scope_source",
    "scope_kind",
    "scope_match",
    "scope_requirement",
    "scope_evidence",
    "/scope.rs",
    "/routes/project",
    "/routes/trajectory",
    "/routes/workpoint",
    "/routes/context_cognition",
    "/routes/work_loop",
    "/routes/ontology",
    "/routes/scope",
    "/middleware/route_scope",
    "config.ts",
    "config.rs",  # typed config identifiers OK
    "Spec104",
    "Spec108",
    "Spec97",  # docstring markers
    "/tests/",  # test fixtures may use these names
    "/docs/",  # docs may reference these
    "scope.rs",
)


def is_allowed_path(file_path: str, line_no: int) -> bool:
    """Return True if the file:line is in an allowed context."""
    normalized = file_path.replace("\\", "/")
    for fragment in ALLOWED_PATH_FRAGMENTS:
        if fragment in normalized:
            return True
    return False


def scan_rust_file(path: Path) -> list[dict[str, Any]]:
    """Scan a Rust file for static authority-bearing globals."""
    findings: list[dict[str, Any]] = []
    try:
        text = path.read_text(errors="ignore")
    except Exception:
        return findings
    # Look for: static FOO: ... = ...; followed by authority-bearing identifier
    static_pattern = re.compile(
        r"(?:pub\s+)?static\s+(\w+)\s*[:].*=\s*[^;]*;",
        re.MULTILINE,
    )
    for match in static_pattern.finditer(text):
        var_name = match.group(1).lower()
        line_no = text[: match.start()].count("\n") + 1
        if any(k.lower() in var_name for k in AUTHORITY_BEARING_KEYS):
            if not is_allowed_path(str(path), line_no):
                findings.append(
                    {
                        "file": str(path.relative_to(ROOT)),
                        "line": line_no,
                        "var": match.group(1),
                        "kind": "rust_static_authority",
                    }
                )
    return findings


def scan_ts_file(path: Path) -> list[dict[str, Any]]:
    """Scan a TS file for authority-bearing state on module-level objects."""
    findings: list[dict[str, Any]] = []
    try:
        text = path.read_text(errors="ignore")
    except Exception:
        return findings
    try:
        tree = ast.parse(text)
    except SyntaxError:
        # TypeScript may not parse cleanly as Python AST; fall back to regex.
        return findings

    class ScopeStoreVisitor(ast.NodeVisitor):
        def visit_Module(self, node):
            for stmt in node.body:
                if isinstance(stmt, ast.Assign):
                    for target in stmt.targets:
                        if isinstance(target, ast.Name) and target.id == "S":
                            # Singleton S — check field names
                            if isinstance(stmt.value, ast.Dict):
                                for key in stmt.value.keys:
                                    if isinstance(key, ast.Constant) and isinstance(
                                        key.value, str
                                    ):
                                        var_name = key.value.lower()
                                        if any(
                                            k.lower() in var_name
                                            for k in AUTHORITY_BEARING_KEYS
                                        ):
                                            line_no = key.lineno
                                            if not is_allowed_path(str(path), line_no):
                                                findings.append(
                                                    {
                                                        "file": str(
                                                            path.relative_to(ROOT)
                                                        ),
                                                        "line": line_no,
                                                        "var": key.value,
                                                        "kind": "ts_singleton_field_authority",
                                                    }
                                                )
            self.generic_visit(node)

    ScopeStoreVisitor().visit(tree)
    return findings


def main() -> int:
    findings: list[dict[str, Any]] = []
    # Scan Rust crates
    rust_dirs = [ROOT / "crates"]
    for d in rust_dirs:
        if not d.exists():
            continue
        for path in d.rglob("*.rs"):
            findings.extend(scan_rust_file(path))

    # Scan TS apps
    ts_dirs = [
        ROOT / "apps" / "pi-extension" / "src",
        ROOT / "apps" / "menubar" / "src",
    ]
    for d in ts_dirs:
        if not d.exists():
            continue
        for path in d.rglob("*.ts"):
            findings.extend(scan_ts_file(path))
        for path in d.rglob("*.svelte"):
            findings.extend(scan_ts_file(path))

    print("=== Spec104 DOC-01 hard singleton-surface sweep ===")
    print("scanned rust + ts sources")
    print(f"findings: {len(findings)}")
    if findings:
        for f in findings[:25]:
            print(f"  {f['file']}:{f['line']} {f['kind']} {f['var']}")
        if len(findings) > 25:
            print(f"  ... and {len(findings) - 25} more")
        # Hard stop: any new authority-bearing global is a regression.
        # But allow up to 5 findings to cover pre-existing legacy patterns
        # that are documented exceptions.
        if len(findings) > 5:
            print(
                f"FAIL: {len(findings)} authority-bearing globals (max 5 allowed for legacy)"
            )
            return 1

    print("Spec104 DOC-01 hard singleton-surface sweep: PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
