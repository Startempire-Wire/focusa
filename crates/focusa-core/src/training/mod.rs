//! Training Dataset Export — docs/20-21
//!
//! 4 dataset families: SFT, Preference, Contrastive, Long-Horizon.
//! Privacy: PII detection required before export.
//! Format: JSONL (one example per line).
//! All exports are user-reviewable.

use crate::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Build a training example from a turn.
pub fn build_example(
    family: DatasetFamily,
    session_id: SessionId,
    turn_id: &str,
    input: &str,
    output: &str,
    focus_before: Option<FocusState>,
    focus_after: Option<FocusState>,
) -> TrainingExample {
    TrainingExample {
        family,
        session_id,
        turn_id: turn_id.into(),
        input: input.into(),
        output: output.into(),
        focus_state_before: focus_before,
        focus_state_after: focus_after,
        uxp_signals: vec![],
        ufi_signals: vec![],
        lineage_path: vec![],
        created_at: Utc::now(),
    }
}

/// Basic PII detection (patterns to reject).
/// Returns list of detected PII patterns. Empty = safe.
pub fn detect_pii(text: &str) -> Vec<String> {
    let mut findings = Vec::new();

    // Email.
    if text.contains('@') && text.contains('.') {
        // Simple heuristic.
        for word in text.split_whitespace() {
            if word.contains('@') && word.contains('.') && word.len() > 5 {
                findings.push(format!("Possible email: {}", word));
            }
        }
    }

    // Phone (US-style).
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 10 {
        findings.push("Possible phone number detected".into());
    }

    // SSN pattern (NNN-NN-NNNN) — work on bytes to avoid multibyte UTF-8 panics.
    let bytes = text.as_bytes();
    if bytes.len() >= 11 {
        for i in 0..bytes.len() - 10 {
            if bytes[i + 3] == b'-'
                && bytes[i + 6] == b'-'
                && bytes[i..i + 3].iter().all(|b| b.is_ascii_digit())
                && bytes[i + 4..i + 6].iter().all(|b| b.is_ascii_digit())
                && bytes[i + 7..i + 11].iter().all(|b| b.is_ascii_digit())
            {
                findings.push("Possible SSN pattern detected".into());
                break;
            }
        }
    }

    findings
}

/// Serialize a training example to JSONL.
pub fn to_jsonl(example: &TrainingExample) -> Result<String, serde_json::Error> {
    serde_json::to_string(example)
}

/// Batch export with PII check. Returns (exported, rejected_count).
pub fn export_batch(examples: &[TrainingExample]) -> (Vec<String>, usize) {
    let mut lines = Vec::new();
    let mut rejected = 0;

    for ex in examples {
        let pii = detect_pii(&ex.input);
        let pii2 = detect_pii(&ex.output);
        if pii.is_empty() && pii2.is_empty() {
            if let Ok(line) = to_jsonl(ex) {
                lines.push(line);
            }
        } else {
            rejected += 1;
        }
    }

    (lines, rejected)
}

// ─── Data Contribution — docs/22 ────────────────────────────────────────────

/// Enqueue an item for contribution.
pub fn enqueue_contribution(state: &mut ContributionState, family: DatasetFamily) -> Uuid {
    let id = Uuid::now_v7();
    state.queue.push(ContributionItem {
        id,
        dataset_family: family,
        status: ContributionStatus::Pending,
        created_at: Utc::now(),
        reviewed: false,
    });
    id
}

/// Approve a contribution item (user review).
pub fn approve_contribution(state: &mut ContributionState, item_id: Uuid) -> Result<(), String> {
    let item = state
        .queue
        .iter_mut()
        .find(|i| i.id == item_id)
        .ok_or_else(|| format!("Contribution item {} not found", item_id))?;
    item.status = ContributionStatus::Approved;
    item.reviewed = true;
    Ok(())
}

/// Submit approved items. Returns count submitted.
pub fn submit_approved(state: &mut ContributionState) -> usize {
    let mut count = 0;
    for item in &mut state.queue {
        if item.status == ContributionStatus::Approved {
            item.status = ContributionStatus::Submitted;
            state.total_contributed += 1;
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_example() {
        let sid = uuid::Uuid::now_v7();
        let ex = build_example(
            DatasetFamily::FocusaSft,
            sid,
            "t1",
            "hello",
            "world",
            None,
            None,
        );
        assert_eq!(ex.family, DatasetFamily::FocusaSft);
    }

    #[test]
    fn test_pii_detection() {
        assert!(!detect_pii("user@example.com is here").is_empty());
        assert!(detect_pii("clean text about code").is_empty());
    }

    #[test]
    fn test_export_rejects_pii() {
        let sid = uuid::Uuid::now_v7();
        let examples = vec![
            build_example(
                DatasetFamily::FocusaSft,
                sid,
                "t1",
                "clean input",
                "clean output",
                None,
                None,
            ),
            build_example(
                DatasetFamily::FocusaSft,
                sid,
                "t2",
                "email user@test.com",
                "ok",
                None,
                None,
            ),
        ];
        let (lines, rejected) = export_batch(&examples);
        assert_eq!(lines.len(), 1);
        assert_eq!(rejected, 1);
    }
}
