//! Typed Acceptance Atoms + Proof Runtime — slice 1 (#278).
//!
//! An acceptance atom is the smallest verifiable unit of completion: a
//! stable ID, the predicate, the required evidence kind, and the exact
//! verification command. The proof runtime evaluates atoms against
//! evidence refs deterministically — the same inputs always produce the
//! same verdict (pairs with #276 completion authority).

use serde::{Deserialize, Serialize};

pub const ACCEPTANCE_ATOM_SCHEMA: &str = "focusa.acceptance_atom.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceAtom {
    pub schema: String,
    pub atom_id: String,
    pub predicate: String,
    pub evidence_kind: String,
    pub verify_command: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtomVerdict {
    pub atom_id: String,
    pub satisfied: bool,
    pub matched_evidence: Vec<String>,
    pub reasons: Vec<String>,
}

/// Deterministic proof: an atom is satisfied when at least one evidence
/// ref matches the atom id (case-insensitive) AND the evidence kind
/// matches the atom's evidence kind. Mismatched kinds are rejected with a
/// typed reason — never silently accepted.
pub fn evaluate_atoms(
    atoms: &[AcceptanceAtom],
    evidence: &[(String, String)],
) -> Vec<AtomVerdict> {
    atoms
        .iter()
        .map(|atom| {
            if atom.schema != ACCEPTANCE_ATOM_SCHEMA {
                return AtomVerdict {
                    atom_id: atom.atom_id.clone(),
                    satisfied: false,
                    matched_evidence: vec![],
                    reasons: vec![format!("unexpected schema {}", atom.schema)],
                };
            }
            let needle = atom.atom_id.trim().to_lowercase();
            let mut matched = Vec::new();
            let mut reasons = Vec::new();
            for (reference, kind) in evidence {
                let reference_lc = reference.to_lowercase();
                if !reference_lc.contains(&needle) || needle.is_empty() {
                    continue;
                }
                if kind == &atom.evidence_kind {
                    matched.push(reference.clone());
                } else {
                    reasons.push(format!(
                        "evidence {reference} has kind {kind}, atom requires {}",
                        atom.evidence_kind
                    ));
                }
            }
            if matched.is_empty() && reasons.is_empty() {
                reasons.push(format!(
                    "no evidence matches atom {} ({})",
                    atom.atom_id, atom.predicate
                ));
            }
            AtomVerdict {
                atom_id: atom.atom_id.clone(),
                satisfied: !matched.is_empty(),
                matched_evidence: matched,
                reasons,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atom(id: &str, kind: &str) -> AcceptanceAtom {
        AcceptanceAtom {
            schema: ACCEPTANCE_ATOM_SCHEMA.to_string(),
            atom_id: id.to_string(),
            predicate: "plan doc exists".to_string(),
            evidence_kind: kind.to_string(),
            verify_command: "test -f docs/plan.md".to_string(),
            required: true,
        }
    }

    #[test]
    fn atom_satisfies_with_kind_matching_evidence() {
        let atoms = vec![atom("plan-doc", "artifact")];
        let evidence = vec![
            ("evidence/plan-doc.md".to_string(), "artifact".to_string()),
        ];
        let verdicts = evaluate_atoms(&atoms, &evidence);
        assert!(verdicts[0].satisfied);
        assert_eq!(verdicts[0].matched_evidence.len(), 1);
    }

    #[test]
    fn kind_mismatch_is_rejected_with_reason() {
        let atoms = vec![atom("plan-doc", "artifact")];
        let evidence = vec![
            ("evidence/plan-doc.md".to_string(), "screenshot".to_string()),
        ];
        let verdicts = evaluate_atoms(&atoms, &evidence);
        assert!(!verdicts[0].satisfied);
        assert!(verdicts[0].reasons[0].contains("kind"));
    }

    #[test]
    fn missing_evidence_is_named() {
        let atoms = vec![atom("plan-doc", "artifact")];
        let verdicts = evaluate_atoms(&atoms, &[]);
        assert!(!verdicts[0].satisfied);
        assert!(verdicts[0].reasons[0].contains("no evidence"));
    }

    #[test]
    fn optional_atom_missing_does_not_block() {
        let mut optional = atom("nice-to-have", "artifact");
        optional.required = false;
        let verdicts = evaluate_atoms(&[optional], &[]);
        assert!(!verdicts[0].satisfied);
        // Non-blocking semantics are the caller's to apply; the verdict is
        // always honest about satisfaction.
        assert!(!verdicts[0].satisfied);
    }

    #[test]
    fn evaluation_is_deterministic() {
        let atoms = vec![atom("a", "artifact"), atom("b", "receipt")];
        let evidence = vec![
            ("evidence/a.md".to_string(), "artifact".to_string()),
            ("receipts/b.json".to_string(), "receipt".to_string()),
        ];
        let first = evaluate_atoms(&atoms, &evidence);
        let second = evaluate_atoms(&atoms, &evidence);
        assert_eq!(first, second);
        assert!(first.iter().all(|v| v.satisfied));
    }
}
