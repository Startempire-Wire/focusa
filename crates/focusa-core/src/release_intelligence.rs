//! Evidence-backed release intelligence and deterministic release-page rendering.

use anyhow::ensure;
use serde::{Deserialize, Serialize};

use crate::release_cycle::{RELEASE_INTELLIGENCE_SCHEMA, ReleaseBenchmark};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseArtifactTruth {
    pub surface_id: String,
    pub artifact_name: String,
    pub platform: String,
    pub sha256: String,
    pub signature_ref: String,
    pub provenance_ref: String,
    pub installed_version: Option<String>,
    pub running_version: Option<String>,
    pub verification_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseIntelligencePacket {
    pub schema: String,
    pub release_id: String,
    pub project_id: String,
    pub profile: String,
    pub version: String,
    pub exact_sha: String,
    pub previous_tag: Option<String>,
    pub purpose: String,
    pub trajectory_refs: Vec<String>,
    pub material_changes: Vec<String>,
    pub impact: Vec<String>,
    pub included_work: Vec<String>,
    pub resolved_work: Vec<String>,
    pub exact_proofs: Vec<String>,
    pub unproven_checks: Vec<String>,
    pub failed_checks: Vec<String>,
    pub known_issues: Vec<String>,
    pub breaking_changes: Vec<String>,
    pub compatibility: Vec<String>,
    pub migrations: Vec<String>,
    pub install_steps: Vec<String>,
    pub upgrade_steps: Vec<String>,
    pub rollback_steps: Vec<String>,
    pub artifacts: Vec<ReleaseArtifactTruth>,
    pub security_and_provenance: Vec<String>,
    pub contributors: Vec<String>,
    pub traceability_refs: Vec<String>,
    pub commits: Vec<String>,
    pub benchmark: Option<ReleaseBenchmark>,
}

impl ReleaseIntelligencePacket {
    pub fn validate(&self, publishable: bool) -> anyhow::Result<()> {
        ensure!(
            self.schema == RELEASE_INTELLIGENCE_SCHEMA,
            "unsupported release intelligence schema"
        );
        ensure!(!self.release_id.trim().is_empty(), "release_id is required");
        ensure!(!self.project_id.trim().is_empty(), "project_id is required");
        ensure!(
            !self.profile.trim().is_empty(),
            "release profile is required"
        );
        ensure!(
            !self.version.trim().is_empty(),
            "release version is required"
        );
        ensure!(self.exact_sha.len() >= 7, "exact release SHA is required");
        ensure!(
            !self.purpose.trim().is_empty(),
            "release purpose is required"
        );
        ensure!(
            !self.material_changes.is_empty(),
            "material changes are required"
        );
        ensure!(
            !self.exact_proofs.is_empty(),
            "exact release proofs are required"
        );
        ensure!(!self.artifacts.is_empty(), "artifact truth is required");
        for artifact in &self.artifacts {
            ensure!(
                !artifact.surface_id.trim().is_empty(),
                "artifact surface is required"
            );
            ensure!(
                !artifact.artifact_name.trim().is_empty(),
                "artifact name is required"
            );
            ensure!(
                artifact.sha256.len() == 64,
                "artifact SHA-256 must contain 64 hexadecimal characters"
            );
            ensure!(
                artifact
                    .sha256
                    .chars()
                    .all(|value| value.is_ascii_hexdigit()),
                "artifact SHA-256 is not hexadecimal"
            );
            ensure!(
                !artifact.signature_ref.trim().is_empty(),
                "artifact signature ref is required"
            );
            ensure!(
                !artifact.provenance_ref.trim().is_empty(),
                "artifact provenance ref is required"
            );
        }
        if publishable {
            ensure!(
                self.unproven_checks.is_empty(),
                "publishable release contains unproven checks"
            );
            ensure!(
                self.failed_checks.is_empty(),
                "publishable release contains failed checks"
            );
            ensure!(
                self.artifacts
                    .iter()
                    .all(|artifact| artifact.verification_ref.is_some()),
                "publishable release contains an unverified artifact"
            );
        }
        Ok(())
    }

    pub fn render_markdown(&self) -> anyhow::Result<String> {
        self.validate(false)?;
        let mut output = String::new();
        output.push_str(&format!("# {} — {}\n\n", self.project_id, self.version));
        output.push_str(&format!(
            "**Release:** `{}`  \n**SHA:** `{}`  \n**Profile:** `{}`\n\n",
            self.release_id, self.exact_sha, self.profile
        ));
        output.push_str(&format!("## Purpose\n\n{}\n\n", self.purpose));
        render_list(&mut output, "Material changes", &self.material_changes);
        render_list(&mut output, "Impact", &self.impact);
        render_list(&mut output, "Included work", &self.included_work);
        render_list(&mut output, "Resolved work", &self.resolved_work);
        render_list(&mut output, "Exact proof", &self.exact_proofs);
        render_list(&mut output, "Unproven checks", &self.unproven_checks);
        render_list(&mut output, "Failed checks", &self.failed_checks);
        render_list(&mut output, "Known issues", &self.known_issues);
        render_list(&mut output, "Breaking changes", &self.breaking_changes);
        render_list(&mut output, "Compatibility", &self.compatibility);
        render_list(&mut output, "Migrations", &self.migrations);
        render_list(&mut output, "Install", &self.install_steps);
        render_list(&mut output, "Upgrade", &self.upgrade_steps);
        render_list(&mut output, "Rollback", &self.rollback_steps);
        output.push_str("## Artifact truth\n\n");
        output.push_str(
            "| Surface | Artifact | Platform | SHA-256 | Installed | Running | Proof |\n",
        );
        output.push_str("|---|---|---|---|---|---|---|\n");
        for artifact in &self.artifacts {
            output.push_str(&format!(
                "| {} | `{}` | {} | `{}` | {} | {} | {} |\n",
                artifact.surface_id,
                artifact.artifact_name,
                artifact.platform,
                artifact.sha256,
                artifact
                    .installed_version
                    .as_deref()
                    .unwrap_or("not installed"),
                artifact.running_version.as_deref().unwrap_or("not running"),
                artifact.verification_ref.as_deref().unwrap_or("unproven")
            ));
        }
        output.push('\n');
        render_list(
            &mut output,
            "Security and provenance",
            &self.security_and_provenance,
        );
        render_list(
            &mut output,
            "Trajectory and traceability",
            &self.trajectory_refs,
        );
        render_list(&mut output, "Traceability refs", &self.traceability_refs);
        render_list(&mut output, "Commits", &self.commits);
        render_list(&mut output, "Contributors", &self.contributors);
        if let Some(benchmark) = &self.benchmark {
            output.push_str("## Release benchmark\n\n");
            output.push_str(&format!(
                "- Total elapsed: {} ms\n- Useful work: {} ms\n- Queue: {} ms\n- Retry: {} ms\n- Flow efficiency: {:.3}\n- First-pass gate success: {:.3}\n- Human interventions: {}\n- Retries: {}\n\n",
                benchmark.total_elapsed_ms,
                benchmark.useful_work_ms,
                benchmark.queue_ms,
                benchmark.retry_ms,
                benchmark.flow_efficiency,
                benchmark.first_pass_gate_success_rate,
                benchmark.human_interventions,
                benchmark.retries
            ));
        }
        Ok(output)
    }
}

fn render_list(output: &mut String, title: &str, items: &[String]) {
    output.push_str(&format!("## {title}\n\n"));
    if items.is_empty() {
        output.push_str("- None recorded.\n\n");
    } else {
        for item in items {
            output.push_str(&format!("- {item}\n"));
        }
        output.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet() -> ReleaseIntelligencePacket {
        ReleaseIntelligencePacket {
            schema: RELEASE_INTELLIGENCE_SCHEMA.into(),
            release_id: "release:focusa:v1".into(),
            project_id: "focusa".into(),
            profile: "multi_surface".into(),
            version: "1.0.0".into(),
            exact_sha: "0123456789abcdef".into(),
            previous_tag: None,
            purpose: "Prove deterministic release intelligence.".into(),
            trajectory_refs: vec!["trajectory:1".into()],
            material_changes: vec!["Typed release packet".into()],
            impact: vec!["Evidence-backed page".into()],
            included_work: vec!["issue:1".into()],
            resolved_work: vec!["bead:1".into()],
            exact_proofs: vec!["actions:1".into()],
            unproven_checks: vec![],
            failed_checks: vec![],
            known_issues: vec![],
            breaking_changes: vec![],
            compatibility: vec!["Compatible".into()],
            migrations: vec![],
            install_steps: vec!["Install signed asset".into()],
            upgrade_steps: vec!["Use verified OTA".into()],
            rollback_steps: vec!["Restore journal backup".into()],
            artifacts: vec![ReleaseArtifactTruth {
                surface_id: "cli".into(),
                artifact_name: "focusa".into(),
                platform: "linux".into(),
                sha256: "a".repeat(64),
                signature_ref: "signature:1".into(),
                provenance_ref: "provenance:1".into(),
                installed_version: Some("1.0.0".into()),
                running_version: None,
                verification_ref: Some("verify:1".into()),
            }],
            security_and_provenance: vec!["Signed".into()],
            contributors: vec!["operator".into()],
            traceability_refs: vec!["workpoint:1".into()],
            commits: vec!["0123456".into()],
            benchmark: None,
        }
    }

    #[test]
    fn publishable_packet_requires_verified_artifacts_and_clean_checks() {
        let mut value = packet();
        value.validate(true).unwrap();
        value.unproven_checks.push("windows".into());
        assert!(value.validate(true).is_err());
    }

    #[test]
    fn renderer_is_deterministic_and_evidence_visible() {
        let value = packet();
        let first = value.render_markdown().unwrap();
        let second = value.render_markdown().unwrap();
        assert_eq!(first, second);
        assert!(first.contains("## Artifact truth"));
        assert!(first.contains("actions:1"));
    }
}
