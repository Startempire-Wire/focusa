//! Worker Job Executor — runs background cognition jobs.
//!
//! Jobs:
//!   - classify_turn: Classify a turn as task/question/correction/meta.
//!   - extract_ascc_delta: Extract ASCC delta from assistant response.
//!   - detect_ufi_signals: Detect UFI signals from user message.
//!   - compute_ari_observation: Score autonomy dimensions for a turn.
//!
//! All jobs run under strict time budget (default 200ms).
//! Results are returned via channel — reducer decides acceptance.

use crate::types::*;
use serde_json::json;
use uuid::Uuid;

/// Job execution result.
#[derive(Debug, Clone)]
pub struct JobResult {
    pub job_id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub success: bool,
}

/// Execute a worker job with panic isolation.
///
/// Per G1-10 §Safety: "workers must be panic-isolated, failure does not
/// affect daemon state"
///
/// Uses std::panic::catch_unwind to isolate panics. Returns failed result
/// if job panics.
pub fn execute_job(job: &WorkerJob) -> JobResult {
    let job_id = job.id;
    let job_kind = job.kind;
    
    // Catch panics to isolate worker failures from daemon.
    let result = std::panic::catch_unwind(|| {
        let content = job.correlation_id.as_deref().unwrap_or("");
        match job.kind {
            WorkerJobKind::ClassifyTurn => classify_turn(content),
            WorkerJobKind::ExtractAsccDelta => extract_ascc_delta(content),
            WorkerJobKind::DetectRepetition => detect_repetition(content),
            WorkerJobKind::ScanForErrors => scan_for_errors(content),
            WorkerJobKind::SuggestMemory => suggest_memory(content),
        }
    });
    
    match result {
        Ok(mut job_result) => {
            // Preserve the input job's ID so callers can correlate results.
            job_result.job_id = job_id;
            job_result
        }
        Err(panic_info) => {
            // Convert panic to failed result.
            let panic_msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "worker job panicked".to_string()
            };
            
            tracing::error!(job_id = %job_id, kind = ?job_kind, "Worker job panicked: {}", panic_msg);
            
            JobResult {
                job_id,
                job_type: format!("{:?}", job_kind),
                payload: serde_json::json!({"error": format!("panic: {}", panic_msg)}),
                success: false,
            }
        }
    }
}

/// Classify a turn based on content heuristics.
///
/// Categories: task, question, correction, meta, clarification, acknowledgement.
fn classify_turn(content: &str) -> JobResult {
    let lower = content.to_lowercase();
    let classification = if lower.contains("fix")
        || lower.contains("bug")
        || lower.contains("error")
        || lower.contains("wrong")
    {
        "correction"
    } else if lower.contains('?')
        || lower.starts_with("what")
        || lower.starts_with("how")
        || lower.starts_with("why")
        || lower.starts_with("when")
    {
        "question"
    } else if lower.starts_with("ok")
        || lower.starts_with("thanks")
        || lower.starts_with("got it")
        || lower.starts_with("yes")
    {
        "acknowledgement"
    } else if lower.contains("let's")
        || lower.contains("implement")
        || lower.contains("create")
        || lower.contains("build")
        || lower.contains("add")
    {
        "task"
    } else if lower.contains("mean") || lower.contains("clarif") || lower.contains("explain") {
        "clarification"
    } else if lower.contains("status") || lower.contains("progress") || lower.contains("where") {
        "meta"
    } else {
        "task" // Default.
    };

    let confidence = if content.len() < 10 { 0.4 } else { 0.7 };

    JobResult {
        job_id: Uuid::now_v7(),
        job_type: "classify_turn".into(),
        payload: json!({
            "classification": classification,
            "confidence": confidence,
        }),
        success: true,
    }
}

/// Extract ASCC delta from assistant response.
///
/// Heuristic extraction per G1-07-ascc §Delta Summarization Rule:
/// "MVP can start with rule-based extraction (regex for file paths/errors)"
///
/// Extracts all 10 ASCC slots where detectable:
///   intent, current_state, decisions, constraints, failures,
///   next_steps, open_questions, recent_results, notes, artifacts
fn extract_ascc_delta(content: &str) -> JobResult {
    let mut decisions = Vec::new();
    let mut next_steps = Vec::new();
    let mut constraints = Vec::new();
    let mut failures = Vec::new();
    let mut open_questions = Vec::new();
    let mut recent_results = Vec::new();
    let mut notes = Vec::new();

    // Extract a brief summary as current_state (first meaningful line, capped).
    let current_state: String = content
        .lines()
        .map(|l| l.trim())
        .find(|l| l.len() > 10 && !l.starts_with('#') && !l.starts_with("```"))
        .unwrap_or("")
        .chars()
        .take(200)
        .collect();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.len() < 5 {
            continue;
        }
        let lower = trimmed.to_lowercase();

        // Decisions: explicit markers + common LLM phrasing.
        if lower.starts_with("decided")
            || lower.starts_with("decision:")
            || lower.contains("choosing ")
            || lower.contains("we'll use")
            || lower.contains("i'll use")
            || lower.contains("going with")
            || lower.contains("selected ")
            || lower.contains("opted for")
            || lower.contains("approach:")
        {
            decisions.push(truncate_line(trimmed, 160));
        }

        // Next steps: task-oriented phrasing.
        if lower.starts_with("next")
            || lower.starts_with("todo")
            || lower.starts_with("- [ ]")
            || lower.starts_with("then ")
            || lower.starts_with("after that")
            || lower.contains("need to ")
            || lower.contains("should ")
            || lower.contains("will ")
            && (lower.contains("implement")
                || lower.contains("add")
                || lower.contains("fix")
                || lower.contains("update")
                || lower.contains("create"))
        {
            next_steps.push(truncate_line(trimmed, 160));
        }

        // Constraints: EXPLICIT marker required — not just any occurrence of constraint
        // language. Text like "the constraint is" or "must fix" in normal conversation
        // should NOT be extracted. Only lines that START with constraint markers.
        if lower.starts_with("constraint:")
            || lower.starts_with("constraint ")
            || lower.starts_with("requirement:")
            || lower.starts_with("required: ")
            || lower.starts_with("must not ")
            || lower.starts_with("cannot: ")
            || lower.starts_with("must: ")
            || lower.starts_with("not allowed: ")
            || lower.starts_with("forbidden: ")
        {
            constraints.push(truncate_line(trimmed, 160));
        }

        // Failures: EXPLICIT marker required — not just any occurrence of failure language.
        // Text like "panic in X" or "error in Y" in normal output should NOT be extracted.
        // Only lines that START with failure markers.
        if lower.starts_with("failure:")
            || lower.starts_with("failed:")
            || lower.starts_with("error:")
            || lower.starts_with("panic:")
            || lower.starts_with("broken:")
            || lower.starts_with("traceback:")
            || lower.starts_with("exception:")
            || lower.starts_with("err ")
            || lower.starts_with("failed ")
        {
            failures.push(truncate_line(trimmed, 160));
        }

        // Open questions: interrogative patterns.
        if (trimmed.ends_with('?')
            && (lower.starts_with("should")
                || lower.starts_with("do we")
                || lower.starts_with("how")
                || lower.starts_with("what")
                || lower.starts_with("why")
                || lower.starts_with("is there")
                || lower.starts_with("can we")))
            || lower.starts_with("open question:")
            || lower.starts_with("question:")
            || lower.starts_with("unclear:")
        {
            open_questions.push(truncate_line(trimmed, 160));
        }

        // Recent results: completion indicators.
        if lower.starts_with("done:")
            || lower.starts_with("completed:")
            || lower.starts_with("✅")
            || lower.starts_with("implemented")
            || lower.starts_with("fixed")
            || lower.starts_with("created ")
            || lower.contains("successfully")
            || lower.contains("passes")
            || lower.contains("working now")
            || lower.contains("compiled")
        {
            recent_results.push(truncate_line(trimmed, 160));
        }

        // Notes: metadata / informational markers.
        if lower.starts_with("note:")
            || lower.starts_with("nb:")
            || lower.starts_with("caveat:")
            || lower.starts_with("fyi:")
            || lower.starts_with("important:")
        {
            notes.push(truncate_line(trimmed, 160));
        }
    }

    let delta = json!({
        "current_state": current_state,
        "decisions": decisions,
        "next_steps": next_steps,
        "constraints": constraints,
        "failures": failures,
        "open_questions": open_questions,
        "recent_results": recent_results,
        "notes": notes,
    });

    JobResult {
        job_id: Uuid::now_v7(),
        job_type: "extract_ascc_delta".into(),
        payload: delta,
        success: true,
    }
}

/// Truncate a line to max chars, preserving word boundary where possible.
fn truncate_line(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let truncated = &s[..s.floor_char_boundary(max.saturating_sub(3))];
    format!("{}...", truncated)
}

/// Truncate content to max chars, safe for multi-byte characters (emoji, etc).
fn truncate_content(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    &s[..s.floor_char_boundary(max)]
}

/// Detect repetition in content.
fn detect_repetition(content: &str) -> JobResult {
    let lines: Vec<&str> = content.lines().collect();
    let unique: std::collections::HashSet<&str> = lines.iter().copied().collect();
    let repetition_ratio = if lines.is_empty() {
        0.0
    } else {
        1.0 - (unique.len() as f64 / lines.len() as f64)
    };

    JobResult {
        job_id: Uuid::now_v7(),
        job_type: "detect_repetition".into(),
        payload: json!({
            "total_lines": lines.len(),
            "unique_lines": unique.len(),
            "repetition_ratio": repetition_ratio,
            "is_repetitive": repetition_ratio > 0.3,
        }),
        success: true,
    }
}

/// Scan content for error patterns.
fn scan_for_errors(content: &str) -> JobResult {
    let lower = content.to_lowercase();
    let patterns = [
        "error:",
        "panic:",
        "fatal:",
        "exception:",
        "traceback",
        "stack trace",
    ];
    let found: Vec<&str> = patterns
        .iter()
        .filter(|p| lower.contains(**p))
        .copied()
        .collect();

    JobResult {
        job_id: Uuid::now_v7(),
        job_type: "scan_for_errors".into(),
        payload: json!({
            "error_patterns_found": found,
            "has_errors": !found.is_empty(),
        }),
        success: true,
    }
}

/// Suggest memory entries from content.
fn suggest_memory(content: &str) -> JobResult {
    let mut suggestions = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.len() > 20
            && (trimmed.contains("always")
                || trimmed.contains("never")
                || trimmed.contains("important")
                || trimmed.contains("remember"))
        {
            suggestions.push(trimmed.to_string());
        }
    }

    JobResult {
        job_id: Uuid::now_v7(),
        job_type: "suggest_memory".into(),
        payload: json!({
            "suggestions": suggestions,
            "count": suggestions.len(),
        }),
        success: true,
    }
}

/// Detect UFI signals from user message.
pub fn detect_ufi_signals(content: &str) -> Vec<UfiSignalType> {
    let lower = content.to_lowercase();
    let mut signals = Vec::new();

    // High tier.
    if lower.contains("undo") || lower.contains("revert") {
        signals.push(UfiSignalType::UndoOrRevert);
    }
    if lower.contains("no, ") || lower.starts_with("no ") || lower.contains("that's wrong") {
        signals.push(UfiSignalType::ExplicitRejection);
    }
    if lower.contains("override") || lower.contains("force") {
        signals.push(UfiSignalType::ManualOverride);
    }

    // Medium tier.
    if lower.contains("i meant")
        || lower.contains("what i mean")
        || lower.contains("let me rephrase")
    {
        signals.push(UfiSignalType::Rephrase);
    }
    if lower.contains("again") || lower.contains("already said") || lower.contains("repeat") {
        signals.push(UfiSignalType::RepeatRequest);
    }
    if lower.contains("scope") || lower.contains("only") || lower.contains("just") {
        signals.push(UfiSignalType::ScopeClarification);
    }

    // Low tier (language only — never dominates).
    if lower.contains("not") || lower.contains("don't") || lower.contains("won't") {
        signals.push(UfiSignalType::NegationLanguage);
    }

    signals
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JobPriority, WorkerJob};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_classify_question() {
        let result = classify_turn("What is the best approach?");
        assert!(result.success);
        assert_eq!(result.payload["classification"], "question");
    }

    #[test]
    fn test_classify_task() {
        let result = classify_turn("Implement the user authentication module");
        assert_eq!(result.payload["classification"], "task");
    }

    #[test]
    fn test_classify_correction() {
        let result = classify_turn("This is wrong, fix the bug in login");
        assert_eq!(result.payload["classification"], "correction");
    }

    #[test]
    fn test_extract_delta() {
        let content =
            "Decided to use JWT tokens.\nNext: add refresh logic.\nConstraint: must support PKCE.";
        let result = extract_ascc_delta(content);
        assert!(result.success);
        assert!(!result.payload["decisions"].as_array().unwrap().is_empty());
        assert!(!result.payload["next_steps"].as_array().unwrap().is_empty());
        assert!(!result.payload["constraints"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_ufi_detection() {
        let signals = detect_ufi_signals("No, undo that, I already said this");
        assert!(signals.contains(&UfiSignalType::ExplicitRejection));
        assert!(signals.contains(&UfiSignalType::UndoOrRevert));
    }
}

/// Execute a worker job with LLM extraction (async).
/// 
/// Tries MiniMax M2.7 first, falls back to regex heuristic on failure.
/// Per UNIFIED_ORGANISM_SPEC §11.3.
pub async fn execute_job_llm(job: &WorkerJob) -> JobResult {
    let content = job.correlation_id.as_deref().unwrap_or("");
    if content.is_empty() {
        return execute_job(job);
    }

    // Build LLM prompt based on job kind
    let prompt = match job.kind {
        WorkerJobKind::ClassifyTurn => format!(
            "Classify this user input as one of: task, question, correction, meta, clarification, acknowledgement.\nReturn JSON: {{\"classification\": \"...\", \"confidence\": 0.0-1.0}}\n\nINPUT:\n{}", 
            truncate_content(&content, 2000)
        ),
        WorkerJobKind::ExtractAsccDelta => format!(
            "Extract structured information from this text.\nReturn JSON with arrays for: decisions, constraints, failures, next_steps, open_questions, recent_results, notes, why_reasons.\nFor decisions, always include WHY (look for: because, the reason, I chose X over Y, this is better because).\nOnly include items actually present.\n\nTEXT:\n{}",
            truncate_content(&content, 4000)
        ),
        WorkerJobKind::DetectRepetition => format!(
            "Is this content semantically repetitive (saying the same thing multiple ways)?\nReturn JSON: {{\"is_repetitive\": true/false, \"ratio\": 0.0-1.0, \"evidence\": \"...\"}}\n\nCONTENT:\n{}",
            truncate_content(&content, 2000)
        ),
        WorkerJobKind::ScanForErrors => format!(
            "Identify errors, stack traces, and failure patterns in this text.\nReturn JSON: {{\"errors\": [{{\"type\": \"...\", \"severity\": \"...\", \"context\": \"...\"}}]}}\n\nTEXT:\n{}",
            truncate_content(&content, 2000)
        ),
        WorkerJobKind::SuggestMemory => format!(
            "Extract stable facts, preferences, and behavioral patterns worth remembering from this text.\nReturn JSON: {{\"suggestions\": [\"fact1\", \"fact2\"], \"count\": N}}\n\nTEXT:\n{}",
            truncate_content(&content, 2000)
        ),
    };

    // Try LLM call (MiniMax M2.7 direct API, 8s timeout)
    let client = reqwest::Client::new();
    let api_key = std::env::var("MINIMAX_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return execute_job(job);
    }
    
    let llm_result = tokio::time::timeout(
        std::time::Duration::from_secs(8),
        client.post("https://api.minimax.io/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "MiniMax-M2.7",
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 500,
            }))
            .send(),
    ).await;

    match llm_result {
        Ok(Ok(resp)) => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                if let Some(text) = data.pointer("/choices/0/message/content").and_then(|v| v.as_str()) {
                    // Try to parse the JSON from LLM response
                    let start = text.find('{').unwrap_or(0);
                    let end = text.rfind('}').map(|i| i + 1).unwrap_or(text.len());
                    if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&text[start..end]) {
                        return JobResult {
                            job_id: job.id,
                            job_type: format!("{:?}", job.kind),
                            payload,
                            success: true,
                        };
                    }
                }
            }
            // LLM returned but couldn't parse — fall back to regex
            tracing::debug!(kind = ?job.kind, "LLM worker: response unparseable, falling back to regex");
            execute_job(job)
        }
        _ => {
            // LLM timeout or error — fall back to regex
            tracing::debug!(kind = ?job.kind, "LLM worker: timeout/error, falling back to regex");
            execute_job(job)
        }
    }
}
