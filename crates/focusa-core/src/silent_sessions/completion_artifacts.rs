use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{SilentSessionId, SilentSessionRunId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionArtifactRefs {
    pub bounded_transcript: String,
    pub redacted_stream_manifest: String,
    pub stdout_stderr_index: String,
    pub effective_config: String,
    pub model_binding: String,
    pub workpoint_history: String,
    pub git_summary: String,
    pub test_results: String,
    pub blocker_summary: String,
    pub completion_evaluation: String,
    pub receipt_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionArtifactManifest {
    pub schema: String,
    pub session_id: SilentSessionId,
    pub run_id: SilentSessionRunId,
    pub artifacts: CompletionArtifactRefs,
    pub manifest_hash: String,
}

impl CompletionArtifactManifest {
    pub fn build(
        session_id: SilentSessionId,
        run_id: SilentSessionRunId,
        artifacts: CompletionArtifactRefs,
    ) -> anyhow::Result<Self> {
        for value in [
            &artifacts.bounded_transcript,
            &artifacts.redacted_stream_manifest,
            &artifacts.stdout_stderr_index,
            &artifacts.effective_config,
            &artifacts.model_binding,
            &artifacts.workpoint_history,
            &artifacts.git_summary,
            &artifacts.test_results,
            &artifacts.blocker_summary,
            &artifacts.completion_evaluation,
        ] {
            if value.trim().is_empty() {
                anyhow::bail!("completion artifact reference must not be empty");
            }
        }
        if artifacts.receipt_refs.is_empty() {
            anyhow::bail!("completion receipt references are required");
        }
        let bytes = serde_json::to_vec(&(session_id, run_id, &artifacts))?;
        Ok(Self {
            schema: "focusa.silent_session_completion_artifacts.v1".into(),
            session_id,
            run_id,
            artifacts,
            manifest_hash: hex::encode(Sha256::digest(bytes)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refs() -> CompletionArtifactRefs {
        CompletionArtifactRefs {
            bounded_transcript: "ref:transcript".into(),
            redacted_stream_manifest: "ref:streams".into(),
            stdout_stderr_index: "ref:index".into(),
            effective_config: "ref:config".into(),
            model_binding: "ref:model".into(),
            workpoint_history: "ref:workpoints".into(),
            git_summary: "ref:git".into(),
            test_results: "ref:tests".into(),
            blocker_summary: "ref:blockers".into(),
            completion_evaluation: "ref:evaluation".into(),
            receipt_refs: vec!["ref:receipt".into()],
        }
    }

    #[test]
    fn manifest_requires_every_completion_artifact_and_is_deterministic() {
        let session = SilentSessionId::new();
        let run = SilentSessionRunId::new();
        let first = CompletionArtifactManifest::build(session, run, refs()).unwrap();
        let second = CompletionArtifactManifest::build(session, run, refs()).unwrap();
        assert_eq!(first.manifest_hash, second.manifest_hash);
        let mut incomplete = refs();
        incomplete.test_results.clear();
        assert!(CompletionArtifactManifest::build(session, run, incomplete).is_err());
    }
}
