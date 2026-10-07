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

    def test_generated_copies_match_producer_bytes(self):
        for skill in REG['skills']:
            for base in ['.pi/skills','apps/pi-extension/skills']:
                directory=ROOT/base/skill['name']
                self.assertEqual((directory/'references'/f"01-{skill['name']}-runbook.md").read_text(),GEN.runbook_body(skill))
                if not skill.get('authored'):
                    self.assertEqual((directory/'SKILL.md').read_text(),GEN.skill_body(skill))


if __name__=='__main__':
    unittest.main()
