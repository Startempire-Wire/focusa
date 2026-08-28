//! Project infrastructure inventory + adoption plan — slice 1 (#255).
//!
//! Read-only, target-workspace-executed detection of existing Git, task
//! providers, specifications, CI, tests, packaging, agent instructions,
//! evidence, and security boundaries. The adoption plan follows the
//! decision chain (explicit operator selection → healthy existing provider
//! → project profile preference → Focusa default → durable waiver) and is
//! PREVIEW ONLY: creation happens only for missing/explicitly replaced
//! concerns, never by inference.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const INVENTORY_SCHEMA: &str = "focusa.project_infrastructure_inventory.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectionOutcome {
    Detected,
    Missing,
    Unreadable,
}

impl DetectionOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectionOutcome::Detected => "detected",
            DetectionOutcome::Missing => "missing",
            DetectionOutcome::Unreadable => "unreadable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConcernDetection {
    pub concern: String,
    pub outcome: DetectionOutcome,
    pub evidence_paths: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfrastructureInventory {
    pub schema: String,
    pub workspace_root: String,
    pub detections: BTreeMap<String, ConcernDetection>,
    pub observed_at: String,
}

/// Task-provider adapter contract (#255 §contract). Every provider Focusa
/// adopts must declare these capabilities truthfully.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderAdapterContract {
    pub provider_id: String,
    pub provider_kind: String,
    pub workspace_binding_id: Option<String>,
    pub location: String,
    pub health: String,
    pub authority_posture: String,
    pub read_capabilities: Vec<String>,
    pub write_capabilities: Vec<String>,
    pub dependency_support: bool,
    pub done_condition_support: bool,
    pub evidence_link_support: bool,
    pub revision_fencing_support: bool,
    pub conflict_detection: bool,
    pub export_import: bool,
    pub migration_risk: String,
    pub operator_selected: bool,
    pub waiver_ref: Option<String>,
}

/// Per-concern adoption decision with the chosen provider and rationale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdoptionDecision {
    pub concern: String,
    pub selected_provider: Option<String>,
    pub selection_basis: String,
    pub existing_detected: bool,
    pub action: String,
    pub conflicts: Vec<String>,
    pub rollback: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectInfrastructureAdoptionPlan {
    pub schema: String,
    pub workspace_root: String,
    pub decisions: Vec<AdoptionDecision>,
    pub systems_left_untouched: Vec<String>,
    pub missing_capabilities: Vec<String>,
    pub requires_operator_approval: bool,
    pub preview_only: bool,
}

/// Read-only scan of the workspace. Never writes, never follows outside the
/// root, and never reads secret CONTENTS (only presence markers).
pub fn scan_infrastructure(root: &Path) -> ProjectInfrastructureInventory {
    let mut detections = BTreeMap::new();
    let mut probe = |concern: &str, paths: &[&str], notes: Vec<String>| {
        let mut evidence = Vec::new();
        for path in paths {
            let full = root.join(path);
            if full.exists() {
                evidence.push(path.to_string());
            }
        }
        let outcome = if evidence.is_empty() {
            DetectionOutcome::Missing
        } else {
            DetectionOutcome::Detected
        };
        detections.insert(
            concern.to_string(),
            ConcernDetection {
                concern: concern.to_string(),
                outcome,
                evidence_paths: evidence,
                notes,
            },
        );
    };

    probe(
        "project_markers",
        &[".focusa-project.json", ".focusa", "project.focusa.json"],
        vec![
            "markers identify Focusa projects; absence does not prove a remote host lacks one"
                .to_string(),
        ],
    );
    probe("repository", &[".git"], vec![]);
    probe("remote_worktrees", &[".git/worktrees"], vec![]);
    probe(
        "task_provider_beads",
        &[".beads/issues.jsonl", ".beads/beads.db", ".beads"],
        vec!["Beads is the fallback task provider, never mandatory duplication".to_string()],
    );
    probe("task_provider_todo_txt", &["todo.txt", "TODO.txt"], vec![]);
    probe(
        "specifications",
        &["_docs_specs", "docs", "specs", "docs/specs"],
        vec![],
    );
    probe(
        "decision_records",
        &["docs/decisions", "decisions", "docs/current"],
        vec![],
    );
    probe(
        "ci",
        &[".github/workflows", ".gitlab-ci.yml", "Jenkinsfile"],
        vec![],
    );
    probe(
        "tests",
        &["tests", "test", "spec", "__tests__"],
        vec!["test discovery is toolchain-specific; this is presence detection only".to_string()],
    );
    probe(
        "package_manifest",
        &[
            "package.json",
            "Cargo.toml",
            "pyproject.toml",
            "go.mod",
            "pom.xml",
        ],
        vec![],
    );
    probe(
        "release_versioning",
        &[
            "CHANGELOG.md",
            "changelog",
            "RELEASES.md",
            ".release-version-stamp",
        ],
        vec![],
    );
    probe(
        "deployment",
        &[
            "Dockerfile",
            "docker-compose.yml",
            "compose.yaml",
            "fly.toml",
            "vercel.json",
        ],
        vec![],
    );
    probe(
        "agent_instructions",
        &["AGENTS.md", "CLAUDE.md", ".clinerules", ".cursorrules"],
        vec![],
    );
    probe(
        "agent_skills",
        &[".pi/skills", ".claude/skills", ".agent-kb"],
        vec![],
    );
    probe(
        "evidence",
        &["docs/evidence", "evidence", "docs/current/evidence"],
        vec![],
    );
    probe(
        "secret_boundaries",
        &[".env", ".env.example", ".secrets", "secrets"],
        vec!["presence-only detection; contents are never read".to_string()],
    );
    probe(
        "ownership_constraints",
        &["CODEOWNERS", ".gitattributes", "docs/OWNERS"],
        vec![],
    );

    ProjectInfrastructureInventory {
        schema: INVENTORY_SCHEMA.to_string(),
        workspace_root: root.to_string_lossy().to_string(),
        detections,
        observed_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// Build the PREVIEW-ONLY adoption plan from an inventory. The decision
/// chain: existing healthy provider first (adoption over duplication),
/// Focusa default only for missing concerns; conflicts are reported, never
/// silently resolved.
pub fn build_adoption_plan(
    inventory: &ProjectInfrastructureInventory,
    operator_overrides: &BTreeMap<String, String>,
) -> ProjectInfrastructureAdoptionPlan {
    let mut decisions = Vec::new();
    let mut untouched = Vec::new();
    let mut missing_caps = Vec::new();
    let mut needs_approval = false;

    for (concern, detection) in &inventory.detections {
        let overridden = operator_overrides.get(concern);
        let existing = detection.outcome == DetectionOutcome::Detected;
        let selected = overridden.cloned().or_else(|| {
            if existing {
                Some(format!("existing:{concern}"))
            } else {
                None
            }
        });
        let basis = if overridden.is_some() {
            "operator_explicit_selection"
        } else if existing {
            "healthy_existing_provider"
        } else {
            "focusa_default_proposed"
        };
        let action = if overridden.is_some() {
            "replace"
        } else if existing {
            "adopt_untouched"
        } else {
            "propose_focusa_creation"
        };
        if action == "adopt_untouched" {
            untouched.push(concern.clone());
        }
        if action == "propose_focusa_creation" {
            missing_caps.push(concern.clone());
            needs_approval = true;
        }
        if action == "replace" {
            needs_approval = true;
        }
        decisions.push(AdoptionDecision {
            concern: concern.clone(),
            selected_provider: selected,
            selection_basis: basis.to_string(),
            existing_detected: existing,
            action: action.to_string(),
            conflicts: detection.notes.clone(),
            rollback: "preview only — no mutation performed".to_string(),
        });
    }

    ProjectInfrastructureAdoptionPlan {
        schema: "focusa.project_infrastructure_adoption_plan.v1".to_string(),
        workspace_root: inventory.workspace_root.clone(),
        decisions,
        systems_left_untouched: untouched,
        missing_capabilities: missing_caps,
        requires_operator_approval: needs_approval,
        preview_only: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scan_detects_existing_systems_without_mutating() {
        let dir = std::env::temp_dir().join(format!("focusa-255-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(dir.join(".beads")).unwrap();
        fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
        fs::write(dir.join("AGENTS.md"), "# rules").unwrap();
        fs::write(dir.join(".env"), "SECRET=1").unwrap();

        let inventory = scan_infrastructure(&dir);
        assert_eq!(
            inventory.detections["task_provider_beads"].outcome,
            DetectionOutcome::Detected
        );
        assert_eq!(
            inventory.detections["ci"].outcome,
            DetectionOutcome::Detected
        );
        assert_eq!(
            inventory.detections["package_manifest"].outcome,
            DetectionOutcome::Detected
        );
        assert_eq!(
            inventory.detections["agent_instructions"].outcome,
            DetectionOutcome::Detected
        );
        assert_eq!(
            inventory.detections["secret_boundaries"].outcome,
            DetectionOutcome::Detected
        );
        // Scan must not create anything.
        assert!(!dir.join("todo.txt").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn adoption_plan_adopts_existing_and_proposes_only_missing() {
        let dir = std::env::temp_dir().join(format!("focusa-255b-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(dir.join(".beads")).unwrap();
        let inventory = scan_infrastructure(&dir);
        let plan = build_adoption_plan(&inventory, &BTreeMap::new());
        assert!(plan.preview_only);
        let beads = plan
            .decisions
            .iter()
            .find(|d| d.concern == "task_provider_beads")
            .unwrap();
        assert_eq!(beads.action, "adopt_untouched");
        assert_eq!(beads.selection_basis, "healthy_existing_provider");
        let specs = plan
            .decisions
            .iter()
            .find(|d| d.concern == "specifications")
            .unwrap();
        assert_eq!(specs.action, "propose_focusa_creation");
        assert!(plan.requires_operator_approval);
        assert!(
            plan.missing_capabilities
                .contains(&"specifications".to_string())
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn operator_override_replaces_and_flags_approval() {
        let dir = std::env::temp_dir().join(format!("focusa-255c-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(dir.join(".beads")).unwrap();
        let inventory = scan_infrastructure(&dir);
        let mut overrides = BTreeMap::new();
        overrides.insert(
            "task_provider_beads".to_string(),
            "provider:jira".to_string(),
        );
        let plan = build_adoption_plan(&inventory, &overrides);
        let beads = plan
            .decisions
            .iter()
            .find(|d| d.concern == "task_provider_beads")
            .unwrap();
        assert_eq!(beads.action, "replace");
        assert_eq!(beads.selected_provider.as_deref(), Some("provider:jira"));
        assert_eq!(beads.selection_basis, "operator_explicit_selection");
        assert!(plan.requires_operator_approval);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn provider_contract_roundtrips() {
        let contract = ProviderAdapterContract {
            provider_id: "beads".to_string(),
            provider_kind: "task_provider".to_string(),
            workspace_binding_id: None,
            location: ".beads".to_string(),
            health: "healthy".to_string(),
            authority_posture: "read_write".to_string(),
            read_capabilities: vec!["issues".to_string()],
            write_capabilities: vec!["issues".to_string()],
            dependency_support: true,
            done_condition_support: true,
            evidence_link_support: true,
            revision_fencing_support: true,
            conflict_detection: true,
            export_import: true,
            migration_risk: "low".to_string(),
            operator_selected: false,
            waiver_ref: None,
        };
        let json = serde_json::to_string(&contract).unwrap();
        let parsed: ProviderAdapterContract = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, contract);
    }
}
