#!/usr/bin/env python3
"""Compute the next Focusa release version from conventional commits.

Implements the release strategy in docs/release-strategy.md:

    SemVer        vMAJOR.MINOR.PATCH
    pre-1.0       MAJOR == 0 -> MINOR is the breaking slot
    fix:...       -> PATCH  (patch lane: security/critical)
    feat:...      -> MINOR  (minor lane: batched features)
    BREAKING      -> MINOR (0.x) or MAJOR (>=1.x)

Exit codes:
    0  policy satisfied (advisory warnings only)
    1  hard violation: malformed tag, non-monotonic tag, or breaking change
       not reflected in the required bump
"""

import argparse
import json
import re
import subprocess
import sys

KNOWN_TYPES = {
    "feat", "fix", "docs", "test", "refactor", "perf", "build",
    "ci", "chore", "revert", "proof", "merge",
}

SUBJECT_RE = re.compile(r"^(?P<type>[a-z]+)(\([^)]*\))?(?P<bang>!)?:")
BREAKING_TRAILER = re.compile(r"^BREAKING[ -]CHANGE\s*:", re.MULTILINE)


def run_git(repo, *args):
    return subprocess.run(
        ["git", "-C", repo, *args],
        capture_output=True,
        text=True,
        check=True,
    ).stdout


SEMVER_TAG = re.compile(
    r"^v(?P<major>0|[1-9][0-9]*)\.(?P<minor>0|[1-9][0-9]*)\."
    r"(?P<patch>0|[1-9][0-9]*)(?:-(?P<suffix>[0-9A-Za-z][0-9A-Za-z.-]*))?$"
)


def parse_tag(tag):
    """Parse 'vMAJOR.MINOR.PATCH(-channel-suffix)?' -> (major, minor, patch) or None.

    Channel suffixes (-dev, -rc, -preview) are accepted: they are the canonical
    dev/preview release tags produced by scripts/select-release-version.py.
    """
    m = SEMVER_TAG.match(tag.strip())
    if not m:
        return None
    return tuple(int(g) for g in (m.group("major"), m.group("minor"), m.group("patch")))


def channel_rank(tag):
    """Ordering rank for release channels: dev < rc < preview < stable."""
    m = SEMVER_TAG.match(tag.strip())
    if not m or not m.group("suffix"):
        return 3
    return {"dev": 0, "rc": 1, "preview": 2}.get(m.group("suffix").split(".", 1)[0], 2)


def tag_key(tag):
    """Monotonic ordering key for a tag (numeric part then channel rank)."""
    parsed = parse_tag(tag)
    if parsed is None:
        return None
    return (*parsed, channel_rank(tag))


def bump(tag_tuple, level):
    major, minor, patch = tag_tuple
    if level == "major":
        return (major + 1, 0, 0)
    if level == "minor":
        return (major, minor + 1, 0)
    return (major, minor, patch + 1)


def fmt_tag(tag_tuple):
    return "v%d.%d.%d" % tag_tuple


def latest_tag(repo, ref="HEAD", exclude=None):
    """Newest v* tag reachable from ref (excluding `exclude`), or None."""
    try:
        out = run_git(repo, "tag", "--merged", ref, "--sort=-version:refname")
    except subprocess.CalledProcessError:
        return None
    for tag in out.splitlines():
        tag = tag.strip()
        if tag == exclude:
            continue
        if parse_tag(tag):
            return tag
    return None


def classify(subject, body):
    """Return ('breaking' | 'minor' | 'patch' | 'unknown', reason)."""
    if BREAKING_TRAILER.search(body):
        return "breaking", "BREAKING CHANGE trailer in body"
    m = SUBJECT_RE.match(subject)
    if not m:
        return "unknown", "not a conventional commit subject"
    if m.group("bang"):
        return "breaking", "bang (!) after type/scope"
    if m.group("type") == "feat":
        return "minor", "feat commit"
    if m.group("type") in KNOWN_TYPES:
        return "patch", "non-feature conventional commit"
    return "unknown", "unrecognized type: %s" % m.group("type")


def commits_since(repo, since):
    """Yield (subject, body) for commits after `since` (tag or commit).

    Uses %x1e as a record separator so multi-paragraph commit messages
    (subject + body) are not confused with separate commits.
    """
    try:
        out = run_git(repo, "log", "--no-merges", "--format=%x1e%B", "%s..HEAD" % since)
    except subprocess.CalledProcessError:
        return
    for raw in out.split("\x1e"):
        raw = raw.strip()
        if not raw:
            continue
        lines = raw.splitlines()
        subject = lines[0].strip()
        body = "\n".join(lines[1:])
        yield subject, body


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--repo", default=".")
    ap.add_argument("--tag", help="tag under check (e.g. v0.9.153); else suggest next")
    ap.add_argument("--last-tag", help="override last tag detection")
    ap.add_argument("--json", action="store_true", dest="as_json")
    args = ap.parse_args()

    violations, warnings = [], []
    counts = {"breaking": 0, "minor": 0, "patch": 0, "unknown": 0}
    examples = {"breaking": [], "minor": [], "unknown": []}

    last_tag = args.last_tag or latest_tag(args.repo, exclude=args.tag)

    if last_tag:
        for subject, body in commits_since(args.repo, last_tag):
            kind, why = classify(subject, body)
            counts[kind] += 1
            if kind in examples and len(examples[kind]) < 5:
                examples[kind].append(subject[:100])
            if kind == "unknown":
                warnings.append("unrecognized commit subject: %s" % subject[:100])

    current = parse_tag(args.tag) if args.tag else None

    if args.tag:
        if not current:
            violations.append("malformed tag %r: expected vMAJOR.MINOR.PATCH" % args.tag)
        elif not last_tag:
            warnings.append("no prior v* tag found; shape-only check for %s" % args.tag)
        else:
            last = tag_key(last_tag)
            current_key = tag_key(args.tag)
            if last is None:
                violations.append("cannot parse last tag %r" % last_tag)
            elif current_key is None:
                violations.append("malformed tag %r: expected vMAJOR.MINOR.PATCH" % args.tag)
            elif current_key <= last:
                violations.append(
                    "non-monotonic tag: %s is not newer than %s" % (args.tag, last_tag))
            else:
                required = "minor" if last[0] == 0 else "major"
                actual = "major" if current_key[0] > last[0] else (
                    "minor" if current_key[1] > last[1] else "patch")
                if counts["breaking"] and actual != required:
                    violations.append(
                        "%d breaking change(s) since %s require a %s bump, got %s (%s)"
                        % (counts["breaking"], last_tag, required, actual, args.tag))
                if counts["minor"] and actual == "patch":
                    warnings.append(
                        "%d feat commit(s) since %s ride a patch bump; feature material "
                        "belongs on the minor lane (0.10.0). Advisory until the minor "
                        "lane is adopted." % (counts["minor"], last_tag))

    if current:
        suggested = fmt_tag(bump(current, "patch"))
        required_bump = "patch"
    else:
        base = parse_tag(last_tag) if last_tag else (0, 9, 0)
        required_bump = "minor" if (counts["breaking"] and base[0] == 0) else (
            "major" if counts["breaking"] else "patch")
        suggested = fmt_tag(bump(base, required_bump))

    result = {
        "policy": "docs/release-strategy.md",
        "tag": args.tag,
        "last_tag": last_tag,
        "counts": counts,
        "examples": examples,
        "required_bump": required_bump,
        "suggested_next": suggested,
        "violations": violations,
        "warnings": warnings,
    }

    if args.as_json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print("last tag        : %s" % (last_tag or "(none)"))
        print("commits         : %s" % ", ".join("%s=%d" % kv for kv in counts.items()))
        print("required bump   : %s" % required_bump)
        print("suggested next  : %s" % suggested)
        for w in warnings:
            print("WARN: %s" % w)
        for v in violations:
            print("VIOLATION: %s" % v)

    return 1 if violations else 0


if __name__ == "__main__":
    sys.exit(main())
