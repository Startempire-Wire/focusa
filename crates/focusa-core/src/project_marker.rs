//! Canonical project-marker service (#243).
//!
//! One core implementation for creating, validating, enriching, migrating,
//! and repairing `.focusa-project.json` markers. Every CLI/API/Pi/onboarding
//! path should route marker production through this module so the schema,
//! validation semantics, atomicity, backup behavior, and ownership rules
//! stay identical everywhere.
//!
//! Guarantees:
//! - atomic writes (temp file + fsync + rename); interrupted writes never
//!   leave a partial marker;
//! - idempotent: writing over a valid identical marker is a no-op outcome;
//! - legacy/minimal v1 markers are detected and enriched without silently
//!   changing project identity (project_id + project_root are preserved);
//! - a pre-migration backup is kept beside the marker when content changes;
//! - directory-ownership preservation: when the project directory is owned
//!   by another user (cPanel-style), creation is refused with a typed
//!   `blocked_permission` outcome instead of writing root-owned files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const MARKER_FILE: &str = ".focusa-project.json";
pub const MARKER_SCHEMA: &str = "focusa.project.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMarker {
    pub schema: String,
    pub project_id: String,
    pub canonical_name: String,
    pub project_root: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_remote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beads_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_id: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl ProjectMarker {
    pub fn canonical_project_root(&self) -> Option<PathBuf> {
        let path = Path::new(&self.project_root);
        if path.is_absolute() {
            Some(path.to_path_buf())
        } else {
            None
        }
    }

    /// Fields the enriched schema requires but a legacy minimal marker may
    /// omit. Identity fields (project_id, project_root, canonical_name) are
    /// never in this list — enrichment must not change identity.
    pub fn missing_enriched_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.repo_remote.is_none() {
            missing.push("repo_remote");
        }
        if self.beads_prefix.is_none() {
            missing.push("beads_prefix");
        }
        if self.workspace_kind.is_none() {
            missing.push("workspace_kind");
        }
        if self.continuity_id.is_none() {
            missing.push("continuity_id");
        }
        if self.updated_at.is_none() {
            missing.push("updated_at");
        }
        missing
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerReadOutcome {
    Missing,
    Valid,
    LegacyMinimal { missing_fields: Vec<String> },
    Corrupted { error: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerWriteOutcome {
    Created,
    AlreadyValid,
    Migrated { backup: Option<String>, added_fields: Vec<String> },
    Written { backup: Option<String> },
    BlockedPermission { directory_owner: String, current_user: String },
}

#[derive(Debug, Clone, Default)]
pub struct MarkerWriteOptions {
    pub preview: bool,
    pub backup: bool,
    pub overwrite: bool,
}

/// Read + classify the marker at `root`.
pub fn read_marker(root: &Path) -> MarkerReadOutcome {
    let path = root.join(MARKER_FILE);
    if !path.is_file() {
        return MarkerReadOutcome::Missing;
    }
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) => {
            return MarkerReadOutcome::Corrupted {
                error: format!("read failed: {error}"),
            }
        }
    };
    let parsed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(error) => {
            return MarkerReadOutcome::Corrupted {
                error: format!("invalid JSON: {error}"),
            }
        }
    };
    if parsed.get("schema").and_then(|v| v.as_str()) != Some(MARKER_SCHEMA) {
        return MarkerReadOutcome::Corrupted {
            error: format!("unknown schema (expected {MARKER_SCHEMA})"),
        };
    }
    let identity_ok = parsed.get("project_id").and_then(|v| v.as_str()).is_some()
        && parsed.get("canonical_name").and_then(|v| v.as_str()).is_some()
        && parsed.get("project_root").and_then(|v| v.as_str()).is_some();
    if !identity_ok {
        return MarkerReadOutcome::Corrupted {
            error: "missing identity fields (project_id, canonical_name, project_root)".into(),
        };
    }
    match serde_json::from_value::<ProjectMarker>(parsed) {
        Ok(marker) => {
            let missing = marker
                .missing_enriched_fields()
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>();
            if missing.is_empty() {
                MarkerReadOutcome::Valid
            } else {
                MarkerReadOutcome::LegacyMinimal {
                    missing_fields: missing,
                }
            }
        }
        Err(error) => MarkerReadOutcome::Corrupted {
            error: format!("schema parse failed: {error}"),
        },
    }
}

/// Detect the directory owner vs the current effective user. Returns
/// (owner_name, current_name) when they differ.
fn id_name(uid: u32) -> String {
    std::process::Command::new("id")
        .arg("-un")
        .arg(uid.to_string())
        .output()
        .ok()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| uid.to_string())
}

/// Detect the directory owner vs the current effective user. Returns
/// (owner_name, current_name) when they differ. Uses the `id` command so
/// this stays dependency-free and never links libc struct layouts.
fn ownership_mismatch(root: &Path) -> Option<(String, String)> {
    use std::os::unix::fs::MetadataExt;
    let metadata = fs::metadata(root).ok()?;
    let owner_uid = metadata.uid();
    let current_uid = {
        std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|output| String::from_utf8_lossy(&output.stdout).trim().parse::<u32>().ok())
            .unwrap_or(owner_uid)
    };
    if owner_uid == current_uid {
        return None;
    }
    Some((id_name(owner_uid), id_name(current_uid)))
}

/// Canonical marker write: atomic, idempotent, ownership-preserving,
/// with optional preview and pre-migration backup. Returns the typed
/// outcome for the caller's JSON payload.
pub fn write_marker(
    root: &Path,
    marker: &ProjectMarker,
    options: &MarkerWriteOptions,
) -> Result<MarkerWriteOutcome> {
    if marker.schema != MARKER_SCHEMA {
        anyhow::bail!("marker schema must be {MARKER_SCHEMA}");
    }
    let path = root.join(MARKER_FILE);
    if let Some((owner, current)) = ownership_mismatch(root) {
        return Ok(MarkerWriteOutcome::BlockedPermission {
            directory_owner: owner,
            current_user: current,
        });
    }
    let existing = read_marker(root);
    let legacy_missing = match &existing {
        MarkerReadOutcome::LegacyMinimal { missing_fields } => Some(missing_fields.clone()),
        _ => None,
    };
    let existing_identity = match &existing {
        MarkerReadOutcome::Valid | MarkerReadOutcome::LegacyMinimal { .. } => {
            let raw = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str::<serde_json::Value>(&raw).ok().and_then(|value| {
                Some((
                    value.get("project_id")?.as_str()?.to_string(),
                    value.get("project_root")?.as_str()?.to_string(),
                ))
            })
        }
        _ => None,
    };
    if let Some((existing_id, existing_root)) = &existing_identity {
        if existing_id != &marker.project_id || existing_root != &marker.project_root {
            anyhow::bail!(
                "marker conflict: existing identity ({existing_id} @ {existing_root}) differs from requested ({} @ {})",
                marker.project_id,
                marker.project_root
            );
        }
        // Non-legacy existing markers short-circuit as already-valid;
        // legacy/minimal markers fall through so the enrichment actually
        // writes (with backup + atomicity) rather than claiming migration.
        if !options.overwrite && legacy_missing.is_none() {
            return Ok(MarkerWriteOutcome::AlreadyValid);
        }
        if options.preview {
            return Ok(MarkerWriteOutcome::Migrated {
                backup: None,
                added_fields: marker
                    .missing_enriched_fields()
                    .iter()
                    .map(|field| field.to_string())
                    .collect(),
            });
        }
    }
    if options.preview {
        return Ok(match existing_identity {
            Some(_) => MarkerWriteOutcome::Migrated {
                backup: None,
                added_fields: marker
                    .missing_enriched_fields()
                    .iter()
                    .map(|field| field.to_string())
                    .collect(),
            },
            None => MarkerWriteOutcome::Created,
        });
    }
    // Atomic write: temp in the same directory, fsync, rename.
    let serialized = serde_json::to_vec_pretty(marker)?;
    let temp = root.join(format!(".focusa-project.json.tmp-{}", std::process::id()));
    let backup = if options.backup && path.is_file() {
        let backup_path = root.join(".focusa-project.json.pre-migration");
        fs::copy(&path, &backup_path)
            .with_context(|| format!("backup marker to {}", backup_path.display()))?;
        Some(backup_path)
    } else {
        None
    };
    fs::write(&temp, serialized).with_context(|| "write marker temp file")?;
    if let Ok(file) = fs::File::open(&temp) {
        let _ = file.sync_all();
    }
    fs::rename(&temp, &path).with_context(|| "atomically replace marker")?;
    let created = existing_identity.is_none();
    if legacy_missing.is_some() {
        Ok(MarkerWriteOutcome::Migrated {
            backup: backup.map(|p| p.display().to_string()),
            added_fields: marker
                .missing_enriched_fields()
                .iter()
                .map(|field| field.to_string())
                .collect(),
        })
    } else if created {
        Ok(MarkerWriteOutcome::Created)
    } else {
        Ok(MarkerWriteOutcome::Written {
            backup: backup.map(|p| p.display().to_string()),
        })
    }
}

/// Repair path (#243): restore a corrupted marker from its pre-migration
/// backup when the backup is a valid v1 marker for the same root. Never
/// invents identity — a backup with different identity is refused.
pub fn repair_marker(root: &Path) -> Result<MarkerWriteOutcome> {
    match read_marker(root) {
        MarkerReadOutcome::Missing
        | MarkerReadOutcome::Valid
        | MarkerReadOutcome::LegacyMinimal { .. } => Ok(MarkerWriteOutcome::AlreadyValid),
        MarkerReadOutcome::Corrupted { error } => {
            let backup = root.join(".focusa-project.json.pre-migration");
            if !backup.is_file() {
                anyhow::bail!("marker corrupted ({error}) and no pre-migration backup exists");
            }
            let raw = fs::read_to_string(&backup)
                .with_context(|| format!("read marker backup {}", backup.display()))?;
            let restored: ProjectMarker = serde_json::from_str(&raw)
                .with_context(|| "marker backup is not a valid project marker")?;
            if restored.schema != MARKER_SCHEMA {
                anyhow::bail!("marker backup has unknown schema");
            }
            let restored_root = Path::new(&restored.project_root);
            if restored_root.is_absolute() && restored_root != root {
                anyhow::bail!(
                    "marker backup identity mismatch: backup root {} != {}",
                    restored.project_root,
                    root.display()
                );
            }
            let serialized = serde_json::to_vec_pretty(&restored)?;
            let temp = root.join(format!(".focusa-project.json.tmp-{}", std::process::id()));
            fs::write(&temp, serialized)?;
            if let Ok(file) = fs::File::open(&temp) {
                let _ = file.sync_all();
            }
            fs::rename(&temp, root.join(MARKER_FILE))
                .with_context(|| "atomically restore marker from backup")?;
            Ok(MarkerWriteOutcome::Written {
                backup: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> PathBuf {
        std::env::temp_dir().join(format!(
            "focusa-marker-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        ))
    }

    fn marker_for(root: &Path) -> ProjectMarker {
        ProjectMarker {
            schema: MARKER_SCHEMA.into(),
            project_id: "test-project".into(),
            canonical_name: "Test Project".into(),
            project_root: root.display().to_string(),
            repo_remote: Some("https://github.com/example/test-project".into()),
            beads_prefix: Some("test-project".into()),
            workspace_kind: Some("repo".into()),
            continuity_id: Some("test-continuity".into()),
            aliases: vec![],
            created_at: "2026-08-15T00:00:00Z".into(),
            updated_at: Some("2026-08-15T00:00:00Z".into()),
        }
    }

    #[test]
    fn create_is_atomic_and_idempotent() {
        let root = fixture();
        fs::create_dir_all(&root).unwrap();
        let outcome = write_marker(&root, &marker_for(&root), &MarkerWriteOptions::default())
            .unwrap();
        assert_eq!(outcome, MarkerWriteOutcome::Created);
        assert_eq!(read_marker(&root), MarkerReadOutcome::Valid);
        // Idempotent second write.
        let again = write_marker(&root, &marker_for(&root), &MarkerWriteOptions::default())
            .unwrap();
        assert_eq!(again, MarkerWriteOutcome::AlreadyValid);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn legacy_minimal_is_detected_and_migrated_with_backup() {
        let root = fixture();
        fs::create_dir_all(&root).unwrap();
        let legacy = serde_json::json!({
            "schema": MARKER_SCHEMA,
            "project_id": "test-project",
            "canonical_name": "Test Project",
            "project_root": root.display().to_string(),
            "aliases": [],
            "created_at": "2026-08-15T00:00:00Z"
        });
        fs::write(
            root.join(MARKER_FILE),
            serde_json::to_vec_pretty(&legacy).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            read_marker(&root),
            MarkerReadOutcome::LegacyMinimal { .. }
        ));
        let outcome = write_marker(
            &root,
            &marker_for(&root),
            &MarkerWriteOptions {
                preview: false,
                backup: true,
                overwrite: false,
            },
        )
        .unwrap();
        assert!(matches!(outcome, MarkerWriteOutcome::Migrated { .. }));
        assert!(root.join(".focusa-project.json.pre-migration").is_file());
        assert_eq!(read_marker(&root), MarkerReadOutcome::Valid);
        // Identity preserved.
        let migrated: ProjectMarker =
            serde_json::from_str(&fs::read_to_string(root.join(MARKER_FILE)).unwrap()).unwrap();
        assert_eq!(migrated.project_id, "test-project");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn conflicting_identity_is_refused() {
        let root = fixture();
        fs::create_dir_all(&root).unwrap();
        write_marker(&root, &marker_for(&root), &MarkerWriteOptions::default()).unwrap();
        let mut other = marker_for(&root);
        other.project_id = "different-project".into();
        let error = write_marker(&root, &other, &MarkerWriteOptions::default()).unwrap_err();
        assert!(error.to_string().contains("marker conflict"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_and_corrupted_are_classified() {
        let root = fixture();
        fs::create_dir_all(&root).unwrap();
        assert_eq!(read_marker(&root), MarkerReadOutcome::Missing);
        fs::write(root.join(MARKER_FILE), "{not json").unwrap();
        assert!(matches!(
            read_marker(&root),
            MarkerReadOutcome::Corrupted { .. }
        ));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn repair_restores_corrupted_marker_from_backup() {
        let root = fixture();
        fs::create_dir_all(&root).unwrap();
        // Seed a legacy marker, then migrate with backup enabled — the
        // migration is what produces the pre-migration backup.
        let legacy = serde_json::json!({
            "schema": MARKER_SCHEMA,
            "project_id": "test-project",
            "canonical_name": "Test Project",
            "project_root": root.display().to_string(),
            "aliases": [],
            "created_at": "2026-08-15T00:00:00Z"
        });
        fs::write(
            root.join(MARKER_FILE),
            serde_json::to_vec_pretty(&legacy).unwrap(),
        )
        .unwrap();
        write_marker(
            &root,
            &marker_for(&root),
            &MarkerWriteOptions {
                backup: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(root.join(".focusa-project.json.pre-migration").is_file());
        // Corrupt the live marker.
        fs::write(root.join(MARKER_FILE), "{corrupted").unwrap();
        assert!(matches!(
            read_marker(&root),
            MarkerReadOutcome::Corrupted { .. }
        ));
        let outcome = repair_marker(&root).unwrap();
        assert_eq!(outcome, MarkerWriteOutcome::Written { backup: None });
        // The pre-migration backup contains the legacy (pre-enrichment)
        // marker — repair restores exactly that, preserving identity.
        assert!(matches!(
            read_marker(&root),
            MarkerReadOutcome::LegacyMinimal { .. }
        ));
        let restored: ProjectMarker =
            serde_json::from_str(&fs::read_to_string(root.join(MARKER_FILE)).unwrap()).unwrap();
        assert_eq!(restored.project_id, "test-project");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn repair_refuses_backup_with_different_identity() {
        let root = fixture();
        fs::create_dir_all(&root).unwrap();
        let legacy = serde_json::json!({
            "schema": MARKER_SCHEMA,
            "project_id": "test-project",
            "canonical_name": "Test Project",
            "project_root": root.display().to_string(),
            "aliases": [],
            "created_at": "2026-08-15T00:00:00Z"
        });
        fs::write(
            root.join(MARKER_FILE),
            serde_json::to_vec_pretty(&legacy).unwrap(),
        )
        .unwrap();
        write_marker(
            &root,
            &marker_for(&root),
            &MarkerWriteOptions {
                backup: true,
                ..Default::default()
            },
        )
        .unwrap();
        let mut tampered = marker_for(&root);
        tampered.project_root = "/somewhere/else".into();
        fs::write(
            root.join(".focusa-project.json.pre-migration"),
            serde_json::to_vec_pretty(&tampered).unwrap(),
        )
        .unwrap();
        fs::write(root.join(MARKER_FILE), "{corrupted").unwrap();
        let error = repair_marker(&root).unwrap_err();
        assert!(error.to_string().contains("identity mismatch"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn preview_never_writes() {
        let root = fixture();
        fs::create_dir_all(&root).unwrap();
        let outcome = write_marker(
            &root,
            &marker_for(&root),
            &MarkerWriteOptions {
                preview: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(outcome, MarkerWriteOutcome::Created);
        assert!(!root.join(MARKER_FILE).exists());
        let _ = fs::remove_dir_all(&root);
    }
}
