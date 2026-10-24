#!/usr/bin/env python3
"""Read-only release Git-administration checks; never repair ownership or locks.

Run as the source owner before expensive gates and again immediately before
committing. A successful check is not a release, CI, or publication receipt.
"""
import argparse
import json
import os
from pathlib import Path
import stat
import subprocess
import sys


def inspect(repo):
    result = {
        "schema": "focusa.release_git_admin_preflight.v1",
        "ok": False,
        "status": "blocked",
        "release_ready": False,
        "mutations": [],
        "failures": [],
    }

    def git(*args):
        return subprocess.check_output(
            ["git", "-C", str(repo), "rev-parse", *args],
            text=True, stderr=subprocess.PIPE, timeout=5,
        ).strip()

    try:
        root = Path(git("--show-toplevel"))
        repo = root
        uid = os.geteuid()
        result.update(project_root=str(root), effective_uid=uid)
        if root.stat().st_uid != uid:
            result["failures"].append({"path": str(root), "reason": "release_user_mismatch"})
        git_dir = Path(git("--absolute-git-dir"))
        common = Path(git("--git-common-dir"))
        if not common.is_absolute():
            common = root / common
        directories = {git_dir, common}
        files = []
        for name in ("COMMIT_EDITMSG", "index", "HEAD", "ORIG_HEAD"):
            path = Path(git("--git-path", name))
            if not path.is_absolute():
                path = root / path
            directories.add(path.parent)
            files.append(path)
        for path in sorted(directories):
            info = path.lstat()
            reason = None
            if not stat.S_ISDIR(info.st_mode):
                reason = "git_admin_directory_not_regular"
            elif info.st_uid != uid:
                reason = "git_admin_owner_mismatch"
            elif not os.access(path, os.W_OK | os.X_OK):
                reason = "git_admin_directory_not_writable"
            if reason:
                result["failures"].append({"path": str(path), "reason": reason})
        for path in files:
            if os.path.lexists(path):
                info = path.lstat()
                reason = None
                if not stat.S_ISREG(info.st_mode):
                    reason = "git_admin_file_not_regular"
                elif info.st_uid != uid:
                    reason = "git_admin_owner_mismatch"
                elif not os.access(path, os.W_OK):
                    reason = "git_admin_file_not_writable"
                if reason:
                    result["failures"].append({"path": str(path), "reason": reason})
            lock = path.with_name(path.name + ".lock")
            if os.path.lexists(lock):
                result["failures"].append({"path": str(lock), "reason": "git_transaction_active"})
        result["git_dir"] = str(git_dir)
    except (OSError, subprocess.SubprocessError) as error:
        result["failures"].append({"reason": "git_admin_inspection_failed", "detail": (getattr(error, "stderr", None) or str(error)).strip()})
    if not result["failures"]:
        result.update(ok=True, status="ready", failure_class=None)
    else:
        result["failure_class"] = "release_git_admin_not_writable"
        result["next_step_hint"] = (
            "Run as the source owner; diagnose only the named administrative paths "
            "through the approved owner-permission recovery route; preserve staged state and locks."
        )
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    args = parser.parse_args()
    result = inspect(args.repo)
    print(json.dumps(result, sort_keys=True))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
