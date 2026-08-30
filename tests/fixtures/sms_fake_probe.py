#!/usr/bin/env python3
import argparse
from pathlib import Path
parser = argparse.ArgumentParser()
parser.add_argument("--profile", type=Path, required=True)
parser.add_argument("--port", required=True)
args = parser.parse_args()
marker = args.profile / "connector.ready"
raise SystemExit(0 if marker.exists() and marker.read_text() == args.port else 1)
