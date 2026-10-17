#!/usr/bin/env python3
"""Independent REST-contract consumer, including real native CLI interchange."""
import copy
import hashlib
import struct
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "docs/contracts/spec135/generated-contract-v1"
OPENAPI = json.loads((BASE / "openapi-3.0.3.json").read_text())
REGISTRY = json.loads((BASE / "operation-registry.json").read_text())


def json_schema(identifier):
    def convert(value):
        if isinstance(value, list):
            return [convert(child) for child in value]
        if not isinstance(value, dict):
            return value
        result = {key: convert(child) for key, child in value.items() if key != "nullable"}
        if "$ref" in result:
            result["$ref"] = result["$ref"].replace("#/components/schemas/", "#/$defs/")
        return {"anyOf": [result, {"type": "null"}]} if value.get("nullable") else result
    components = OPENAPI["components"]["schemas"]
    name = identifier.replace(".", "_")
    result = convert(components[name])
    result["$defs"] = {key: convert(value) for key, value in components.items() if value.get("title", "").startswith("focusa.procedure")}
    Draft202012Validator.check_schema(result)
    return Draft202012Validator(result)


def definition():
    return {"schema": "focusa.procedure.v1", "procedure_id": "digest-fixture", "version": 1,
            "steps": [{"step_id": "a", "kind": "shell", "effect_class": "local", "require_confirmation": False,
                       "rollback_step": None, "command": "run a"}]}


class ProcedureContracts(unittest.TestCase):
    def test_native_dependency_contract_and_independent_owner_snapshot(self):
        binary = os.environ.get("TEST_NATIVE_FOCUSA_BIN")
        if not binary:
            self.skipTest("explicit native binary required")
        request = {"requirements": [{"role_ref": "integrity", "operation_ref": "focusa.procedure.preview", "operation_version": "1.0.0", "input_schema_ref": "focusa.procedure.v1", "output_schema_ref": "focusa.procedure_preview.v1", "required_policy_refs": ["read:*"]}]}
        json_schema("focusa.procedure_dependency_request.v1").validate(request)
        digest = hashlib.sha256(b"focusa.procedure_owner_contracts.digest.v1\0")
        for artifact in [BASE / "operation-registry.json", BASE / "openapi-3.0.3.json"]:
            data = artifact.read_bytes(); digest.update(struct.pack(">Q", len(data))); digest.update(data)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "requirements.json"; path.write_text(json.dumps(request))
            output = subprocess.run([binary, "--json", "procedure", "resolve", "--requirements", str(path)], check=True, text=True, capture_output=True)
        result = json.loads(output.stdout)
        json_schema("focusa.procedure_dependencies.v1").validate(result)
        self.assertEqual(result["owner_contract_digest"], "sha256:" + digest.hexdigest())
        self.assertTrue(result["contract_compatible"])
        self.assertFalse(result["canonical"] or result["permission_verified"] or result["execution_authorized"])

    def test_exact_registered_operations_are_nonexecuting_reads(self):
        by_id = {entry["operation_id"]: entry for entry in REGISTRY["operations"]}
        self.assertEqual(len(by_id), REGISTRY["operation_count"])
        for name, path, request, response in [
            ("preview", "/v1/procedures/preview", "focusa.procedure.v1", "focusa.procedure_preview.v1"),
            ("verify_preview", "/v1/procedures/verify-preview", "focusa.procedure_verify.request.v1", "focusa.procedure_verification.v1"),
            ("resolve_dependencies", "/v1/procedures/resolve-dependencies", "focusa.procedure_dependency_request.v1", "focusa.procedure_dependencies.v1")]:
            entry = by_id["focusa.procedure." + name]
            self.assertEqual(entry["path"], path)
            self.assertEqual(entry["control"]["mode"], "read")
            self.assertFalse(entry["control"]["receipt_required"])
            self.assertEqual(entry["contracts"]["error_schema_ref"], "focusa.tool_result.v1")
            self.assertEqual(entry["side_effect_class"], "none")
            self.assertFalse(entry["scope"]["project_scoped"])
            self.assertEqual(entry["request_schema_ref"], request)
            self.assertEqual(entry["response_schema_ref"], response)
            self.assertEqual(OPENAPI["paths"][path]["post"]["operationId"], entry["operation_id"])

    def test_undeclared_input_meaning_and_invalid_versions_are_rejected(self):
        validator = json_schema("focusa.procedure.v1")
        validator.validate(definition())
        for kind in ["unknown", "version", "blank"]:
            value = definition()
            if kind == "unknown": value["steps"][0]["unregistered_policy_override"] = True
            elif kind == "version": value["version"] = 0
            else: value["steps"][0]["command"] = " "
            self.assertFalse(validator.is_valid(value))

    def test_independent_v2_golden_contract_and_authority_rejection(self):
        preview = {"schema": "focusa.procedure_preview.v1", "digest_schema": "focusa.procedure-plan.digest.v2",
                   "canonical": False, "execution_authorized": False,
                   "plan": {"procedure_id": "digest-fixture", "version": 1,
                            "plan_steps": [{"step_id": "a", "effect_class": "local", "confirmation_required": False,
                                            "handoff_required": False, "rollback_of": None}],
                            "rollback_order": [], "digest": "sha256:5f068c6abb4a2d2f67231fcda8aa108e05cc31aa5da1b6922177fd0af6290cec"}}
        validator = json_schema("focusa.procedure_preview.v1")
        validator.validate(preview)
        json_schema("focusa.procedure_verify.request.v1").validate({"definition": definition(), "preview": preview})
        for field, value in [("canonical", True), ("execution_authorized", True), ("digest_schema", "legacy")]:
            changed = copy.deepcopy(preview); changed[field] = value
            self.assertFalse(validator.is_valid(changed))

    def test_real_cli_serialization_matches_http_contracts(self):
        binary = os.environ.get("TEST_NATIVE_FOCUSA_BIN")
        if not binary:
            self.skipTest("explicit native source binary supplied during integration qualification")
        with tempfile.TemporaryDirectory(prefix="procedure-api-consumer-") as directory:
            source = Path(directory) / "source.json"; source.write_text(json.dumps(definition()))
            result = subprocess.run([binary, "--json", "procedure", "compile", "--definition", str(source)], capture_output=True, text=True, check=True, timeout=15)
            preview = json.loads(result.stdout)
            json_schema("focusa.procedure_preview.v1").validate(preview)
            document = Path(directory) / "preview.json"; document.write_text(json.dumps(preview))
            result = subprocess.run([binary, "--json", "procedure", "verify", "--definition", str(source), "--plan", str(document)], capture_output=True, text=True, check=True, timeout=15)
            verification = json.loads(result.stdout)
            json_schema("focusa.procedure_verification.v1").validate(verification)
            self.assertTrue(verification["verified"])
            self.assertFalse(verification["execution_authorized"])


if __name__ == "__main__":
    unittest.main()
