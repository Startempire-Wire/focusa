#!/usr/bin/env python3
"""Regression tests for JSON-compatible YAML and typed malformed input."""

from __future__ import annotations

import json
from pathlib import Path
import tempfile

from structured_contract_loader import StructuredContractError, load_contract_mapping

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    real_yaml = load_contract_mapping(
        ROOT / "docs/contracts/spec138a-normative-source-coverage.v1.yaml"
    )
    json_syntax_yaml = load_contract_mapping(
        ROOT / "docs/contracts/spec137a-applicability-matrix.v1.yaml"
    )
    assert real_yaml["schema"] == "focusa.spec138a_normative_source_coverage.v2"
    assert json_syntax_yaml["schema"] == "focusa.spec137a_applicability_matrix.v1"

    with tempfile.TemporaryDirectory(prefix="focusa-contract-loader-") as raw:
        root = Path(raw)
        yaml_path = root / "real-yaml.v1.yaml"
        yaml_path.write_text("schema: example.v1\nitems:\n  - one\n", encoding="utf-8")
        assert load_contract_mapping(yaml_path) == {
            "schema": "example.v1",
            "items": ["one"],
        }

        json_yaml_path = root / "json-syntax.v1.yaml"
        json_yaml_path.write_text(
            json.dumps({"schema": "example.v1", "items": ["one"]}),
            encoding="utf-8",
        )
        assert load_contract_mapping(json_yaml_path) == load_contract_mapping(yaml_path)

        malformed_path = root / "malformed.v1.yaml"
        malformed_path.write_text("schema: [unterminated\n", encoding="utf-8")
        try:
            load_contract_mapping(malformed_path)
        except StructuredContractError as error:
            assert error.path == malformed_path
            assert error.code == "YAML_PARSE_ERROR"
            assert str(malformed_path) in str(error)
            assert "YAML_PARSE_ERROR" in str(error)
        else:
            raise AssertionError("malformed YAML must fail with a typed path-bound error")

        list_path = root / "list-root.v1.yaml"
        list_path.write_text("- not\n- a mapping\n", encoding="utf-8")
        try:
            load_contract_mapping(list_path)
        except StructuredContractError as error:
            assert error.code == "CONTRACT_ROOT_NOT_MAPPING"
        else:
            raise AssertionError("non-mapping contract root must fail")

    print("structured contract loader: PASS")


if __name__ == "__main__":
    main()
