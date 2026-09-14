#!/usr/bin/env python3
"""Fail early when the hash-locked Python test environment is incomplete."""

from __future__ import annotations

from importlib.metadata import version

from jsonschema import Draft202012Validator


if __name__ == "__main__":
    # Constructing the validator forces the import path used by Spec 135/152 gates.
    Draft202012Validator.check_schema({"type": "object"})
    print(f"jsonschema={version('jsonschema')}")
