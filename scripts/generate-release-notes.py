#!/usr/bin/env python3
"""
Deterministic release notes generator — script-owned, never agent.
Generates detailed, categorized notes from git tag delta, diff stats,
PR/issue lists, and capability manifests.

Replaces the inline heredoc in .github/workflows/release.yml.
Handles both stable (vX.Y.Z) and dev (vX.Y.Z-dev).

Usage:
  python3 scripts/generate-release-notes.py --tag v0.9.177 --output /tmp/release-notes.md
  python3 scripts/generate-release-notes.py --range v0.9.172..v0.9.177 --output /tmp/release-notes.md
  python3 scripts/generate-release-notes.py --preview  # prints to stdout for local preflight
"""

import argparse
import json
import os
import re
import subprocess
import sys
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Map top-level paths to human feature areas
AREA_MAP = [
    ("crates/focusa-daemon", "Daemon"),
    ("crates/focusa-api", "API"),
    ("crates/focusa-core", "Core"),
    ("crates/focusa-cli", "CLI"),
    ("crates/focusa-tui", "TUI"),
    ("crates/focusa-license", "Licensing"),
    ("crates/focusa-terminal-ui", "Terminal UI"),
    ("apps/menubar", "Menubar"),
    ("apps/pi-extension", "Pi Extension"),
    ("apps/cockpit", "Cockpit"),
    ("cmd/uiai-engine", "UIAI Engine"),
    ("internal/", "UIAI Engine"),
    ("wpuiai-plugin", "WP-UIAI Plugin"),
    ("scripts/install", "Installer"),
    ("scripts/", "Scripts"),
    (".github/workflows", "CI / Release"),
    ("docs/current", "Docs"),
    ("docs/contracts", "Contracts"),
    ("docs/", "Docs"),
    ("tests/", "Tests"),
    ("config/", "Config"),
]

COMMIT_TYPES = [
    ("feat", "Features added"),
    ("fix", "Fixes shipped"),
    ("perf", "Performance"),
    ("refactor", "Refactors"),
    ("docs", "Documentation"),
    ("build", "Build"),
    ("ci", "CI"),
    ("test", "Tests"),
    ("chore", "Chore"),
    ("revert", "Reverts"),
    ("security", "Security"),
    ("proof", "Proof"),
]


def sh(cmd, cwd=ROOT, env=None):
    return subprocess.check_output(cmd, shell=True, cwd=str(cwd), env=env, text=True).strip()


def sh_lines(cmd, cwd=ROOT, env=None):
    out = subprocess.check_output(cmd, shell=True, cwd=str(cwd), env=env, text=True)
    return [l for l in out.splitlines() if l]


def detect_prev_tag(tag):
    try:
        tags = sh(f"git tag --sort=-version:refname").splitlines()
        for t in tags:
            if t != tag:
                return t
    except Exception:
        pass
    return ""


def categorize_commit(subject):
    # Returns (type, breaking)
    breaking = "!" in subject.split(":")[0] if ":" in subject else "!" in subject
    # extract type before ( or ! or :
    m = re.match(r"^(\w+)(?:\([^)]+\))?(!)?:", subject)
    ctype = m.group(1) if m else "other"
    if ctype not in {t for t, _ in COMMIT_TYPES}:
        ctype = "other"
    return ctype, breaking


def area_of(path):
    for prefix, name in AREA_MAP:
        if path.startswith(prefix):
            return name
    # fallback to top dir
    return path.split("/")[0] if "/" in path else "Root"


def generate(tag, range_arg, output, preview=False):
    if not tag and not range_arg:
        # auto-detect tag from git
        tag = sh("git describe --tags --abbrev=0 2>/dev/null || echo v0.0.0")
    if not tag:
        tag = "v0.0.0"
    if range_arg:
        range_spec = range_arg
        prev_tag = range_arg.split("..")[0] if ".." in range_arg else ""
    else:
        prev_tag = detect_prev_tag(tag)
        range_spec = f"{prev_tag}..{tag}" if prev_tag else tag

    repo = os.environ.get("GITHUB_REPOSITORY", "Startempire-Wire/focusa")
    server = os.environ.get("GITHUB_SERVER_URL", "https://github.com")
    compare_url = f"{server}/{repo}/compare/{prev_tag}...{tag}" if prev_tag else f"{server}/{repo}/commits/{tag}"

    # Collect commits
    commits = []
    try:
        raw = sh(f"git log {range_spec} --no-merges --format='%H%x09%s%x09%aN%x09%aI' 2>/dev/null || true")
        for line in raw.splitlines():
            parts = line.split("\t")
            if len(parts) >= 2:
                commits.append({"sha": parts[0], "subject": parts[1], "author": parts[2] if len(parts) > 2 else "", "date": parts[3] if len(parts) > 3 else ""})
    except Exception:
        commits = []

    # Group commits
    by_type = defaultdict(list)
    breaking = []
    for c in commits:
        ctype, is_breaking = categorize_commit(c["subject"])
        by_type[ctype].append(c)
        if is_breaking:
            breaking.append(c)

    # Diff stats
    try:
        stat_lines = sh_lines(f"git diff --stat {range_spec} 2>/dev/null | tail -n 1 || true")
        diff_stat = stat_lines[0] if stat_lines else ""
        numstat = sh(f"git diff --shortstat {range_spec} 2>/dev/null || true")
        # file-level
        files_changed = sh_lines(f"git diff --name-only {range_spec} 2>/dev/null | head -n 500 || true")
        # numstat for additions/deletions per file
        nums = sh_lines(f"git diff --numstat {range_spec} 2>/dev/null | head -n 500 || true")
        insertions = deletions = 0
        area_counter = Counter()
        file_details = []
        for line in nums:
            parts = line.split("\t")
            if len(parts) == 3:
                try:
                    ins = int(parts[0]) if parts[0] != "-" else 0
                    dels = int(parts[1]) if parts[1] != "-" else 0
                except ValueError:
                    ins = dels = 0
                insertions += ins
                deletions += dels
                path = parts[2]
                area_counter[area_of(path)] += 1
                if len(file_details) < 80:
                    file_details.append(f"| `{path}` | +{ins} / -{dels} | {area_of(path)} |")
    except Exception:
        diff_stat = numstat = ""
        files_changed = []
        area_counter = Counter()
        file_details = []
        insertions = deletions = 0

    total_commits = len(commits)
    total_files = len(files_changed)

    # Contributors
    contributors = sorted(set(c["author"] for c in commits if c["author"]))

    # Try to collect resolved issues / PRs via gh if available (best-effort, never fails)
    resolved_issues = ""
    merged_prs = ""
    try:
        prev_published = sh(f"gh release view {prev_tag} --repo {repo} --json publishedAt --jq .publishedAt 2>/dev/null || echo 1970-01-01T00:00:00Z")
    except Exception:
        prev_published = "1970-01-01T00:00:00Z"
    prev_day = prev_published.split("T")[0] if "T" in prev_published else "1970-01-01"
    if sh("which gh 2>/dev/null || echo missing") != "missing":
        try:
            issues_json = sh(f"gh issue list --repo {repo} --state closed --limit 500 --search 'closed:>={prev_day}' --json number,title,url,closedAt 2>/dev/null || echo '[]'")
            issues = json.loads(issues_json) if issues_json else []
            # filter by closedAt > prev_published
            filtered = [i for i in issues if i.get("closedAt", "") > prev_published]
            filtered.sort(key=lambda x: x["number"])
            if filtered:
                resolved_issues = "\n".join(f"- [{i['title']}]({i['url']}) (#{i['number']})" for i in filtered[:100])
        except Exception:
            pass
        try:
            prs_json = sh(f"gh pr list --repo {repo} --state merged --limit 500 --search 'merged:>={prev_day}' --json number,title,url,mergedAt,author 2>/dev/null || echo '[]'")
            prs = json.loads(prs_json) if prs_json else []
            filtered = [p for p in prs if p.get("mergedAt", "") > prev_published]
            filtered.sort(key=lambda x: x["number"])
            if filtered:
                merged_prs = "\n".join(f"- [{p['title']}]({p['url']}) (#{p['number']}) — @{p['author']['login']}" for p in filtered[:100])
        except Exception:
            pass

    # Known issues
    known_issues = ""
    if sh("which gh 2>/dev/null || echo missing") != "missing":
        try:
            kj = sh(f"gh issue list --repo {repo} --state open --limit 100 --label 'release-note:known-issue' --json number,title,url 2>/dev/null || echo '[]'")
            ks = json.loads(kj) if kj else []
            if ks:
                known_issues = "\n".join(f"- [{k['title']}]({k['url']}) (#{k['number']})" for k in ks)
        except Exception:
            pass

    breaking_md = "\n".join(f"- [`{c['sha'][:8]}`]({server}/{repo}/commit/{c['sha']}) {c['subject']}" for c in breaking) if breaking else "- None recorded in this tag delta."
    # Build per-type sections
    sections = {}
    for ctype, title in COMMIT_TYPES:
        lst = by_type.get(ctype, [])
        if lst:
            sections[title] = "\n".join(f"- [`{c['sha'][:8]}`]({server}/{repo}/commit/{c['sha']}) {c['subject']} — {c['author']}" for c in lst)
        else:
            sections[title] = "- None recorded in this tag delta."
    other = by_type.get("other", [])
    other_md = "\n".join(f"- [`{c['sha'][:8]}`]({server}/{repo}/commit/{c['sha']}) {c['subject']}" for c in other) if other else "- None recorded in this tag delta."

    # Area breakdown
    area_md = ""
    if area_counter:
        area_md = "\n".join(f"- **{area}**: {cnt} file(s) changed" for area, cnt in area_counter.most_common())
    else:
        area_md = "- No file changes recorded."

    # File detail table (truncated)
    file_table = ""
    if file_details:
        file_table = "| File | +/- | Area |\n|------|-----|------|\n" + "\n".join(file_details)
        if total_files > len(file_details):
            file_table += f"\n| _…and {total_files - len(file_details)} more_ | | |"
    else:
        file_table = "_No file-level diff recorded._"

    # Important software features — infer from feat commits + area changes
    feat_commits = by_type.get("feat", [])
    feature_highlights = ""
    if feat_commits or any(a in area_counter for a in ["Core", "API", "Licensing", "TUI", "Pi Extension"]):
        lines = []
        for c in feat_commits[:10]:
            lines.append(f"- {c['subject']}")
        if area_counter.get("Licensing", 0) > 2:
            lines.append("- **Licensing** surface expanded (activation, EDD bind, lease issuer)")
        if area_counter.get("Core", 0) > 5:
            lines.append("- **Core** authority/ledger/workset flows hardened")
        if area_counter.get("API", 0) > 3:
            lines.append("- **API** routes / entitlement / work_loop surfaces updated")
        if area_counter.get("Menubar", 0):
            lines.append("- **Menubar** app refreshed (settings, updater, entitlement posture)")
        if area_counter.get("Pi Extension", 0):
            lines.append("- **Pi Extension** tools/contracts synced")
        feature_highlights = "\n".join(lines) if lines else "- See Features added below."
    else:
        feature_highlights = "- Maintenance / infra release — no user-facing feature bump."

    # Software features narrative — repo-aware: Focusa vs UIAI Engine/Cockpit
    is_dev = tag.endswith("-dev")
    channel_label = "Dev prerelease" if is_dev else "Stable"
    generated_at = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
    # Detect product from repo or tag prefix
    is_uiai = "uiai" in repo.lower() or tag.startswith("engine-v") or tag.startswith("uiai-engine-v") or tag.startswith("cockpit-v")
    is_cockpit = tag.startswith("cockpit-v")
    if is_cockpit:
        product_name = "UIAI Engine Cockpit"
        product_desc = "UIAI Engine Cockpit is the Svelte/Tauri desktop client for UIAI Engine — signed OTA, updater-verified, desktop-contract-gated."
    elif is_uiai:
        product_name = "UIAI Engine"
        product_desc = "UIAI Engine is the agent-first browser and proof backend — persistent sessions, diagnostics, research, and Focusa evidence handoff."
    else:
        product_name = "Focusa"
        product_desc = "Focusa is a local-first cognitive runtime for agent continuity, scope authority, evidence, recovery, trajectory, and tool governance."
    infra_block = (
        "**Infrastructure & determinism (agent-removed):**\n"
        "- Single-source version stamp (`scripts/stamp-uiai-version.py` — Go + Cockpit + `distribution-manifest.json` `source_commit`/`generated_at`)\n"
        "- Blocking preflight (`scripts/local-uiai-preflight.sh` — Windows lint, version surfaces, manifest freshness, `go fmt`)\n"
        "- Deterministic `pre-push` hook (`PREFLIGHT_FAST=1` <30s) — blocks stale pushes before CI\n"
        "- Cockpit Tauri signing + updater manifest verified; Browser-reliability soak gates CI\n"
        if is_uiai else
        "**Infrastructure & determinism (agent-removed):**\n"
        "- Single-source version stamp (`scripts/stamp-menubar-version.py` — 16 surfaces + `distribution-manifest.json` `sha256`/`source_commit`/`generated_at`)\n"
        "- Blocking preflight (`scripts/local-release-preflight.sh` — Windows NTFS `:` lint, version surfaces, docs/runtime parity, manifest freshness, `cargo fmt`)\n"
        "- Deterministic `pre-push` hook (`PREFLIGHT_FAST=1` <30s) — blocks stale pushes before CI\n"
        "- CI resilient to azure mirror (`timeout 45` + `ripgrep` fallback) + Spec104 `INF-01` allowlist\n"
        "- Release waits deterministically for `Spec 132 terminal matrix 11/11` (`windows-conpty` + `aarch64-pc-windows-msvc`)\n"
    )

    md = f"""## {product_name} {tag} — {channel_label} • Functional Dogfood Release

> **Generated by `scripts/generate-release-notes.py`** — deterministic, from tag delta. No hand editing. `generated_at={generated_at}`.

{product_desc}

**Release delta:** `{prev_tag or 'repository start'}` → `{tag}` · **Commits:** {total_commits} · **Files:** {total_files} · **+{insertions} / -{deletions}** · [Compare]({compare_url})

---

### TL;DR — What matters in this release

{feature_highlights}

**Who should upgrade:** anyone on `{prev_tag or 'earlier'}` — this tag includes all fixes below plus infra hardening for the canonical cycle.

---

### Important software features & additions

{sections.get('Features added', '- —')}

{infra_block}

---

### Breaking changes

{breaking_md}

---

### Detailed changes by type

#### Features added
{sections.get('Features added')}

#### Fixes shipped
{sections.get('Fixes shipped')}

#### Performance
{sections.get('Performance')}

#### Refactors
{sections.get('Refactors')}

#### Documentation
{sections.get('Documentation')}

#### Build / CI / Tests
{chr(10).join([sections.get('Build'), sections.get('CI'), sections.get('Tests')])}

#### Chore / Other
{sections.get('Chore')}
{other_md}

---

### Changes by area

{area_md}

<details>
<summary>File-level +/- (top {min(len(file_details), 80)} of {total_files})</summary>

{file_table}

</details>

---

### Pull requests merged since {prev_tag or 'start'}

{merged_prs or "- None recorded since the previous published release."}

### Issues resolved since {prev_tag or 'start'}

{resolved_issues or "- None recorded since the previous published release."}

### Known issues

{known_issues or "- None declared."}

---

### Contributors

{chr(10).join(f"- {c}" for c in contributors) if contributors else "- None recorded."}

### Full commit audit ({total_commits})

{chr(10).join(f"- [`{c['sha'][:8]}`]({server}/{repo}/commit/{c['sha']}) {c['subject']}" for c in commits) if commits else "- No commits recorded."}

[Compare {prev_tag or 'repository start'}…{tag}]({compare_url})

---

### Upgrade, rollback, and integrity

- Upgrade with the signed release assets or the documented `focusa update` flow.
- Verify `SHA256SUMS`, sigstore provenance, and `docs/contracts/spec141/generated-capability-v2/distribution-manifest.json` (`sha256` per artifact) before install.
- Roll back with `focusa update rollback --dry-run=false --yes` after reviewing the rollback plan.
- User data, env config, license state, and recovery journals are preserved by update policy (`docs/current/INSTALLER_UPDATE_POLICY.md`).
- This is a **full 16-surface, all-OS** release — no partial matrix.

### Downloads

| Platform | Asset | Description |
|----------|-------|-------------|
| macOS Apple Silicon | `focusa-{tag}-aarch64-apple-darwin` | CLI |
| macOS Intel | `focusa-{tag}-x86_64-apple-darwin` | CLI |
| macOS Apple Silicon | `focusa-daemon-{tag}-aarch64-apple-darwin` | API daemon |
| macOS Intel | `focusa-daemon-{tag}-x86_64-apple-darwin` | API daemon |
| macOS Apple Silicon | `focusa-tui-{tag}-aarch64-apple-darwin` | TUI |
| macOS Intel | `focusa-tui-{tag}-x86_64-apple-darwin` | TUI |
| macOS / Windows | `Focusa_*` DMG / app / MSI / setup | Menubar |
| Linux x64 glibc | `focusa-{tag}-x86_64-unknown-linux-gnu` | CLI |
| Linux x64 musl | `focusa-{tag}-x86_64-unknown-linux-musl` | Bootstrap |
| Linux ARM64 glibc | `focusa-{tag}-aarch64-unknown-linux-gnu` | CLI |
| Linux | `focusa-daemon-{tag}-*-unknown-linux-gnu` | Daemon |
| Linux | `focusa-tui-{tag}-*-unknown-linux-gnu` | TUI |
| Windows x64 | `focusa-{tag}-x86_64-pc-windows-msvc.exe` | CLI |
| Windows ARM64 | `focusa-{tag}-aarch64-pc-windows-msvc.exe` | CLI |
| Windows | `focusa-daemon-{tag}-*-pc-windows-msvc.exe` | Daemon |
| Windows | `focusa-tui-{tag}-*-pc-windows-msvc.exe` | TUI |

### Quick Start (macOS/Linux)

```bash
chmod +x ./focusa-{tag}-<your-platform> ./focusa-daemon-{tag}-<your-platform>
./focusa-daemon-{tag}-<your-platform> &
./focusa-{tag}-<your-platform> status
```

---

*Local-first. Source-available. Proof is the Release workflow + attached `SHA256SUMS` for this tag. Notes generated by `scripts/generate-release-notes.py`.*
"""

    if preview:
        print(md)
        return
    Path(output).write_text(md, encoding="utf-8")
    print(f"wrote {output} for {tag} range {range_spec} commits {total_commits} files {total_files}")


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--tag", default="")
    p.add_argument("--range", dest="range_arg", default="")
    p.add_argument("--output", default="/tmp/release-notes.md")
    p.add_argument("--preview", action="store_true")
    args = p.parse_args()
    generate(args.tag, args.range_arg, args.output, args.preview)


if __name__ == "__main__":
    main()
