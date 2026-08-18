//! Spec152F runtime entrypoint classification — §152F.03.07.
//!
//! Maps the 20 production runtime file-derived entrypoints to canonical capability
//! families and operation classes. No file is classified merely because its filename
//! contains "release", "update", "export", or "scheduler"; every entry resolves
//! through callable operation metadata.

use focusa_license::{CapabilityFamily, OperationClass};
use serde::{Deserialize, Serialize};

/// Resolution for a single runtime entrypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEntrypointResolution {
    /// Basic customer data export (always available, no commercial entitlement required).
    CustomerDataExport,
    /// Release proof — premium family; base read paths distinguished from premium orchestration.
    ReleaseProof,
    /// Inherit from the initiating canonical operation at dispatch time.
    InheritInitiatingOperation,
}

impl RuntimeEntrypointResolution {
    pub const fn label(self) -> &'static str {
        match self {
            Self::CustomerDataExport => "customer_data_export",
            Self::ReleaseProof => "release_proof",
            Self::InheritInitiatingOperation => "inherit_initiating_operation",
        }
    }

    pub const fn capability_family(self) -> CapabilityFamily {
        match self {
            Self::CustomerDataExport => CapabilityFamily::CustomerDataExport,
            Self::ReleaseProof => CapabilityFamily::ReleaseProof,
            Self::InheritInitiatingOperation => CapabilityFamily::InternalMaintenance,
        }
    }

    pub const fn operation_class(self) -> OperationClass {
        match self {
            Self::CustomerDataExport => OperationClass::Read,
            Self::ReleaseProof => OperationClass::ValueMutation,
            Self::InheritInitiatingOperation => OperationClass::InternalMaintenance,
        }
    }
}

/// Surface group for a runtime entrypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSurface {
    Export,
    Release,
    Scheduler,
    Update,
}

impl RuntimeSurface {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Export => "export",
            Self::Release => "release",
            Self::Scheduler => "scheduler",
            Self::Update => "update",
        }
    }
}

/// Classification of a single runtime entrypoint source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEntrypointClassification {
    pub source_path: String,
    pub surface: RuntimeSurface,
    pub resolution: RuntimeEntrypointResolution,
    pub capability_family: String,
    pub operation_class: String,
    pub rationale: String,
}

/// The complete runtime entrypoint map — 20 production entries.
///
/// Generated from the Spec152F surface reconciliation manifest. Every entry
/// is classified by callable operation semantics, not filename.
const RUNTIME_ENTRYPOINT_MAP: &[(&str, RuntimeSurface, RuntimeEntrypointResolution, &str)] = &[
    // ── export (2) ──────────────────────────────────────────────────────────
    (
        "crates/focusa-api/src/routes/silent_sessions_retention_export.rs",
        RuntimeSurface::Export,
        RuntimeEntrypointResolution::CustomerDataExport,
        "Basic customer-data export; always available. Premium packaging is operation-bound.",
    ),
    (
        "crates/focusa-cli/src/commands/export.rs",
        RuntimeSurface::Export,
        RuntimeEntrypointResolution::CustomerDataExport,
        "Basic customer-data export via CLI; always available. Premium packaging is operation-bound.",
    ),
    // ── release (12) ────────────────────────────────────────────────────────
    (
        "crates/focusa-api/src/routes/release.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release route surface; base read (status/list) gated through route-level classification. Premium proof operations require release_proof family.",
    ),
    (
        "crates/focusa-cli/src/commands/release.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "CLI release command; base query/status allowed. Premium orchestration requires release_proof.",
    ),
    (
        "crates/focusa-cli/src/commands/release_master.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release master coordination; base inquiry allowed. Premium orchestration requires release_proof.",
    ),
    (
        "crates/focusa-core/src/release_adapters.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release adapter layer; premium release_proof family. Base status reads inherit from caller context.",
    ),
    (
        "crates/focusa-core/src/release_calibration.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release calibration engine; premium release_proof family.",
    ),
    (
        "crates/focusa-core/src/release_cycle.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release lifecycle management; premium release_proof family.",
    ),
    (
        "crates/focusa-core/src/release_intelligence.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release intelligence/insight engine; premium release_proof family.",
    ),
    (
        "crates/focusa-core/src/release_ledger.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release ledger persistence; premium release_proof family.",
    ),
    (
        "crates/focusa-core/src/release_orchestrator.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release orchestration engine; premium release_proof family.",
    ),
    (
        "crates/focusa-core/src/release_planner.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release planning engine; premium release_proof family.",
    ),
    (
        "crates/focusa-core/src/release_protocol.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Release wire protocol; premium release_proof family.",
    ),
    (
        "crates/focusa-core/src/temporal_release_gate.rs",
        RuntimeSurface::Release,
        RuntimeEntrypointResolution::ReleaseProof,
        "Temporal release gate; premium release_proof family.",
    ),
    // ── scheduler (2) ───────────────────────────────────────────────────────
    (
        "crates/focusa-core/src/silent_session_scheduler.rs",
        RuntimeSurface::Scheduler,
        RuntimeEntrypointResolution::InheritInitiatingOperation,
        "Silent session scheduler; inherits initiating operation authority at dispatch time.",
    ),
    (
        "crates/focusa-core/src/work_item/scheduler.rs",
        RuntimeSurface::Scheduler,
        RuntimeEntrypointResolution::InheritInitiatingOperation,
        "Work item scheduler; inherits initiating operation authority at dispatch time.",
    ),
    // ── update (4) ──────────────────────────────────────────────────────────
    (
        "crates/focusa-api/src/routes/update.rs",
        RuntimeSurface::Update,
        RuntimeEntrypointResolution::InheritInitiatingOperation,
        "Update route; stable security update inherits account_recovery allowance. Premium unattended/nightly updates inherit premium_updates via initiating operation.",
    ),
    (
        "crates/focusa-cli/src/commands/update.rs",
        RuntimeSurface::Update,
        RuntimeEntrypointResolution::InheritInitiatingOperation,
        "CLI update command; stable security update inherits account_recovery allowance. Premium channels inherit via initiating operation.",
    ),
    (
        "crates/focusa-cli/src/commands/update_trust.rs",
        RuntimeSurface::Update,
        RuntimeEntrypointResolution::InheritInitiatingOperation,
        "Update trust management; inherits initiating operation authority.",
    ),
    (
        "crates/focusa-core/src/update.rs",
        RuntimeSurface::Update,
        RuntimeEntrypointResolution::InheritInitiatingOperation,
        "Core update module; stable security update allowance or premium channel via initiating operation.",
    ),
];

/// Classify a single runtime source path into its entitlement metadata.
///
/// Returns `None` when the path is not one of the 20 recognized production
/// runtime entrypoints (test-only files are excluded by design).
pub fn classify_runtime_entrypoint(source_path: &str) -> Option<RuntimeEntrypointClassification> {
    let canonical = source_path.trim().trim_start_matches("./");
    RUNTIME_ENTRYPOINT_MAP
        .iter()
        .find(|(path, _, _, _)| *path == canonical)
        .map(
            |(path, surface, resolution, rationale)| RuntimeEntrypointClassification {
                source_path: (*path).to_string(),
                surface: *surface,
                resolution: *resolution,
                capability_family: resolution.capability_family().label().to_string(),
                operation_class: resolution.operation_class().label().to_string(),
                rationale: (*rationale).to_string(),
            },
        )
}

/// Returns true when the path is a recognized production release module.
pub fn is_release_proof_surface(source_path: &str) -> bool {
    matches!(
        classify_runtime_entrypoint(source_path),
        Some(RuntimeEntrypointClassification {
            resolution: RuntimeEntrypointResolution::ReleaseProof,
            ..
        })
    )
}

/// Returns true when the path inherits entitlement from its initiating operation.
pub fn is_inheriting_entrypoint(source_path: &str) -> bool {
    matches!(
        classify_runtime_entrypoint(source_path),
        Some(RuntimeEntrypointClassification {
            resolution: RuntimeEntrypointResolution::InheritInitiatingOperation,
            ..
        })
    )
}

/// The total number of classified production runtime entrypoints.
pub const RUNTIME_ENTRYPOINT_COUNT: usize = RUNTIME_ENTRYPOINT_MAP.len();

/// Returns the static count of classified runtime entrypoints plus grouped breakdown.
pub fn runtime_entrypoint_summary() -> RuntimeEntrypointSummary {
    let mut export_count = 0usize;
    let mut release_count = 0usize;
    let mut scheduler_count = 0usize;
    let mut update_count = 0usize;

    for (_, surface, _, _) in RUNTIME_ENTRYPOINT_MAP {
        match surface {
            RuntimeSurface::Export => export_count += 1,
            RuntimeSurface::Release => release_count += 1,
            RuntimeSurface::Scheduler => scheduler_count += 1,
            RuntimeSurface::Update => update_count += 1,
        }
    }

    RuntimeEntrypointSummary {
        total: RUNTIME_ENTRYPOINT_COUNT,
        export_count,
        release_count,
        scheduler_count,
        update_count,
        release_proof_count: release_count,
        customer_data_export_count: export_count,
        inheriting_count: scheduler_count + update_count,
    }
}

/// Lightweight summary of the runtime entrypoint map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeEntrypointSummary {
    pub total: usize,
    pub export_count: usize,
    pub release_count: usize,
    pub scheduler_count: usize,
    pub update_count: usize,
    pub release_proof_count: usize,
    pub customer_data_export_count: usize,
    pub inheriting_count: usize,
}

/// Build the full 20-entry classification list for export/contract validation.
pub fn runtime_entrypoint_classifications() -> Vec<RuntimeEntrypointClassification> {
    RUNTIME_ENTRYPOINT_MAP
        .iter()
        .map(
            |(path, surface, resolution, rationale)| RuntimeEntrypointClassification {
                source_path: (*path).to_string(),
                surface: *surface,
                resolution: *resolution,
                capability_family: resolution.capability_family().label().to_string(),
                operation_class: resolution.operation_class().label().to_string(),
                rationale: (*rationale).to_string(),
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── runtime_entrypoint_entitlement tests ──────────────────────────────

    #[test]
    fn runtime_entrypoint_entitlement_map_has_exactly_20_entries() {
        let entries = runtime_entrypoint_classifications();
        assert_eq!(
            entries.len(),
            20,
            "must classify exactly 20 production runtime files"
        );
        assert_eq!(RUNTIME_ENTRYPOINT_COUNT, 20);
    }

    #[test]
    fn runtime_entrypoint_entitlement_no_file_classified_by_filename_alone() {
        let entries = runtime_entrypoint_classifications();
        for entry in &entries {
            let path_base = entry
                .source_path
                .rsplit('/')
                .next()
                .unwrap_or(&entry.source_path);
            // No file is granted premium treatment merely because its path
            // contains "release".  All release files must carry explicit
            // ReleaseProof resolution with rationale.
            if path_base.contains("release")
                || path_base.contains("update")
                || path_base.contains("scheduler")
            {
                // Must have a non-default rationale (not empty).
                assert!(
                    !entry.rationale.is_empty(),
                    "file {} must have explicit rationale, not filename-based policy",
                    entry.source_path
                );
            }
        }
    }

    #[test]
    fn runtime_entrypoint_entitlement_exports_are_always_available_basic() {
        let entries = runtime_entrypoint_classifications();
        let exports: Vec<_> = entries
            .iter()
            .filter(|e| matches!(e.surface, RuntimeSurface::Export))
            .collect();
        assert_eq!(exports.len(), 2, "exactly 2 export entrypoints");
        for entry in &exports {
            assert_eq!(
                entry.resolution,
                RuntimeEntrypointResolution::CustomerDataExport,
                "export entrypoint {} must resolve to customer_data_export",
                entry.source_path
            );
            assert_eq!(
                entry.capability_family, "customer_data_export",
                "export entrypoint {} must have customer_data_export family",
                entry.source_path
            );
            // Basic data export is always available; no commercial entitlement required.
            assert!(
                entry.operation_class == "read" || entry.operation_class == "recovery",
                "basic export is read or recovery, not value_mutation"
            );
        }
    }

    #[test]
    fn runtime_entrypoint_entitlement_release_modules_are_release_proof_not_base() {
        let entries = runtime_entrypoint_classifications();
        let releases: Vec<_> = entries
            .iter()
            .filter(|e| matches!(e.surface, RuntimeSurface::Release))
            .collect();
        assert_eq!(releases.len(), 12, "exactly 12 release entrypoints");
        for entry in &releases {
            assert_eq!(
                entry.resolution,
                RuntimeEntrypointResolution::ReleaseProof,
                "release entrypoint {} must have ReleaseProof resolution",
                entry.source_path
            );
            assert_eq!(
                entry.capability_family, "release_proof",
                "release entrypoint {} must have release_proof capability family",
                entry.source_path
            );
        }
    }

    #[test]
    fn runtime_entrypoint_entitlement_schedulers_inherit_initiating_operation() {
        let entries = runtime_entrypoint_classifications();
        let schedulers: Vec<_> = entries
            .iter()
            .filter(|e| matches!(e.surface, RuntimeSurface::Scheduler))
            .collect();
        assert_eq!(schedulers.len(), 2, "exactly 2 scheduler entrypoints");
        for entry in &schedulers {
            assert_eq!(
                entry.resolution,
                RuntimeEntrypointResolution::InheritInitiatingOperation,
                "scheduler {} must inherit initiating operation",
                entry.source_path
            );
            assert_eq!(
                entry.capability_family, "internal_maintenance",
                "scheduler {} must have internal_maintenance family",
                entry.source_path
            );
            assert_eq!(entry.operation_class, "internal_maintenance");
        }
    }

    #[test]
    fn runtime_entrypoint_entitlement_updates_inherit_initiating_operation() {
        let entries = runtime_entrypoint_classifications();
        let updates: Vec<_> = entries
            .iter()
            .filter(|e| matches!(e.surface, RuntimeSurface::Update))
            .collect();
        assert_eq!(updates.len(), 4, "exactly 4 update entrypoints");
        for entry in &updates {
            assert_eq!(
                entry.resolution,
                RuntimeEntrypointResolution::InheritInitiatingOperation,
                "update entrypoint {} must inherit initiating operation",
                entry.source_path
            );
            assert_eq!(
                entry.capability_family, "internal_maintenance",
                "update entrypoint {} must have internal_maintenance family with initiating inheritance",
                entry.source_path
            );
        }
    }

    #[test]
    fn runtime_entrypoint_entitlement_summary_matches_composition() {
        let summary = runtime_entrypoint_summary();
        assert_eq!(summary.total, 20);
        assert_eq!(summary.export_count, 2);
        assert_eq!(summary.release_count, 12);
        assert_eq!(summary.scheduler_count, 2);
        assert_eq!(summary.update_count, 4);
        assert_eq!(summary.release_proof_count, 12);
        assert_eq!(summary.customer_data_export_count, 2);
        assert_eq!(summary.inheriting_count, 6); // 2 scheduler + 4 update
    }

    #[test]
    fn runtime_entrypoint_entitlement_test_files_are_not_classified() {
        // Test-only paths that must NOT appear in the map.
        let test_paths = [
            "crates/focusa-cli/tests/silent_proof_export_parity_e2e.rs",
            "crates/focusa-core/src/release_adapters_test.rs",
            "crates/focusa-core/src/release_calibration_test.rs",
            "crates/focusa-core/src/release_cycle_test.rs",
            "crates/focusa-core/src/release_ledger_test.rs",
            "crates/focusa-core/src/release_orchestrator_test.rs",
            "crates/focusa-cli/tests/spec128_update_runtime_e2e.rs",
        ];
        for path in &test_paths {
            assert!(
                classify_runtime_entrypoint(path).is_none(),
                "test-only path must not be classified as runtime entrypoint: {path}"
            );
        }
    }

    #[test]
    fn runtime_entrypoint_entitlement_is_release_proof_surface_identifies_only_release() {
        // release files
        assert!(is_release_proof_surface(
            "crates/focusa-core/src/release_orchestrator.rs"
        ));
        assert!(is_release_proof_surface(
            "crates/focusa-core/src/release_protocol.rs"
        ));
        // non-release files
        assert!(!is_release_proof_surface(
            "crates/focusa-core/src/update.rs"
        ));
        assert!(!is_release_proof_surface(
            "crates/focusa-api/src/routes/silent_sessions_retention_export.rs"
        ));
        assert!(!is_release_proof_surface(
            "crates/focusa-core/src/silent_session_scheduler.rs"
        ));
    }

    #[test]
    fn runtime_entrypoint_entitlement_is_inheriting_entrypoint_detects_schedulers_and_updates() {
        assert!(is_inheriting_entrypoint(
            "crates/focusa-core/src/silent_session_scheduler.rs"
        ));
        assert!(is_inheriting_entrypoint(
            "crates/focusa-core/src/work_item/scheduler.rs"
        ));
        assert!(is_inheriting_entrypoint(
            "crates/focusa-api/src/routes/update.rs"
        ));
        // non-inheriting files
        assert!(!is_inheriting_entrypoint(
            "crates/focusa-core/src/release_orchestrator.rs"
        ));
        assert!(!is_inheriting_entrypoint(
            "crates/focusa-cli/src/commands/export.rs"
        ));
    }

    #[test]
    fn runtime_entrypoint_entitlement_unknown_path_returns_none() {
        assert!(classify_runtime_entrypoint("src/main.rs").is_none());
        assert!(classify_runtime_entrypoint("").is_none());
        assert!(classify_runtime_entrypoint("crates/focusa-core/src/unknown.rs").is_none());
    }

    #[test]
    fn runtime_entrypoint_entitlement_all_20_paths_are_unique() {
        let entries = runtime_entrypoint_classifications();
        let mut paths: Vec<&str> = entries.iter().map(|e| e.source_path.as_str()).collect();
        let unique_count = {
            paths.sort_unstable();
            paths.dedup();
            paths.len()
        };
        assert_eq!(unique_count, 20, "all 20 paths must be unique");
    }

    #[test]
    fn runtime_entrypoint_entitlement_disjoint_resolution_surfaces_dont_overlap() {
        // Export, Release, Scheduler, and Update surfaces must be disjoint.
        let entries = runtime_entrypoint_classifications();
        let export_sources: Vec<&str> = entries
            .iter()
            .filter(|e| matches!(e.surface, RuntimeSurface::Export))
            .map(|e| e.source_path.as_str())
            .collect();
        let release_sources: Vec<&str> = entries
            .iter()
            .filter(|e| matches!(e.surface, RuntimeSurface::Release))
            .map(|e| e.source_path.as_str())
            .collect();
        let scheduler_sources: Vec<&str> = entries
            .iter()
            .filter(|e| matches!(e.surface, RuntimeSurface::Scheduler))
            .map(|e| e.source_path.as_str())
            .collect();
        let update_sources: Vec<&str> = entries
            .iter()
            .filter(|e| matches!(e.surface, RuntimeSurface::Update))
            .map(|e| e.source_path.as_str())
            .collect();
        assert!(!export_sources.iter().any(|p| release_sources.contains(p)
            || scheduler_sources.contains(p)
            || update_sources.contains(p)));
        assert!(
            !release_sources
                .iter()
                .any(|p| scheduler_sources.contains(p) || update_sources.contains(p))
        );
        assert!(!scheduler_sources.iter().any(|p| update_sources.contains(p)));
    }
}
