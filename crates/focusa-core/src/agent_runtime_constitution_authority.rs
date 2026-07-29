//! Spec 140 instruction discovery, trust, conflict, and resolution authority.

use crate::agent_runtime_constitution::{
    InstructionApplicability, InstructionAuthorityGraph, InstructionClaim, InstructionConflict,
    InstructionInjectionRecord, InstructionResolution, InstructionSource,
    InstructionSourceAuthority, InstructionTrustClass, PathInstructionPolicy,
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::path::{Component, Path, PathBuf};

pub const PROJECT_INSTRUCTION_FILES: &[&str] = &[
    "AGENTS.md",
    "CLAUDE.md",
    "GEMINI.md",
    ".cursorrules",
    ".windsurfrules",
];

pub fn source_authority_rank(authority: InstructionSourceAuthority) -> u8 {
    match authority {
        InstructionSourceAuthority::HarnessSystem => 100,
        InstructionSourceAuthority::FocusaConstitution => 90,
        InstructionSourceAuthority::ProjectRoot => 70,
        InstructionSourceAuthority::PathLocal => 60,
        InstructionSourceAuthority::UserManaged => 50,
        InstructionSourceAuthority::Imported => 30,
        InstructionSourceAuthority::Untrusted => 0,
    }
}

pub fn default_authority_graph() -> InstructionAuthorityGraph {
    InstructionAuthorityGraph {
        graph_id: "focusa.instruction_authority.v1".into(),
        ordered_authorities: vec![
            InstructionSourceAuthority::HarnessSystem,
            InstructionSourceAuthority::FocusaConstitution,
            InstructionSourceAuthority::ProjectRoot,
            InstructionSourceAuthority::PathLocal,
            InstructionSourceAuthority::UserManaged,
            InstructionSourceAuthority::Imported,
            InstructionSourceAuthority::Untrusted,
        ],
        conditional_edges: vec![
            ("operator_steering".into(), "all_runtime_layers".into()),
            ("path_local".into(), "matching_path_scope_only".into()),
        ],
    }
}

pub fn bounded_instruction_path(
    project_root: &Path,
    candidate: &Path,
    policy: &PathInstructionPolicy,
) -> Result<PathBuf, String> {
    let root = project_root
        .canonicalize()
        .map_err(|_| "project_root_unreadable")?;
    let clean = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        root.join(candidate)
    };
    if clean.components().any(|part| part == Component::ParentDir) {
        return Err("parent_traversal_forbidden".into());
    }
    let canonical = clean
        .canonicalize()
        .map_err(|_| "instruction_source_unreadable")?;
    if !canonical.starts_with(&root) {
        return Err("instruction_source_outside_project".into());
    }
    let relative = canonical
        .strip_prefix(&root)
        .map_err(|_| "scope_resolution_failed")?;
    if policy
        .deny_paths
        .iter()
        .any(|entry| relative.starts_with(entry))
    {
        return Err("instruction_source_denied".into());
    }
    Ok(canonical)
}

pub fn instruction_source_from_bytes(
    source_id: impl Into<String>,
    source_ref: impl Into<String>,
    bytes: &[u8],
    authority: InstructionSourceAuthority,
    trust: InstructionTrustClass,
    scope_ref: impl Into<String>,
) -> InstructionSource {
    InstructionSource {
        source_id: source_id.into(),
        source_ref: source_ref.into(),
        content_sha256: hex::encode(Sha256::digest(bytes)),
        authority,
        trust,
        freshness: crate::agent_runtime_constitution::InstructionFreshness::Current,
        scope_ref: scope_ref.into(),
        discovered_at: Utc::now(),
    }
}

pub fn detect_instruction_injection(
    record_id: impl Into<String>,
    source: &InstructionSource,
    body: &str,
) -> Option<InstructionInjectionRecord> {
    let normalized = body.to_ascii_lowercase();
    let suspicious = [
        "ignore previous instructions",
        "override system prompt",
        "reveal hidden prompt",
        "disable safety",
        "exfiltrate secret",
    ]
    .iter()
    .find(|needle| normalized.contains(**needle));
    suspicious.map(|needle| InstructionInjectionRecord {
        record_id: record_id.into(),
        source_ref: source.source_ref.clone(),
        trust: source.trust,
        blocked: matches!(
            source.trust,
            InstructionTrustClass::Untrusted | InstructionTrustClass::Quarantined
        ),
        reason_code: format!("instruction_injection:{}", needle.replace(' ', "_")),
        content_sha256: hex::encode(Sha256::digest(body.as_bytes())),
    })
}

pub fn detect_conflicts(
    claims: &[InstructionClaim],
    graph: &InstructionAuthorityGraph,
) -> Vec<InstructionConflict> {
    let mut conflicts = Vec::new();
    for (left_index, left) in claims.iter().enumerate() {
        for right in claims.iter().skip(left_index + 1) {
            if left.claim_class == right.claim_class
                && left.scope_ref == right.scope_ref
                && left.normalized_text != right.normalized_text
                && left.applicability != InstructionApplicability::NotApplicable
                && right.applicability != InstructionApplicability::NotApplicable
            {
                let mut refs = vec![left.claim_id.clone(), right.claim_id.clone()];
                refs.sort();
                conflicts.push(InstructionConflict {
                    conflict_id: format!("conflict:{}", refs.join(":")),
                    claim_refs: refs,
                    conflict_class: "contradictory_instruction".into(),
                    authority_graph_ref: graph.graph_id.clone(),
                    requires_operator: false,
                    detected_at: Utc::now(),
                });
            }
        }
    }
    conflicts
}

pub fn resolve_conflict(
    conflict: &InstructionConflict,
    claims: &[InstructionClaim],
    sources: &[InstructionSource],
    operator_winner: Option<&str>,
) -> Result<InstructionResolution, String> {
    let candidates: Vec<_> = conflict
        .claim_refs
        .iter()
        .filter_map(|claim_id| claims.iter().find(|claim| &claim.claim_id == claim_id))
        .collect();
    if candidates.len() != conflict.claim_refs.len() {
        return Err("conflict_claim_missing".into());
    }
    let ranked: Vec<_> = candidates
        .iter()
        .map(|claim| {
            let source = sources
                .iter()
                .find(|source| source.source_id == claim.source_id)
                .ok_or("claim_source_missing")?;
            Ok((*claim, source_authority_rank(source.authority)))
        })
        .collect::<Result<_, &str>>()
        .map_err(str::to_string)?;
    let highest = ranked.iter().map(|(_, rank)| *rank).max().unwrap_or(0);
    let leaders: Vec<_> = ranked
        .iter()
        .filter(|(_, rank)| *rank == highest)
        .map(|(claim, _)| *claim)
        .collect();
    let winner = if leaders.len() == 1 {
        leaders[0]
    } else {
        let selected = operator_winner.ok_or("operator_resolution_required")?;
        leaders
            .iter()
            .copied()
            .find(|claim| claim.claim_id == selected)
            .ok_or("operator_winner_not_authoritative")?
    };
    let suppressed = candidates
        .iter()
        .filter(|claim| claim.claim_id != winner.claim_id)
        .map(|claim| claim.claim_id.clone())
        .collect();
    Ok(InstructionResolution {
        resolution_id: format!("resolution:{}", conflict.conflict_id),
        conflict_id: conflict.conflict_id.clone(),
        disposition: if leaders.len() == 1 {
            "authority_precedence"
        } else {
            "operator_confirmed"
        }
        .into(),
        winning_claim_refs: vec![winner.claim_id.clone()],
        suppressed_claim_refs: suppressed,
        rationale: "Explicit authority graph; no source-order or last-write-wins inference.".into(),
        operator_confirmed: leaders.len() > 1,
        evidence_refs: vec![conflict.authority_graph_ref.clone()],
    })
}
