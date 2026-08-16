//! Completion Authority — slice 1 (#276/#277): deterministic acceptance
//! evaluation. A completion claim is ALLOWED only when every acceptance
//! atom is covered by at least one typed evidence ref or receipt. Free
//! text never counts as coverage; uncovered atoms block with reasons.
//! No inference — missing values stay explicit.

use serde::{Deserialize, Serialize};

pub const COMPLETION_CLAIM_SCHEMA: &str = "focusa.completion_claim.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionClaim {
    pub schema: String,
    pub work_item_id: String,
    pub acceptance_atoms: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub receipts: Vec<String>,
    pub claim_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtomCoverage {
    pub atom: String,
    pub covered_by: Vec<String>,
    pub covered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionVerdict {
    pub allow: bool,
    pub covered_atoms: Vec<AtomCoverage>,
    pub uncovered_atoms: Vec<String>,
    pub overclaim_risks: Vec<String>,
    pub reasons: Vec<String>,
}

/// Deterministic evaluation: an atom is covered when it matches (case-
/// insensitive, trimmed) at least one evidence ref or receipt identifier.
/// Evidence that covers nothing is flagged as an overclaim risk.
pub fn evaluate_completion_claim(claim: &CompletionClaim) -> CompletionVerdict {
    if claim.schema != COMPLETION_CLAIM_SCHEMA {
        return CompletionVerdict {
            allow: false,
            covered_atoms: vec![],
            uncovered_atoms: claim.acceptance_atoms.clone(),
            overclaim_risks: vec![],
            reasons: vec![format!("unexpected schema {}", claim.schema)],
        };
    }
    let normalized_refs: Vec<&str> = claim
        .evidence_refs
        .iter()
        .chain(claim.receipts.iter())
        .map(|r| r.as_str())
        .collect();

    let mut covered_atoms = Vec::new();
    let mut uncovered_atoms = Vec::new();
    for atom in &claim.acceptance_atoms {
        let needle = atom.trim().to_lowercase();
        let covered_by: Vec<String> = normalized_refs
            .iter()
            .filter(|r| r.to_lowercase().contains(&needle) && !needle.is_empty())
            .map(|r| r.to_string())
            .collect();
        if covered_by.is_empty() {
            uncovered_atoms.push(atom.clone());
        }
        let covered = !covered_by.is_empty();
        covered_atoms.push(AtomCoverage {
            atom: atom.clone(),
            covered_by,
            covered,
        });
    }

    // Overclaim risk: evidence that covers no atom (spurious citations).
    let overclaim_risks: Vec<String> = normalized_refs
        .iter()
        .filter(|r| {
            !claim
                .acceptance_atoms
                .iter()
                .any(|atom| !atom.trim().is_empty() && r.to_lowercase().contains(&atom.trim().to_lowercase()))
        })
        .map(|r| r.to_string())
        .collect();

    let mut reasons = Vec::new();
    if !uncovered_atoms.is_empty() {
        reasons.push(format!(
            "{} acceptance atoms lack typed evidence or receipts: {}",
            uncovered_atoms.len(),
            uncovered_atoms.join(", ")
        ));
    }
    if !overclaim_risks.is_empty() {
        reasons.push(format!(
            "{} evidence refs/receipts cover no acceptance atom",
            overclaim_risks.len()
        ));
    }

    CompletionVerdict {
        allow: uncovered_atoms.is_empty(),
        covered_atoms,
        uncovered_atoms,
        overclaim_risks,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(atoms: &[&str], evidence: &[&str], receipts: &[&str]) -> CompletionClaim {
        CompletionClaim {
            schema: COMPLETION_CLAIM_SCHEMA.to_string(),
            work_item_id: "w1".to_string(),
            acceptance_atoms: atoms.iter().map(|a| a.to_string()).collect(),
            evidence_refs: evidence.iter().map(|a| a.to_string()).collect(),
            receipts: receipts.iter().map(|a| a.to_string()).collect(),
            claim_text: "done".to_string(),
        }
    }

    #[test]
    fn allow_requires_every_atom_covered() {
        let verdict = evaluate_completion_claim(&claim(
            &["plan-doc", "review"],
            &["evidence/plan-doc.md"],
            &["receipt-review-1"],
        ));
        assert!(verdict.allow, "reasons: {:?}", verdict.reasons);
        assert!(verdict.uncovered_atoms.is_empty());
    }

    #[test]
    fn uncovered_atom_blocks_with_reason() {
        let verdict = evaluate_completion_claim(&claim(
            &["plan-doc", "review"],
            &["evidence/plan-doc.md"],
            &[],
        ));
        assert!(!verdict.allow);
        assert_eq!(verdict.uncovered_atoms, vec!["review".to_string()]);
        assert!(verdict.reasons[0].contains("review"));
    }

    #[test]
    fn spurious_evidence_is_an_overclaim_risk() {
        let verdict = evaluate_completion_claim(&claim(
            &["plan-doc"],
            &["evidence/plan-doc.md", "evidence/unrelated.md"],
            &[],
        ));
        assert!(verdict.allow);
        assert!(verdict.overclaim_risks.contains(&"evidence/unrelated.md".to_string()));
        assert!(!verdict.reasons.is_empty());
    }

    #[test]
    fn free_text_never_covers() {
        let verdict = evaluate_completion_claim(&claim(
            &["plan-doc"],
            &[],
            &[],
        ));
        assert!(!verdict.allow);
    }

    #[test]
    fn wrong_schema_fails_closed() {
        let mut c = claim(&["a"], &["evidence/a.md"], &[]);
        c.schema = "other".to_string();
        let verdict = evaluate_completion_claim(&c);
        assert!(!verdict.allow);
        assert!(verdict.reasons[0].contains("schema"));
    }
}
