#!/usr/bin/env python3
"""Focus State write/read surfaces must preserve ProjectRootKey + WorkstreamKey authority."""

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "crates/focusa-cli/src/commands/focus.rs"
TOOLS = ROOT / "apps/pi-extension/src/tools.ts"
STATE = ROOT / "apps/pi-extension/src/state.ts"
COMPACTION = ROOT / "apps/pi-extension/src/compaction.ts"
SESSION = ROOT / "apps/pi-extension/src/session.ts"
MENUBAR = ROOT / "apps/menubar/src/routes/+page.svelte"
API = ROOT / "crates/focusa-api/src/routes/focus.rs"


def fail(msg: str) -> None:
    print(f"✗ FAIL: {msg}")
    sys.exit(1)


def require(path: Path, pattern: str, msg: str) -> None:
    text = path.read_text()
    if not re.search(pattern, text, re.S):
        fail(msg)


def main() -> None:
    require(
        API,
        r"focus_update_requires_frame_id_or_project_root_plus_continuity_id",
        "API must fail closed for unscoped focus/update",
    )
    require(
        API,
        r"FocusFramePushed[\s\S]*project_root:\s*Some\(project_root\)[\s\S]*continuity_id:\s*Some\(continuity_id\)",
        "API push must persist project_root+continuity_id",
    )

    require(
        CLI,
        r"Push[\s\S]*project_root:\s*Option<String>[\s\S]*continuity_id:\s*Option<String>",
        "CLI focus push must expose project_root+continuity_id flags",
    )
    require(
        CLI,
        r"Update[\s\S]*frame_id:\s*Option<String>[\s\S]*project_root:\s*Option<String>[\s\S]*continuity_id:\s*Option<String>",
        "CLI focus update must expose frame/project/continuity flags",
    )
    require(
        CLI,
        r"/v1/focus/update[\s\S]*\"frame_id\":\s*frame_id[\s\S]*\"project_root\":\s*project_root[\s\S]*\"continuity_id\":\s*continuity_id",
        "CLI focus update must send scope fields",
    )

    require(
        STATE,
        r"selectExistingBeadsIssueIdForFocusFrame",
        "Pi frame creation must select a real project bead id",
    )
    require(
        STATE,
        r"pi_frame_creation_blocked_missing_beads_issue",
        "Pi frame creation must fail closed when no project bead exists",
    )
    require(
        STATE,
        r"/focus/push[\s\S]*beads_issue_id:\s*beadsIssueId[\s\S]*project_root:\s*cwd[\s\S]*continuity_id:\s*continuityId",
        "Pi focus frame push must send real bead/project/continuity scope",
    )

    require(
        TOOLS,
        r"focus_update_requires_safe_project_root_and_continuity_id",
        "Pi durable slot writes must reject missing safe scope before /focus/update",
    )
    require(
        TOOLS,
        r"/focus/update[\s\S]*project_root:\s*projectRoot[\s\S]*continuity_id:\s*continuityId",
        "Pi durable slot writes must send project_root+continuity_id",
    )
    require(
        COMPACTION,
        r"/focus/update[\s\S]*project_root:\s*normalizeProjectRoot[\s\S]*continuity_id:\s*ensureContinuityId",
        "Pi compaction focus updates must send scope",
    )
    require(
        SESSION,
        r"/focus/update[\s\S]*project_root:\s*normalizeProjectRoot[\s\S]*continuity_id:\s*ensureContinuityId",
        "Pi session/fork focus updates must send scope",
    )

    require(
        MENUBAR,
        r"/v1/project/identity[\s\S]*scopedQuery",
        "Menubar must derive scoped query from ProjectIdentity",
    )
    require(
        MENUBAR,
        r"scopedSuffix\s*=\s*scopedQuery \? `&\$\{scopedQuery\}`",
        "Menubar must derive scoped query suffix for existing query strings",
    )
    require(
        MENUBAR,
        r"scopedPathSuffix\s*=\s*scopedQuery \? `\?\$\{scopedQuery\}`",
        "Menubar must derive scoped query suffix for bare paths",
    )
    require(
        MENUBAR,
        r"/v1/focus/frame/current\?\$\{scopedQuery\}",
        "Menubar must read scoped current Focus frame",
    )
    require(
        MENUBAR,
        r"/v1/trajectory/view\?mode=summary\$\{scopedSuffix\}",
        "Menubar must read scoped trajectory",
    )
    require(
        MENUBAR,
        r"/v1/workpoint/current\$\{scopedPathSuffix\}",
        "Menubar must read scoped workpoint",
    )

    print(
        "✓ PASS: Focus State scope is preserved across API, CLI, Pi extension, and menubar surfaces"
    )


if __name__ == "__main__":
    main()
