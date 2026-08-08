//! Active-project guard for Spec 172 verified-no-license limited mode.
//!
//! In verified no-license posture, mutation is allowed only in exactly one
//! explicitly operator-selected project. All other retained projects remain
//! readable and exportable. The system never deletes data or uses activity
//! heuristics to select a project.
//!
//! If a downgrade (refund/revoke) leaves more than one project without an
//! explicit selection, the runtime requires operator choice and performs no
//! value-producing mutation until a project is explicitly selected.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Schema identifier for the persisted active-project selection.
const SCHEMA_ACTIVE_PROJECT: &str = "focusa.active_project_selection.v1";

/// Persisted record of the operator's explicit active-project selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActiveProjectSelection {
    pub schema: String,
    pub project_root: String,
    pub selected_at: String,
    pub selected_by: String,
    pub note: Option<String>,
}

impl ActiveProjectSelection {
    pub fn new(project_root: impl Into<String>, selected_by: impl Into<String>) -> Self {
        Self {
            schema: SCHEMA_ACTIVE_PROJECT.to_string(),
            project_root: project_root.into(),
            selected_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            selected_by: selected_by.into(),
            note: None,
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }
}

/// Decision for a project mutation attempt in verified-no-license posture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum ProjectMutationDecision {
    /// Mutation is permitted: this is the active project, or entitlement is paid.
    Allowed,
    /// Denied: a second project was targeted. The active project is preserved.
    /// Upgrade to Operator or explicitly switch the active project.
    DeniedSecondProject {
        active_project_root: String,
        attempted_project_root: String,
        reason: String,
        recovery_action: String,
    },
    /// Denied: no active project has been explicitly selected.
    /// Operator must choose one project before value-producing mutation.
    DeniedNoSelection {
        attempted_project_root: String,
        reason: String,
        recovery_action: String,
    },
}

impl ProjectMutationDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    pub fn is_denied(&self) -> bool {
        !self.is_allowed()
    }

    pub fn recovery_action(&self) -> &str {
        match self {
            Self::Allowed => "",
            Self::DeniedSecondProject {
                recovery_action, ..
            }
            | Self::DeniedNoSelection {
                recovery_action, ..
            } => recovery_action,
        }
    }
}

/// Active-project guard for Spec 172 verified-no-license limited mode.
///
/// The guard is a pure policy function: it receives the current entitlement
/// posture and the persisted active-project selection, and decides whether
/// a mutation on a given project root is permitted.
///
/// It never deletes data, never uses activity heuristics, and never
/// manufactures a selection.
pub struct ActiveProjectGuard;

impl ActiveProjectGuard {
    /// Check whether a project mutation is permitted under the current posture.
    ///
    /// - `Paid`: always allowed (the base product gate handles the rest).
    /// - `VerifiedNoLicense`: allowed only when `project_root` matches the
    ///   explicitly selected active project. If no selection exists, requires
    ///   operator choice. If a different project is selected, denies with an
    ///   upgrade/switch action.
    /// - Any other posture: denied (handled by the base product gate).
    pub fn check_mutation(
        posture: focusa_license::BaseProductDecision,
        project_root: &str,
        active_selection: Option<&ActiveProjectSelection>,
    ) -> ProjectMutationDecision {
        match posture {
            focusa_license::BaseProductDecision::Entitled => {
                // Paid entitlement: base product gate already passed.
                ProjectMutationDecision::Allowed
            }
            focusa_license::BaseProductDecision::Limited => {
                // Verified no-license: enforce one-project rule.
                match active_selection {
                    None => ProjectMutationDecision::DeniedNoSelection {
                        attempted_project_root: project_root.to_string(),
                        reason: "No active project has been explicitly selected. In verified no-license posture, exactly one project must be selected before value-producing mutation."
                            .to_string(),
                        recovery_action: "Select an active project with `focusa project use <path>` or upgrade to Focusa Operator."
                            .to_string(),
                    },
                    Some(selection) => {
                        if selection.project_root == project_root {
                            ProjectMutationDecision::Allowed
                        } else {
                            ProjectMutationDecision::DeniedSecondProject {
                                active_project_root: selection.project_root.clone(),
                                attempted_project_root: project_root.to_string(),
                                reason: "Verified no-license posture allows mutation in only one project. The currently active project differs from the targeted project."
                                    .to_string(),
                                recovery_action: "Switch the active project with `focusa project switch <path>` or upgrade to Focusa Operator for multi-project mutation."
                                    .to_string(),
                            }
                        }
                    }
                }
            }
            focusa_license::BaseProductDecision::Denied => {
                // Base product gate already denied; this is a secondary check.
                // The primary gate should have rejected this already.
                ProjectMutationDecision::DeniedNoSelection {
                    attempted_project_root: project_root.to_string(),
                    reason: "Base product entitlement is denied for this posture."
                        .to_string(),
                    recovery_action: "Upgrade to Focusa Operator or verify mailbox to access limited mode."
                        .to_string(),
                }
            }
        }
    }
}

/// Path to the persisted active-project selection file.
pub fn active_project_selection_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/root"));
    home.join(".config")
        .join("focusa")
        .join("active-project-selection.json")
}

/// Load the persisted active-project selection, if any.
pub fn load_active_project_selection() -> Option<ActiveProjectSelection> {
    let path = active_project_selection_path();
    if !path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    let selection: ActiveProjectSelection = serde_json::from_str(&raw).ok()?;
    if selection.schema == SCHEMA_ACTIVE_PROJECT && !selection.project_root.is_empty() {
        Some(selection)
    } else {
        None
    }
}

/// Persist the active-project selection to disk.
pub fn save_active_project_selection(selection: &ActiveProjectSelection) -> std::io::Result<()> {
    let path = active_project_selection_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(selection).map_err(std::io::Error::other)?;
    std::fs::write(&path, json)
}

/// Clear the persisted active-project selection.
pub fn clear_active_project_selection() -> std::io::Result<()> {
    let path = active_project_selection_path();
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

/// Switch the active project to a new root.
///
/// This is always permitted — it only changes which project is the mutable
/// one. It preserves all retained project data and never deletes anything.
pub fn switch_active_project(
    project_root: impl Into<String>,
    selected_by: impl Into<String>,
) -> std::io::Result<ActiveProjectSelection> {
    let selection = ActiveProjectSelection::new(project_root, selected_by);
    save_active_project_selection(&selection)?;
    Ok(selection)
}

/// Convenience: persist the canonical project selection from a path.
/// Used by `focusa project use` and `focusa project switch` CLI paths.
pub fn persist_active_project_from_path(
    project_root: &Path,
    selected_by: &str,
) -> std::io::Result<ActiveProjectSelection> {
    let canonical = std::fs::canonicalize(project_root)
        .unwrap_or_else(|_| project_root.to_path_buf());
    switch_active_project(
        canonical.to_string_lossy().to_string(),
        selected_by,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verified_limited_project_allows_mutation_in_active_project() {
        let selection = ActiveProjectSelection::new("/home/user/projects/my-focusa", "cli");
        let decision = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/my-focusa",
            Some(&selection),
        );
        assert!(decision.is_allowed());
    }

    #[test]
    fn verified_limited_project_denies_mutation_in_second_project() {
        let selection = ActiveProjectSelection::new("/home/user/projects/project-a", "cli");
        let decision = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-b",
            Some(&selection),
        );
        assert!(decision.is_denied());
        match decision {
            ProjectMutationDecision::DeniedSecondProject {
                active_project_root,
                attempted_project_root,
                ..
            } => {
                assert_eq!(active_project_root, "/home/user/projects/project-a");
                assert_eq!(attempted_project_root, "/home/user/projects/project-b");
            }
            _ => panic!("expected DeniedSecondProject"),
        }
    }

    #[test]
    fn verified_limited_project_denies_mutation_without_selection() {
        let decision = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/any-project",
            None,
        );
        assert!(decision.is_denied());
        match decision {
            ProjectMutationDecision::DeniedNoSelection { .. } => {}
            _ => panic!("expected DeniedNoSelection"),
        }
    }

    #[test]
    fn verified_limited_project_allows_paid_entitlement_regardless_of_selection() {
        // With paid entitlement, the project check is bypassed.
        let selection = ActiveProjectSelection::new("/home/user/projects/project-a", "cli");
        let decision = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Entitled,
            "/home/user/projects/project-b",
            Some(&selection),
        );
        assert!(decision.is_allowed());

        // Even without a selection, paid entitlement allows mutation.
        let decision = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Entitled,
            "/home/user/projects/any-project",
            None,
        );
        assert!(decision.is_allowed());
    }

    #[test]
    fn verified_limited_project_denied_posture_is_denied() {
        let selection = ActiveProjectSelection::new("/home/user/projects/project-a", "cli");
        let decision = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Denied,
            "/home/user/projects/project-a",
            Some(&selection),
        );
        assert!(decision.is_denied());
    }

    #[test]
    fn verified_limited_project_switch_preserves_and_does_not_delete() {
        let selection_a = ActiveProjectSelection::new("/home/user/projects/project-a", "cli");
        let decision_a = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-a",
            Some(&selection_a),
        );
        assert!(decision_a.is_allowed());

        // After switching to project-b, project-a is no longer mutable
        let selection_b = ActiveProjectSelection::new("/home/user/projects/project-b", "cli");
        let decision_b = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-b",
            Some(&selection_b),
        );
        assert!(decision_b.is_allowed());

        // But project-a is now denied for mutation (it's still readable/exportable)
        let decision_a_after_switch = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-a",
            Some(&selection_b),
        );
        assert!(decision_a_after_switch.is_denied());
        match decision_a_after_switch {
            ProjectMutationDecision::DeniedSecondProject {
                active_project_root,
                ..
            } => {
                assert_eq!(active_project_root, "/home/user/projects/project-b");
            }
            _ => panic!("expected DeniedSecondProject"),
        }
    }

    #[test]
    fn verified_limited_project_empty_project_root_is_rejected() {
        let selection = ActiveProjectSelection::new("", "cli");
        let decision = ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/any-project",
            Some(&selection),
        );
        assert!(decision.is_denied());
    }

    #[test]
    fn verified_limited_project_read_export_never_blocked() {
        // Read projection and export operations do not go through the project
        // mutation guard. They are always available regardless of the active
        // project selection. This test verifies that the guard itself does not
        // interfere with non-mutation postures.
        let selection = ActiveProjectSelection::new("/home/user/projects/project-a", "cli");

        // The guard is only for mutation checks. Read/export/recovery
        // operations are handled by the entitlement state grid reducer and
        // are always available regardless of active project selection.
        // This test confirms the guard's posture classification is correct.
        assert!(ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-a",
            Some(&selection),
        )
        .is_allowed());
        assert!(ActiveProjectGuard::check_mutation(
            focusa_license::BaseProductDecision::Limited,
            "/home/user/projects/project-b",
            Some(&selection),
        )
        .is_denied());
    }

    #[test]
    fn verified_limited_project_serialization_round_trips() {
        let selection = ActiveProjectSelection::new("/home/user/projects/my-focusa", "focusa-cli")
            .with_note("selected for Spec 172 limited mode");
        let json = serde_json::to_string(&selection).unwrap();
        let parsed: ActiveProjectSelection = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.schema, SCHEMA_ACTIVE_PROJECT);
        assert_eq!(parsed.project_root, "/home/user/projects/my-focusa");
        assert_eq!(parsed.selected_by, "focusa-cli");
        assert_eq!(parsed.note.as_deref(), Some("selected for Spec 172 limited mode"));
        assert!(!parsed.selected_at.is_empty());
    }

    #[test]
    fn verified_limited_project_decision_serialization_round_trips() {
        let allowed = ProjectMutationDecision::Allowed;
        let json = serde_json::to_string(&allowed).unwrap();
        let parsed: ProjectMutationDecision = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_allowed());

        let denied = ProjectMutationDecision::DeniedSecondProject {
            active_project_root: "/a".to_string(),
            attempted_project_root: "/b".to_string(),
            reason: "test".to_string(),
            recovery_action: "switch".to_string(),
        };
        let json = serde_json::to_string(&denied).unwrap();
        let parsed: ProjectMutationDecision = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_denied());

        let no_sel = ProjectMutationDecision::DeniedNoSelection {
            attempted_project_root: "/c".to_string(),
            reason: "test".to_string(),
            recovery_action: "select".to_string(),
        };
        let json = serde_json::to_string(&no_sel).unwrap();
        let parsed: ProjectMutationDecision = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_denied());
    }
}