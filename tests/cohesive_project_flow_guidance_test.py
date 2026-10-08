#!/usr/bin/env python3
"""Focused producer and generated-consumer clarity checks, not runtime acceptance."""
import importlib.util
import json
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('skills', ROOT/'scripts/generate-agent-skills.py')
GEN=importlib.util.module_from_spec(spec)
spec.loader.exec_module(GEN)
REG=json.loads((ROOT/'config/agent-skills-v2.json').read_text())


class CohesiveProjectFlowTests(unittest.TestCase):
    def test_all_registered_runbooks_link_one_journey(self):
        for skill in REG['skills']:
            with self.subTest(skill=skill['name']):
                body=GEN.runbook_body(skill)
                self.assertIn('02-focusa-cohesive-project-flow.md',body)
                self.assertNotIn('## Minimal path',body)
                self.assertNotIn('## Dependency graph',body)
                self.assertIn('not define execution dependencies',body)

    def test_generated_core_does_not_mandate_mutations(self):
        for skill in REG['skills']:
            body=GEN.skill_body(skill)
            self.assertNotIn('## Required sequence',body)
            self.assertIn('not a mandatory sequence',body)
            self.assertIn('A rejected operation is not a stopped mission',body)
            self.assertIn('never fabricate admission',body)

    def test_resume_provenance_recovery_is_actionable(self):
        for skill in REG['skills']:
            body=GEN.runbook_body(skill)
            self.assertIn('omit `current_ask`',body)
            self.assertIn('resume_evaluated_different_ask',body)
            self.assertIn('not a permission override',body)

    def test_bootstrap_genesis_are_available_not_compulsory(self):
        skill=next(row for row in REG['skills'] if row['name']=='focusa-agent-bootstrap')
        self.assertIn('focusa_project_bootstrap',skill['tools'])
        self.assertIn('focusa_project_genesis',skill['tools'])
        self.assertIn('never re-onboard',' '.join(skill['runbook_notes']))

    def test_no_duplicate_tool_inventory(self):
        for skill in REG['skills']:
            self.assertEqual(len(skill['tools']),len(set(skill['tools'])),skill['name'])

    def test_restore_is_conditional(self):
        body=(ROOT/'docs/agent/02-focusa-cohesive-project-flow.md').read_text()
        self.assertIn('Snapshot restore, rollback',body)
        self.assertIn('never mandatory steps',body)

    def test_scope_and_installed_truth_are_explicit(self):
        guide=(ROOT/'docs/agent/02-focusa-cohesive-project-flow.md').read_text()
        for fragment in ['WorkstreamId','AttachmentKey','does not activate capabilities',
                         'whole-outcome settlement','newly discovered in-scope dependency',
                         'unrun scenarios remain explicitly unverified']:
            self.assertIn(fragment,guide)
        index=(ROOT/'docs/agent/01-focusa-agent-docs-index.md').read_text()
        self.assertNotIn('Exact authority is `project_root + continuity_id`',index)
        self.assertIn('02-focusa-cohesive-project-flow.md',index)

    def test_every_tool_page_references_the_same_journey(self):
        registry=json.loads((ROOT/'docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json').read_text())
        for descriptor in registry['descriptors']:
            page=(ROOT/descriptor['docs_ref']).read_text()
            self.assertIn('02-focusa-cohesive-project-flow.md',page)
            self.assertIn('supported scoped recovery',page)
        resume=(ROOT/'docs/focusa-tools/tools/focusa_workpoint_resume.md').read_text()
        self.assertIn('override must match it exactly',resume)
        self.assertIn('not dispatch admission',resume)

    def test_entry_documents_do_not_conflict_with_the_journey(self):
        for ref in ['README.md','docs/current/GOLDEN_WORKFLOW.md','docs/current/AUTHORITY_MODEL.md',
                    'docs/current/AGENT_ADAPTER_CONTRACT.md','docs/current/NON_PI_AGENT_FOCUSA_USAGE.md','docs/AGENTS.md']:
            text=(ROOT/ref).read_text()
            self.assertIn('02-focusa-cohesive-project-flow.md',text,ref)
            self.assertNotIn('same-continuity/different-session activity remains one Workstream',text)
        instructions=(ROOT/'docs/AGENTS.md').read_text()
        self.assertNotIn('1. Pause\n2. Surface candidate\n3. Await instruction',instructions)
        self.assertIn('Resume the interrupted admitted action automatically',instructions)
        self.assertIn('Never invent admission',instructions)
        golden=(ROOT/'docs/current/GOLDEN_WORKFLOW.md').read_text()
        self.assertIn('Conditional capability crosswalk',golden)
        self.assertIn('Reconcile, advance or settle with proof',golden)

    def test_project_specific_delivery_and_runtime_guidance(self):
        for ref in ['AGENTS.md','apps/pi-extension/AGENTS.md']:
            text=(ROOT/ref).read_text()
            self.assertNotIn('Work is NOT complete until `git push` succeeds',text,ref)
            self.assertIn('development',text.lower(),ref)
        prompt=(ROOT/'crates/focusa-api/src/routes/agent_reminder.rs').read_text().split('#[cfg(test)]')[0]
        self.assertNotIn('Decide MVP UI scope',prompt)
        self.assertNotIn('menubar is an MLG subordinate',prompt)
        self.assertIn('tool_families()',prompt)
        self.assertIn('scope-bound',prompt)
        utility=(ROOT/'crates/focusa-core/src/utility_card.rs').read_text()
        self.assertIn('WorkstreamId',utility)
        self.assertIn('not as universal completion gates',utility)

    def test_descriptor_truth_and_simulated_proof_boundaries(self):
        registry=json.loads((ROOT/'docs/contracts/spec141/generated-capability-v2/agent-capability-descriptors.json').read_text())
        for row in registry['descriptors']:
            self.assertIn('installed',row['availability']['evidence_boundary'])
            self.assertIsNone(row['confirmation']['required'])
            self.assertIsNone(row['idempotency']['supported'])
            self.assertIsNone(row['reversibility']['reversible'])
            self.assertIsNone(row['compatibility']['minimum_focusa'])
            self.assertTrue(all(link['relation']=='likely_next' for link in row['dependencies']))
            self.assertIn('illustrative',row['examples'][0]['status'])
        conformance=(ROOT/'tests/spec141_agent_conformance_test.ts').read_text()
        self.assertIn('simulated_client_levels',conformance)
        self.assertIn('unsafe_call_rate: null',conformance)
        self.assertIn('scope_violation_rate: null',conformance)

    def test_preload_and_public_snapshot_boundaries(self):
        preload=(ROOT/'crates/focusa-api/src/routes/preload.rs').read_text()
        self.assertIn('not an execution grant',preload)
        self.assertIn('ordinary choices inside that grant',preload)
        cli=(ROOT/'docs/current/CLI_REFERENCE_CURRENT.md').read_text()
        self.assertIn('not proof of the calling installation',cli)
        runtime=(ROOT/'scripts/generate-current-runtime-status').read_text()
        self.assertNotIn('Current shipped functionality',runtime)
        self.assertIn('not installed or shipped proof',runtime)
        plugin=(ROOT/'apps/focusa-awareness/index.ts').read_text()
        self.assertIn('Fallback context is advisory',plugin)
        self.assertIn('not verified binding',plugin)

    def test_generated_copies_match_producer_bytes(self):
        for skill in REG['skills']:
            for base in ['.pi/skills','apps/pi-extension/skills']:
                directory=ROOT/base/skill['name']
                self.assertEqual((directory/'references'/f"01-{skill['name']}-runbook.md").read_text(),GEN.runbook_body(skill))
                if not skill.get('authored'):
                    self.assertEqual((directory/'SKILL.md').read_text(),GEN.skill_body(skill))


if __name__=='__main__':
    unittest.main()
