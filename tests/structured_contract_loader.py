#!/usr/bin/env python3
"""Test import adapter for the canonical documentation contract loader."""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from scripts.structured_contract_loader import (  # noqa: E402
    StructuredContractError,
    load_contract_mapping,
)

__all__ = ["StructuredContractError", "load_contract_mapping"]
