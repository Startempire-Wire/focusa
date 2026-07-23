#!/usr/bin/env python3
"""Compatibility wrapper for Spec 122 proactive self-heal.

The old auto-heal implementation was a passive audit mirror. Keep this filename
for existing workflow callers, but route behavior to propose-system-fix.py so
self-heal only writes thresholded system-fix rows and avoids noisy commits.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


def main() -> int:
    script = Path(__file__).resolve().with_name("propose-system-fix.py")
    return subprocess.call([sys.executable, str(script), *sys.argv[1:]])


if __name__ == "__main__":
    raise SystemExit(main())
