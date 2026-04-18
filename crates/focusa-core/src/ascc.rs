//! ASCC — Anchored Structured Context Checkpointing.
//!
//! Source: G1-07-ascc.md
//!
//! ASCC maintains a persistent structured summary per focus frame that:
//! - replaces linear chat history in prompts,
//! - updates incrementally using anchors,
//! - preserves high-fidelity task continuity.
//!
//! Provides:
//!   - Conversion: FocusState ↔ AsccSections
//!   - Serializers: to_string_compact(), to_messages_slots(), to_digest()
//!   - CheckpointRecord lifecycle: create, update from delta, persist

use crate::types::*;
use chrono::Utc;

// ─── FocusState ↔ AsccSections Conversion ───────────────────────────────────

impl From<&FocusState> for AsccSections {
    fn from(fs: &FocusState) -> Self {
        AsccSections {
            intent: fs.intent.clone(),
            current_focus: fs.current_state.clone(),
            decisions: fs.decisions.clone(),
            artifacts: fs.artifacts.clone(),
            constraints: fs.constraints.clone(),
            open_questions: fs.open_questions.clone(),
            next_steps: fs.next_steps.clone(),
            recent_results: fs.recent_results.clone(),
            failures: fs.failures.clone(),
            notes: fs.notes.clone(),
            slot_meta: AsccSlotMetadata::default(),
        }
    }
}

// ─── Serializers (G1-07 §Prompt Serialization) ─────────────────────────────

impl AsccSections {
    /// Compact string serialization for prompt assembly.
    ///
    /// Per G1-07: "ASCC must serialize into a compact slot form (no fluff)"
    pub fn to_string_compact(&self, title: &str) -> String {
        let mut out = format!("FOCUS FRAME: {}\n", title);

        if !self.intent.is_empty() {
            out.push_str(&format!("INTENT: {}\n", self.intent));
        }
        if !self.current_focus.is_empty() {
            out.push_str(&format!("CURRENT_FOCUS: {}\n", self.current_focus));
        }
        append_list(&mut out, "DECISIONS", &self.decisions);
        append_list(&mut out, "CONSTRAINTS", &self.constraints);
        append_list(&mut out, "OPEN_QUESTIONS", &self.open_questions);
        append_list(&mut out, "NEXT_STEPS", &self.next_steps);
        append_list(&mut out, "RECENT_RESULTS", &self.recent_results);
        append_list(&mut out, "FAILURES", &self.failures);
        if !self.artifacts.is_empty() {
            out.push_str("ARTIFACTS:\n");
            for a in &self.artifacts {
                out.push_str(&format!(
                    "  - [{}] {}\n",
                    artifact_kind_str(a.kind),
                    a.label
                ));
            }
        }
        append_list(&mut out, "NOTES", &self.notes);

        out
    }

    /// Message-slot serialization for chat-format prompt assembly.
    ///
    /// Per G1-07: "to_messages_slots()" — returns vec of (role, content) pairs.
    pub fn to_messages_slots(&self, title: &str) -> Vec<(String, String)> {
        let mut slots = Vec::new();
        slots.push(("system".into(), format!("FOCUS_FRAME: {}", title)));

        if !self.intent.is_empty() {
            slots.push(("system".into(), format!("INTENT: {}", self.intent)));
        }
        if !self.current_focus.is_empty() {
            slots.push((
                "system".into(),
                format!("CURRENT_FOCUS: {}", self.current_focus),
            ));
        }
        if !self.decisions.is_empty() {
            let items: Vec<String> = self.decisions.iter().map(|d| format!("- {}", d)).collect();
            slots.push(("system".into(), format!("DECISIONS:\n{}", items.join("\n"))));
        }
        if !self.constraints.is_empty() {
            let items: Vec<String> = self
                .constraints
                .iter()
                .map(|c| format!("- {}", c))
                .collect();
            slots.push((
                "system".into(),
                format!("CONSTRAINTS:\n{}", items.join("\n")),
            ));
        }

        slots
    }

    /// Ultra-compact fallback summary.
    ///
    /// Per G1-07 UPDATE: "ASCC must expose to_digest() → ultra-compact fallback
    /// summary. Used only when prompt budget cannot be satisfied."
    pub fn to_digest(&self, title: &str) -> String {
        if self.intent.is_empty() {
            format!("FOCUS: {}", title)
        } else {
            format!("FOCUS: {} — {}", title, self.intent)
        }
    }

    /// Returns true if all slots are empty (no content to serialize).
    pub fn is_empty(&self) -> bool {
        self.intent.is_empty()
            && self.current_focus.is_empty()
            && self.decisions.is_empty()
            && self.artifacts.is_empty()
            && self.constraints.is_empty()
            && self.open_questions.is_empty()
            && self.next_steps.is_empty()
            && self.recent_results.is_empty()
            && self.failures.is_empty()
            && self.notes.is_empty()
    }
}

// ─── CheckpointRecord Lifecycle ─────────────────────────────────────────────

impl CheckpointRecord {
    /// Create a new checkpoint from a frame's current FocusState.
    pub fn from_frame(frame: &FrameRecord, turn_id: &str) -> Self {
        CheckpointRecord {
            frame_id: frame.id,
            revision: 1,
            updated_at: Utc::now(),
            anchor_turn_id: turn_id.to_string(),
            sections: AsccSections::from(&frame.focus_state),
            breadcrumbs: vec![],
        }
    }

    /// Update checkpoint from the frame's current FocusState after a delta was applied.
    ///
    /// Per G1-07: "revision += 1, anchor_turn_id = turn_id, updated_at = now"
    pub fn update_from_frame(&mut self, frame: &FrameRecord, turn_id: &str) {
        self.sections = AsccSections::from(&frame.focus_state);
        self.revision += 1;
        self.anchor_turn_id = turn_id.to_string();
        self.updated_at = Utc::now();
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn append_list(out: &mut String, label: &str, items: &[String]) {
    if !items.is_empty() {
        out.push_str(&format!("{}:\n", label));
        for item in items {
            out.push_str(&format!("  - {}\n", item));
        }
    }
}

pub fn artifact_kind_str(kind: ArtifactLineKind) -> &'static str {
    match kind {
        ArtifactLineKind::File => "file",
        ArtifactLineKind::Diff => "diff",
        ArtifactLineKind::Log => "log",
        ArtifactLineKind::Url => "url",
        ArtifactLineKind::Handle => "handle",
        ArtifactLineKind::Other => "other",
    }
}

// ─── File Persistence (G1-07 §Persistence) ─────────────────────────────────────

/// Save a checkpoint to ~/.focusa/ascc/<frame_id>.json
///
/// Per G1-07: "Checkpoint per frame stored in: ~/.focusa/ascc/<frame_id>.json.
/// MVP: only current checkpoint required."
pub fn save_checkpoint(data_dir: &str, checkpoint: &CheckpointRecord) -> std::io::Result<()> {
    let expanded = if data_dir.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        data_dir.replacen('~', &home, 1)
    } else {
        data_dir.to_string()
    };
    let dir = std::path::PathBuf::from(&expanded).join("ascc");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", checkpoint.frame_id));
    let json = serde_json::to_string_pretty(checkpoint).map_err(std::io::Error::other)?;
    std::fs::write(path, json)
}

/// Load a checkpoint from ~/.focusa/ascc/<frame_id>.json
pub fn load_checkpoint(
    data_dir: &str,
    frame_id: uuid::Uuid,
) -> std::io::Result<Option<CheckpointRecord>> {
    let expanded = if data_dir.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        data_dir.replacen('~', &home, 1)
    } else {
        data_dir.to_string()
    };
    let path = std::path::PathBuf::from(&expanded)
        .join("ascc")
        .join(format!("{}.json", frame_id));
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(path)?;
    let cp: CheckpointRecord = serde_json::from_str(&json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Some(cp))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> FocusState {
        FocusState {
            intent: "Implement user auth".into(),
            current_state: "Working on OAuth flow".into(),
            decisions: vec!["Use JWT tokens".into(), "PKCE for mobile".into()],
            constraints: vec!["Must support refresh tokens".into()],
            open_questions: vec!["Which OAuth provider?".into()],
            next_steps: vec!["Add token refresh endpoint".into()],
            recent_results: vec!["Login endpoint working".into()],
            failures: vec![],
            notes: vec![],
            artifacts: vec![],
        }
    }

    #[test]
    fn test_focus_state_to_ascc_sections() {
        let fs = sample_state();
        let ascc = AsccSections::from(&fs);
        assert_eq!(ascc.intent, "Implement user auth");
        assert_eq!(ascc.current_focus, "Working on OAuth flow");
        assert_eq!(ascc.decisions.len(), 2);
        assert_eq!(ascc.constraints.len(), 1);
        assert_eq!(ascc.open_questions.len(), 1);
    }

    #[test]
    fn test_to_string_compact() {
        let fs = sample_state();
        let ascc = AsccSections::from(&fs);
        let s = ascc.to_string_compact("Auth Module");
        assert!(s.contains("FOCUS FRAME: Auth Module"));
        assert!(s.contains("INTENT: Implement user auth"));
        assert!(s.contains("CURRENT_FOCUS: Working on OAuth flow"));
        assert!(s.contains("Use JWT tokens"));
        assert!(s.contains("CONSTRAINTS:"));
        assert!(s.contains("OPEN_QUESTIONS:"));
    }

    #[test]
    fn test_to_digest() {
        let fs = sample_state();
        let ascc = AsccSections::from(&fs);
        let d = ascc.to_digest("Auth Module");
        assert_eq!(d, "FOCUS: Auth Module — Implement user auth");
    }

    #[test]
    fn test_to_digest_empty_intent() {
        let ascc = AsccSections::default();
        let d = ascc.to_digest("Task");
        assert_eq!(d, "FOCUS: Task");
    }

    #[test]
    fn test_is_empty() {
        assert!(AsccSections::default().is_empty());
        let fs = sample_state();
        let ascc = AsccSections::from(&fs);
        assert!(!ascc.is_empty());
    }

    #[test]
    fn test_to_messages_slots() {
        let fs = sample_state();
        let ascc = AsccSections::from(&fs);
        let slots = ascc.to_messages_slots("Auth");
        assert!(slots.len() >= 3); // frame + intent + current_focus + decisions + constraints
        assert!(slots[0].1.contains("Auth"));
    }

    #[test]
    fn test_checkpoint_from_frame() {
        let frame = FrameRecord {
            id: uuid::Uuid::now_v7(),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: FrameStatus::Active,
            title: "Test".into(),
            goal: "Test goal".into(),
            beads_issue_id: "BEAD-1".into(),
            tags: vec![],
            priority_hint: None,
            ascc_checkpoint_id: None,
            stats: FrameStats::default(),
            constraints: vec![],
            focus_state: sample_state(),
            completed_at: None,
            completion_reason: None,
        };
        let cp = CheckpointRecord::from_frame(&frame, "turn-001");
        assert_eq!(cp.frame_id, frame.id);
        assert_eq!(cp.revision, 1);
        assert_eq!(cp.anchor_turn_id, "turn-001");
        assert_eq!(cp.sections.intent, "Implement user auth");
    }

    #[test]
    fn test_checkpoint_update_increments_revision() {
        let frame = FrameRecord {
            id: uuid::Uuid::now_v7(),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: FrameStatus::Active,
            title: "Test".into(),
            goal: "Test goal".into(),
            beads_issue_id: "BEAD-1".into(),
            tags: vec![],
            priority_hint: None,
            ascc_checkpoint_id: None,
            stats: FrameStats::default(),
            constraints: vec![],
            focus_state: sample_state(),
            completed_at: None,
            completion_reason: None,
        };
        let mut cp = CheckpointRecord::from_frame(&frame, "turn-001");
        assert_eq!(cp.revision, 1);

        cp.update_from_frame(&frame, "turn-002");
        assert_eq!(cp.revision, 2);
        assert_eq!(cp.anchor_turn_id, "turn-002");
    }
}
