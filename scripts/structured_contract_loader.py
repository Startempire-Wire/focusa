#!/usr/bin/env python3
"""Canonical format-aware loader for documentation contract tooling."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml


class StructuredContractError(ValueError):
    """Typed, path-bound parse/type failure for a structured contract."""

    def __init__(self, path: Path, code: str, reason: str) -> None:
        self.path = path
        self.code = code
        self.reason = reason
        super().__init__(f"{path}: {code}: {reason}")


def load_contract_mapping(path: Path) -> dict[str, Any]:
    """Load JSON-syntax or ordinary YAML through the canonical YAML parser."""

    try:
        value = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, yaml.YAMLError) as error:
        raise StructuredContractError(path, "YAML_PARSE_ERROR", str(error)) from error
    if not isinstance(value, dict):
        raise StructuredContractError(
            path,
            "CONTRACT_ROOT_NOT_MAPPING",
            f"expected mapping, got {type(value).__name__}",
        )
    return value
