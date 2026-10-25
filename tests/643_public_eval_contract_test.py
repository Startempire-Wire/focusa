#!/usr/bin/env python3
"""Exact public contracts; schema conformance is not full browser acceptance."""
import copy
import importlib.util
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec135/generated-contract-v1"
spec = importlib.util.spec_from_file_location("public_eval_projection", ROOT / "scripts/redact-spec135-eval-artifacts.py")
projection = importlib.util.module_from_spec(spec)
spec.loader.exec_module(projection)
FULL_ID = "uiai.focusa_ui_eval_result.v1"
SUMMARY_ID = projection.COMPACT_SUMMARY_SCHEMA


class PublicEvalContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.schemas = {key: json.loads((CONTRACT / f"{key}.schema.json").read_text()) for key in [FULL_ID, SUMMARY_ID]}
        cls.validators = {}
        for key, schema in cls.schemas.items():
            Draft202012Validator.check_schema(schema)
            cls.validators[key] = Draft202012Validator(schema, format_checker=FormatChecker())
        cls.documents = [(p, json.loads(p.read_text())) for p in sorted(CONTRACT.glob("uiai-eval*.result.json"))]
        cls.summaries = [d for _, d in cls.documents if d["schema"] == SUMMARY_ID]
        cls.full = [d for _, d in cls.documents if d["schema"] == FULL_ID]

    def test_every_public_fixture_matches_its_exact_declared_schema(self):
        self.assertTrue(self.documents)
        self.assertEqual(len(self.summaries), 5)
        self.assertTrue(self.full)
        for path, document in self.documents:
            with self.subTest(path=path.name):
                self.assertIn(document["schema"], self.validators)
                self.validators[document["schema"]].validate(document)

    def test_public_projection_is_idempotent_and_keeps_reported_history_unverified(self):
        for _, document in self.documents:
            projected = projection.public_projection(document, self.schemas[FULL_ID])
            self.assertEqual(projected, document)
            self.assertEqual(projection.public_projection(projected, self.schemas[FULL_ID]), projected)
        for document in self.summaries:
            self.assertEqual(document["source_schema"], FULL_ID)
            self.assertEqual(document["reported_status"], "passed")
            self.assertEqual(document["verification_status"], "incomplete")
            self.assertIs(document["closure_eligible"], False)
            for step in document["step_results"]:
                self.assertEqual(step["status"], "unverified")
                self.assertIn(step["reported_status"], ["passed", "failed", "blocked", "skipped"])

    def test_summary_is_never_full_evidence_even_if_relabelled(self):
        for document in self.summaries:
            self.assertFalse(self.validators[FULL_ID].is_valid(document))
            relabelled = copy.deepcopy(document)
            relabelled["schema"] = FULL_ID
            self.assertFalse(self.validators[FULL_ID].is_valid(relabelled))
            eligible = copy.deepcopy(document)
            eligible["closure_eligible"] = True
            self.assertFalse(self.validators[SUMMARY_ID].is_valid(eligible))

    def test_full_contract_is_not_reinterpreted_as_a_compact_summary(self):
        for document in self.full:
            self.assertFalse(self.validators[SUMMARY_ID].is_valid(document))
            relabelled = copy.deepcopy(document)
            relabelled["schema"] = SUMMARY_ID
            self.assertFalse(self.validators[SUMMARY_ID].is_valid(relabelled))
            self.assertEqual(projection.public_projection(document, self.schemas[FULL_ID])["schema"], FULL_ID)

    def test_legacy_responsive_classification_preserves_reported_measurement(self):
        source = next(copy.deepcopy(d) for d in self.full if any(
            x.get("summary", "").startswith("reported_category=responsive; ") for x in d["diagnostics"]))
        diagnostic = next(x for x in source["diagnostics"] if x["summary"].startswith("reported_category=responsive; "))
        diagnostic["category"] = "responsive"
        diagnostic["summary"] = diagnostic["summary"].removeprefix("reported_category=responsive; ")
        original = diagnostic["summary"]
        normalized = projection.public_projection(source, self.schemas[FULL_ID])
        self.validators[FULL_ID].validate(normalized)
        recovered = next(x for x in normalized["diagnostics"] if x["summary"].endswith(original))
        self.assertEqual(recovered["category"], "visual")
        self.assertEqual(recovered["summary"], "reported_category=responsive; " + original)
        self.assertEqual(normalized["status"], source["status"])

    def test_only_legacy_compact_declarations_are_migrated(self):
        for summary in self.summaries:
            legacy = copy.deepcopy(summary)
            legacy["schema"] = legacy.pop("source_schema")
            legacy["status"] = legacy.pop("reported_status")
            for key in ["verification_status", "closure_eligible"]:
                legacy.pop(key)
            for step in legacy["step_results"]:
                step["status"] = step.pop("reported_status")
            self.assertEqual(projection.public_projection(legacy, self.schemas[FULL_ID]), summary)
        malformed_full = copy.deepcopy(self.full[0])
        malformed_full.pop("diagnostics")
        projected = projection.public_projection(malformed_full, self.schemas[FULL_ID])
        self.assertEqual(projected["schema"], FULL_ID)
        self.assertFalse(self.validators[FULL_ID].is_valid(projected))


if __name__ == "__main__":
    unittest.main()
