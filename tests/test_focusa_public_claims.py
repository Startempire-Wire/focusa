"""Regression tests for proposed Focusa public FAQ and owner claims."""
import importlib.util
import tempfile
import unittest
from pathlib import Path

MOD_PATH = Path(__file__).resolve().parents[1] / "scripts" / "focusa_public_claims.py"
spec = importlib.util.spec_from_file_location("focusa_public_claims", MOD_PATH)
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class TestBuyerFaqContract(unittest.TestCase):
    def test_all_fifty_five_visible_and_structured_answers_match(self):
        self.assertEqual([], gate.validate_contract())

    def test_prior_agent_style_omission_is_blocked(self):
        old = (
            "Startempire Wire owns and licenses Focusa. UIAI Engine is a separate "
            "product. Philoveracity is a separate business. Verious Smith is "
            "Startempire Wire's named commercial licensing contact for Focusa."
        )
        self.assertTrue(gate.validate_outbound(old))

    def test_founder_architect_ownership_statement_passes(self):
        correct = (
            "Verious Smith III is the creator and principal architect of Focusa. "
            "He owns Startempire Wire LLC, the software company under which both "
            "Focusa and UIAI Engine are developed. Philoveracity is also his "
            "agency and DBA, operating since 2010."
        )
        self.assertEqual([], gate.validate_outbound(correct))

    def test_generic_price_email_not_forced_into_biography(self):
        self.assertEqual([], gate.validate_outbound("The Focusa setup is $5,000; licenses are separate."))

    def test_placeholders_fail_closed(self):
        self.assertTrue(gate.validate_outbound("Hi, [[CONFIRM BEFORE SEND: product owner]]"))

    def test_wordpress_surgical_patch_and_idempotence(self):
        doc = gate.PAGE.read_text(encoding="utf-8")
        self.assertEqual(doc, gate.patch_wordpress(doc))
        faq = gate.read(gate.FAQ)
        first = next(i for i in faq["questions"] if i.get("review_state") == "proposed_correction")
        old_answer = "Yes. A daily summary can show urgent work, approaching deadlines, items waiting on someone else, blocked work, and work that Focusa completed. We must first connect and test the systems that supply this information."
        self.assertEqual("focusa-faq-005", first["id"])
        old = doc.replace(first["answer"], old_answer, 1)
        self.assertEqual(doc, gate.patch_wordpress(old))

    def test_wordpress_new_unapproved_change_is_blocked(self):
        doc = gate.PAGE.read_text(encoding="utf-8")
        dirty = doc.replace("<summary>Which work should we give Focusa first?</summary>", "<summary>Different question?</summary>")
        with self.assertRaises(ValueError):
            gate.patch_wordpress(dirty)


if __name__ == "__main__":
    unittest.main()
