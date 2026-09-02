#!/usr/bin/env python3
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

SOURCE = Path(__file__).resolve().parents[1] / "scripts" / "generate-release-notes.py"


def run(*args, cwd, env=None, check=True):
    return subprocess.run(
        args,
        cwd=cwd,
        env=env,
        check=check,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


with tempfile.TemporaryDirectory(prefix="focusa-release-notes-preview-") as tmp:
    root = Path(tmp)
    scripts = root / "scripts"
    fake_bin = root / "bin"
    scripts.mkdir()
    fake_bin.mkdir()
    shutil.copy2(SOURCE, scripts / SOURCE.name)
    fake_gh = fake_bin / "gh"
    fake_gh.write_text(
        "#!/bin/sh\n"
        "if [ \"$1 $2\" = \"release view\" ]; then\n"
        "  echo 1970-01-01T00:00:00Z\n"
        "else\n"
        "  echo '[]'\n"
        "fi\n"
    )
    fake_gh.chmod(0o755)
    env = os.environ.copy()
    for key in (
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ):
        env.pop(key, None)
    env["PATH"] = f"{fake_bin}:/usr/bin:/bin"
    env["GITHUB_REPOSITORY"] = "example/focusa"

    run("git", "init", "-q", cwd=root, env=env)
    run("git", "config", "user.name", "Focusa Test", cwd=root, env=env)
    run("git", "config", "user.email", "focusa-test@example.invalid", cwd=root, env=env)
    (root / "release.txt").write_text("baseline\n")
    run("git", "add", "release.txt", cwd=root, env=env)
    run("git", "commit", "-qm", "chore: baseline", cwd=root, env=env)
    run("git", "tag", "v1.0.0", cwd=root, env=env)
    (root / "release.txt").write_text("baseline\npreview change\n")
    run("git", "commit", "-qam", "fix: preview target", cwd=root, env=env)

    preview = run(
        sys.executable,
        str(scripts / SOURCE.name),
        "--tag",
        "v1.0.1",
        "--preview",
        cwd=root,
        env=env,
    ).stdout
    assert "**Commits:** 1" in preview, preview
    assert "**Files:** 1" in preview, preview
    assert "fix: preview target" in preview, preview

    missing_tag = run(
        sys.executable,
        str(scripts / SOURCE.name),
        "--tag",
        "v1.0.1",
        "--output",
        str(root / "notes.md"),
        cwd=root,
        env=env,
        check=False,
    )
    assert missing_tag.returncode != 0, missing_tag.stdout
    assert "does not exist; use --preview" in missing_tag.stderr, missing_tag.stderr

print("release notes pre-tag preview: PASS")
