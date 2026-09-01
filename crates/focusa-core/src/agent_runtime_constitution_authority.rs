//! Spec 140 instruction discovery, trust, conflict, and resolution authority.

use crate::agent_runtime_constitution::{
    InstructionApplicability, InstructionAuthorityGraph, InstructionClaim, InstructionConflict,
    InstructionInjectionRecord, InstructionResolution, InstructionSource,
    InstructionSourceAuthority, InstructionTrustClass, PathInstructionPolicy,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

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
    if candidate
        .components()
        .any(|part| part == Component::ParentDir)
    {
        return Err("parent_traversal_forbidden".into());
    }
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
        blocked: true,
        reason_code: format!("instruction_injection:{}", needle.replace(' ', "_")),
        content_sha256: hex::encode(Sha256::digest(body.as_bytes())),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredInstructionSet {
    pub sources: Vec<InstructionSource>,
    pub claims: Vec<InstructionClaim>,
    pub findings: Vec<String>,
}

pub fn discover_project_instructions(
    project_root: &Path,
    max_source_bytes: u64,
) -> Result<DiscoveredInstructionSet, String> {
    let root = project_root
        .canonicalize()
        .map_err(|_| "project_root_unreadable")?;
    let mut candidates = Vec::new();
    collect_candidates(&root, &root, 0, &mut candidates)?;
    candidates.sort();
    candidates.dedup();
    let mut sources = Vec::new();
    let mut claims = Vec::new();
    let mut findings = Vec::new();
    for path in candidates {
        let metadata = fs::symlink_metadata(&path).map_err(|_| "instruction_source_unreadable")?;
        if metadata.file_type().is_symlink() {
            findings.push(format!("symlink_refused:{}", path.display()));
            continue;
        }
        if metadata.len() > max_source_bytes {
            findings.push(format!("source_too_large:{}", path.display()));
            continue;
        }
        let bytes = fs::read(&path).map_err(|_| "instruction_source_unreadable")?;
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| "instruction_source_outside_project")?
            .to_string_lossy()
            .replace('\\', "/");
        let authority = if path.parent() == Some(root.as_path()) {
            InstructionSourceAuthority::ProjectRoot
        } else {
            InstructionSourceAuthority::PathLocal
        };
        let source_id = format!(
            "source:{}",
            &hex::encode(Sha256::digest(relative.as_bytes()))[..16]
        );
        let mut source = instruction_source_from_bytes(
            source_id,
            relative,
            &bytes,
            authority,
            InstructionTrustClass::TrustedProject,
            root.display().to_string(),
        );
        if let Ok(body) = std::str::from_utf8(&bytes) {
            if contains_secret_like_material(body) {
                findings.push(format!("secret_like_source_excluded:{}", source.source_ref));
                continue;
            }
            if let Some(injection) = detect_instruction_injection(
                format!("injection:{}", source.source_id),
                &source,
                body,
            ) {
                source.trust = InstructionTrustClass::Quarantined;
                findings.push(format!("{}:{}", injection.reason_code, source.source_ref));
            } else {
                claims.extend(extract_atomic_claims(&source, body));
            }
        } else {
            findings.push(format!("non_utf8_source:{}", source.source_ref));
        }
        sources.push(source);
    }
    Ok(DiscoveredInstructionSet {
        sources,
        claims,
        findings,
    })
}

fn collect_candidates(
    root: &Path,
    directory: &Path,
    depth: usize,
    output: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if depth > 8 {
        return Ok(());
    }
    let entries = fs::read_dir(directory).map_err(|_| "instruction_directory_unreadable")?;
    for entry in entries {
        let entry = entry.map_err(|_| "instruction_directory_unreadable")?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|_| "instruction_source_unreadable")?;
        let relative = path
            .strip_prefix(root)
            .map_err(|_| "instruction_source_outside_project")?;
        let first = relative
            .components()
            .next()
            .and_then(|part| part.as_os_str().to_str());
        if matches!(first, Some(".git" | "node_modules" | "target" | ".focusa")) {
            continue;
        }
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_candidates(root, &path, depth + 1, output)?;
        } else if file_type.is_file() && is_registered_instruction_path(relative) {
            output.push(path);
        }
    }
    Ok(())
}

fn contains_secret_like_material(body: &str) -> bool {
    let normalized = body.to_ascii_lowercase();
    [
        "-----begin private key-----",
        "api_key=",
        "api-key=",
        "access_token=",
        "client_secret=",
        "password=",
        "\"private_key\":",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn is_registered_instruction_path(relative: &Path) -> bool {
    let normalized = relative.to_string_lossy().replace('\\', "/");
    let file_name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    matches!(
        normalized.as_str(),
        "AGENTS.md"
            | "CLAUDE.md"
            | "CLAUDE.local.md"
            | "GEMINI.md"
            | ".cursorrules"
            | ".windsurfrules"
            | ".pi/SYSTEM.md"
            | ".pi/APPEND_SYSTEM.md"
            | ".focusa-project.json"
            | "package.json"
            | "Cargo.toml"
            | "pyproject.toml"
            | "Taskfile.yml"
            | "Taskfile.yaml"
            | "Makefile"
    ) || normalized == ".claude/CLAUDE.md"
        || (normalized.starts_with(".claude/rules/") && normalized.ends_with(".md"))
        || normalized == ".github/copilot-instructions.md"
        || (normalized.starts_with(".github/instructions/")
            && normalized.ends_with(".instructions.md"))
        || normalized.starts_with(".cursor/rules/")
        || ((normalized.starts_with(".pi/skills/")
            || normalized.starts_with(".agents/skills/")
            || normalized.starts_with(".github/skills/"))
            && file_name == "SKILL.md")
        || (normalized.starts_with(".github/workflows/")
            && matches!(
                relative.extension().and_then(|ext| ext.to_str()),
                Some("yml" | "yaml")
            ))
        || (normalized.starts_with("docs/")
            && normalized.to_ascii_lowercase().contains("runbook")
            && normalized.ends_with(".md"))
        || (normalized.starts_with("config/")
            && normalized.to_ascii_lowercase().contains("policy")
            && matches!(
                relative.extension().and_then(|ext| ext.to_str()),
                Some("json" | "yaml" | "yml")
            ))
}

pub fn extract_atomic_claims(source: &InstructionSource, body: &str) -> Vec<InstructionClaim> {
    body.lines()
        .enumerate()
        .filter_map(|(index, raw)| {
            let text = raw
                .trim()
                .trim_start_matches(['-', '*'])
                .trim_start_matches(|character: char| {
                    character.is_ascii_digit() || character == '.' || character == ')'
                })
                .trim();
            let lower = text.to_ascii_lowercase();
            let normative = lower.contains("must ")
                || lower.contains("must not")
                || lower.contains("never ")
                || lower.contains("do not ")
                || lower.contains("required")
                || lower.starts_with("use ")
                || lower.starts_with("run ");
            if text.is_empty() || text.starts_with('#') || !normative {
                return None;
            }
            let text_hash = hex::encode(Sha256::digest(text.as_bytes()));
            let claim_class = if lower.contains("release") {
                "release_authority"
            } else if lower.contains("permission") || lower.contains("secret") {
                "security_boundary"
            } else if lower.contains("test") || lower.contains("proof") {
                "verification"
            } else if lower.contains("file") || lower.contains("edit") {
                "file_mutation"
            } else {
                "operating_instruction"
            };
            let words: Vec<_> = text.split_whitespace().collect();
            let modality_index = words.iter().position(|word| {
                matches!(
                    word.to_ascii_lowercase().as_str(),
                    "must" | "shall" | "should" | "may" | "never"
                )
            });
            let mut modality = modality_index.map(|position| words[position].to_ascii_lowercase());
            let mut action_index = modality_index.map(|position| position + 1).unwrap_or(0);
            if action_index < words.len()
                && words[action_index].eq_ignore_ascii_case("not")
                && modality.as_deref().is_some_and(|value| value != "never")
            {
                modality = modality.map(|value| format!("{value}_not"));
                action_index += 1;
            }
            let condition = [" if ", " when ", " provided that "]
                .iter()
                .find_map(|delimiter| {
                    lower
                        .find(delimiter)
                        .map(|position| text[position + delimiter.len()..].to_string())
                });
            let exceptions = [" unless ", " except "]
                .iter()
                .filter_map(|delimiter| {
                    lower
                        .find(delimiter)
                        .map(|position| text[position + delimiter.len()..].to_string())
                })
                .collect();
            Some(InstructionClaim {
                claim_id: format!("claim:{}:{}", source.source_id, index + 1),
                source_id: source.source_id.clone(),
                claim_class: claim_class.into(),
                normalized_text: text.split_whitespace().collect::<Vec<_>>().join(" "),
                source_text_sha256: text_hash.clone(),
                applicability: InstructionApplicability::Applicable,
                scope_ref: source.scope_ref.clone(),
                condition,
                subject: words.first().map(|word| (*word).to_string()),
                action: words.get(action_index).map(|word| (*word).to_string()),
                object: (action_index + 1 < words.len())
                    .then(|| words[action_index + 1..].join(" ")),
                modality,
                exceptions,
                rationale: lower
                    .find(" because ")
                    .map(|position| text[position + 9..].to_string()),
                verification_ref: (lower.contains("test")
                    || lower.contains("verify")
                    || lower.contains("proof"))
                .then(|| "instruction:verification_required".into()),
                enforcement_ref: (lower.contains("must not")
                    || lower.contains("never")
                    || lower.contains("forbid"))
                .then(|| "daemon:fail_closed".into()),
                authority_ref: Some(format!("{:?}", source.authority).to_lowercase()),
                trust_ref: Some(format!("{:?}", source.trust).to_lowercase()),
                provenance_refs: vec![source.source_ref.clone(), format!("sha256:{text_hash}")],
            })
        })
        .collect()
}

fn normalized_target(value: Option<&str>) -> Option<String> {
    value.map(|raw| {
        raw.trim_matches(|ch: char| !ch.is_alphanumeric())
            .split_whitespace()
            .map(|part| {
                part.trim_matches(|ch: char| !ch.is_alphanumeric())
                    .to_ascii_lowercase()
            })
            .collect::<Vec<_>>()
            .join(" ")
    })
}

fn modality_polarity(value: Option<&str>) -> Option<bool> {
    match value {
        Some("never" | "must_not" | "shall_not" | "should_not" | "may_not") => Some(false),
        Some("must" | "shall" | "should" | "may") | None => Some(true),
        Some(_) => None,
    }
}

fn claims_are_contradictory(left: &InstructionClaim, right: &InstructionClaim) -> bool {
    let same_target = normalized_target(left.action.as_deref())
        .zip(normalized_target(right.action.as_deref()))
        .is_some_and(|(left_action, right_action)| left_action == right_action)
        && normalized_target(left.object.as_deref())
            .zip(normalized_target(right.object.as_deref()))
            .is_some_and(|(left_object, right_object)| left_object == right_object);
    let same_condition = normalized_target(left.condition.as_deref())
        == normalized_target(right.condition.as_deref());
    same_target
        && same_condition
        && modality_polarity(left.modality.as_deref())
            .zip(modality_polarity(right.modality.as_deref()))
            .is_some_and(|(left_polarity, right_polarity)| left_polarity != right_polarity)
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
                && claims_are_contradictory(left, right)
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
