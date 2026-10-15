"""Spec187 source-authoring cohesion, not runtime/credential/installed proof."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "docs/contracts/spec187"
PROFILE = json.loads((BASE / "release-workset.v1.json").read_text())
REQUIREMENTS = json.loads((BASE / "requirements.v1.json").read_text())
SPEC = (ROOT / PROFILE["spec_ref"]).read_text()
loader = importlib.util.spec_from_file_location("flow_authoring_compiler", ROOT / "scripts/compile-flow-workset.py")
compiler = importlib.util.module_from_spec(loader)
loader.loader.exec_module(compiler)


class FlowAuthoringContract(unittest.TestCase):
    def test_source_is_not_admission(self):
        self.assertTrue(PROFILE["source_authoring_only"])
        self.assertEqual(PROFILE["workset_id_strategy"], "generated_by_existing_Spec149_confirmed_creation")
        self.assertNotIn("run_id", PROFILE)
        self.assertNotIn("workset_id", PROFILE)
        self.assertIn("authoritative daemon is unavailable", SPEC)

    def test_current_full_release_retains_prior_work_and_truthful_zero_goal(self):
        directive = PROFILE["operator_directed_trajectory"]
        self.assertIn("truthful zero", directive["ultimate_goal"])
        self.assertIn("entire Autonomous Agentic Flow State", directive["current_full_release_goal"])
        self.assertIn("unfinished previously authorized", directive["current_full_release_goal"])
        self.assertIn("no silent dropping", directive["carryover_policy"])
        self.assertIn("Pending restoration", directive["live_tracker_update"])
        self.assertIn("next ready authorized work", directive["remaining_repository_work"])
        self.assertIn("without a routine continue prompt", directive["remaining_repository_work"])
        self.assertIn("Mandatory carryover audit", SPEC)
        self.assertIn("Current full release trajectory", SPEC)
        self.assertIn("failed the Pi compaction/session-recovery gate", SPEC)
        r18 = next(r for r in REQUIREMENTS["requirements"] if r["id"] == "FLOW-R18")
        self.assertTrue(any("unfinished previously authorized release obligations" in text for text in r18["acceptance"]))

    def test_exact_issue_boundary_and_no_automatic_program_closure(self):
        self.assertEqual(set(PROFILE["selected_issue_numbers"]),
            {685,660,657,679,683,684,261,680,682,686,299,526,410,417,590,672})
        self.assertEqual(PROFILE["related_program_issue_numbers"], [101,252,253])
        self.assertTrue(set(PROFILE["selected_issue_numbers"]).isdisjoint(PROFILE["related_program_issue_numbers"]))
        self.assertIn("keep the issue open", SPEC)

    def test_required_requirements_are_complete_and_unique(self):
        reqs = REQUIREMENTS["requirements"]
        ids = [r["id"] for r in reqs]
        self.assertEqual(len(ids), len(set(ids)))
        self.assertEqual(ids, [f"FLOW-R{n:02d}" for n in range(1,20)])
        self.assertEqual(PROFILE["completion_contract"]["required_requirement_ids"], ids)
        for row in reqs:
            self.assertTrue(row["owner_refs"])
            self.assertTrue(row["acceptance"])
            self.assertTrue(row["scenarios"])

    def test_all_scoped_tasks_have_real_provider_refs_and_full_contracts(self):
        rows = PROFILE["work_items"]
        self.assertEqual(len(rows), 19)
        self.assertEqual(len({r["work_item_ref"] for r in rows}), len(rows))
        known = set(PROFILE["completion_contract"]["required_requirement_ids"])
        for row in rows:
            self.assertRegex(row["work_item_ref"], r"^focusa-ak5sv\.\d+$")
            self.assertTrue(row["scope"])
            self.assertTrue(row["done"])
            self.assertTrue(set(row["requirement_ids"]) <= known)
            self.assertTrue(row["requirement_ids"])

    def test_literal_source_owners_exist_and_scopes_are_unique(self):
        refs = [p for row in PROFILE["work_items"] for p in row["scope"]]
        refs += [p for row in REQUIREMENTS["requirements"] for p in row["owner_refs"]]
        for ref in refs:
            if ref.startswith(("docs/", "crates/", "apps/", "scripts/", "config/", "release-proof/")):
                self.assertTrue((ROOT / ref).exists(), ref)
        for row in PROFILE["work_items"]:
            self.assertEqual(len(row["scope"]), len(set(row["scope"])))

    def test_dependency_projection_and_requirement_ownership_agree(self):
        rows = {r["step"]: r for r in PROFILE["work_items"]}
        for req in REQUIREMENTS["requirements"]:
            for step in req["steps"]:
                self.assertIn(req["id"], rows[step]["requirement_ids"])
        for row in rows.values():
            self.assertTrue(set(row["deps"]) <= rows.keys())
        graph = compiler.fixture_graph(PROFILE)
        self.assertEqual({(e["from_frame_id"],e["to_frame_id"]) for e in graph["edges"]},
            {(d,r["step"]) for r in rows.values() for d in r["deps"]})
        for edge in graph["edges"]:
            if len(rows[edge["to_frame_id"]]["deps"]) > 1:
                self.assertEqual(edge["join_policy"], "all")

    def test_every_issue_and_requirement_has_a_delivery_path(self):
        issues = {n for r in PROFILE["work_items"] for n in r["issues"]}
        self.assertTrue(set(PROFILE["selected_issue_numbers"]) <= issues)
        self.assertIn("FLOW-15", next(r for r in PROFILE["work_items"] if r["step"]=="FLOW-16")["deps"])
        self.assertIn("FLOW-16", next(r for r in PROFILE["work_items"] if r["step"]=="FLOW-17")["deps"])
        self.assertEqual(set(PROFILE["selected_issue_numbers"]),
            set(next(r for r in PROFILE["work_items"] if r["step"]=="FLOW-16")["issues"]))

    def test_all_scenarios_defined_and_referenced(self):
        scenarios = {s for r in REQUIREMENTS["requirements"] for s in r["scenarios"]}
        self.assertEqual(scenarios, {f"FLOW-T{n:02d}" for n in range(1,26)})
        for scenario in scenarios:
            self.assertIn("| " + scenario + " |", SPEC)

    def test_native_shape_fixture_is_derived_and_cannot_claim_runtime_authority(self):
        fixture = json.loads((BASE / "callgraph.structural-fixture.v1.json").read_text())
        self.assertEqual(fixture, compiler.fixture_graph(PROFILE))
        self.assertEqual(fixture["schema"], "focusa.callgraph.v1")
        self.assertEqual(fixture["created_by"]["authority_kind"], "synthetic_fixture")
        self.assertIn("NEVER ADMIT OR DISPATCH", fixture["description"])
        self.assertEqual(fixture["workpoint_refs"], [])
        self.assertTrue(all("execution_binding" not in row for row in fixture["frames"]))
        self.assertEqual(fixture["entry_frame_ids"], ["FLOW-00"])

    def test_implicit_real_scope_is_rejected(self):
        result = subprocess.run([sys.executable, str(ROOT / "scripts/compile-flow-workset.py")],
            text=True, capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires explicit verified", result.stderr)
        with self.assertRaises(ValueError):
            compiler.compile_graph(PROFILE, "relative", "", "fixture", "fixture")

    def test_project_and_ad_hoc_routes_are_both_goal_directed_and_safe(self):
        row = next(r for r in REQUIREMENTS["requirements"] if r["id"] == "FLOW-R19")
        self.assertEqual(row["issues"], [672])
        text = " ".join(row["acceptance"])
        for invariant in ["working objective", "not goal-less", "inferred candidates", "without a redundant", "exact permitted target", "compaction/restart", "promotion"]:
            self.assertIn(invariant, text)
        route = next(r for r in PROFILE["work_items"] if r["step"] == "FLOW-18")
        self.assertEqual(route["work_item_ref"], "focusa-ak5sv.19")
        for step in ["FLOW-10", "FLOW-11", "FLOW-12", "FLOW-13"]:
            self.assertIn("FLOW-18", next(r for r in PROFILE["work_items"] if r["step"] == step)["deps"])
        self.assertIn("Required session routes", SPEC)

    def test_proof_and_safety_boundaries_are_preserved(self):
        text = " ".join(a for r in REQUIREMENTS["requirements"] for a in r["acceptance"])
        for boundary in ["Recovery codes", "exhausted budget", "advisory", "revocation", "installed", "model context"]:
            self.assertIn(boundary, text)
        kinds = {r["evidence_type"] for r in compiler.fixture_graph(PROFILE)["required_evidence"]}
        self.assertEqual(kinds, {"versioned_contract","producer_test","consumer_test","cross_version_interop","installed_e2e"})


if __name__ == "__main__":
    unittest.main()
