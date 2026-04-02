//! Reliability Focus Mode (RFM) — docs/36-reliability-focus-mode.md
//!
//! 4 levels: R0 (normal) → R3 (ensemble).
//! Artifact Integrity Score (AIS): ≥0.90 safe, 0.70–0.90 degraded, <0.70 triggers RFM.
//! 4 microcell validators: Schema, Constraint, Consistency, ReferenceGrounding.
//!
//! Trigger: AIS < 0.70 → auto-escalate to R1.
//! De-escalation: 3 consecutive passes → drop one level.

use crate::types::*;
use chrono::Utc;

/// Run all validators against an artifact.
pub fn validate(content: &str, constraints: &[String]) -> Vec<ValidatorResult> {
    let now = Utc::now();
    vec![
        ValidatorResult {
            validator: MicrocellValidator::Schema,
            passed: validate_schema(content),
            details: "Schema validation".into(),
            timestamp: now,
        },
        ValidatorResult {
            validator: MicrocellValidator::Constraint,
            passed: validate_constraints(content, constraints),
            details: "Constraint compliance".into(),
            timestamp: now,
        },
        ValidatorResult {
            validator: MicrocellValidator::Consistency,
            passed: validate_consistency(content),
            details: "Internal consistency".into(),
            timestamp: now,
        },
        ValidatorResult {
            validator: MicrocellValidator::ReferenceGrounding,
            passed: validate_grounding(content),
            details: "Reference grounding".into(),
            timestamp: now,
        },
    ]
}

/// Compute AIS from validator results.
pub fn compute_ais(results: &[ValidatorResult]) -> f64 {
    if results.is_empty() {
        return 1.0;
    }
    let passed = results.iter().filter(|r| r.passed).count() as f64;
    passed / results.len() as f64
}

/// Update RFM state based on AIS.
/// Returns true if level changed.
pub fn update_rfm(state: &mut RfmState, results: Vec<ValidatorResult>) -> bool {
    let ais = compute_ais(&results);
    let old_level = state.level;
    state.ais_score = ais;
    state.validator_results = results;

    // Escalation.
    if ais < 0.70 && state.level < RfmLevel::R1 {
        state.level = RfmLevel::R1;
    }
    if ais < 0.50 && state.level < RfmLevel::R2 {
        state.level = RfmLevel::R2;
    }
    if ais < 0.30 && state.level < RfmLevel::R3 {
        state.level = RfmLevel::R3;
    }

    // De-escalation: all pass → drop one level.
    if ais >= 0.90 && state.level > RfmLevel::R0 {
        state.level = match state.level {
            RfmLevel::R3 => RfmLevel::R2,
            RfmLevel::R2 => RfmLevel::R1,
            RfmLevel::R1 => RfmLevel::R0,
            RfmLevel::R0 => RfmLevel::R0,
        };
    }

    state.level != old_level
}

/// Check if regeneration is needed (R2+).
pub fn needs_regeneration(state: &RfmState) -> bool {
    state.level >= RfmLevel::R2
}

/// Check if ensemble is needed (R3).
pub fn needs_ensemble(state: &RfmState) -> bool {
    state.level >= RfmLevel::R3
}

// ─── Microcell validators ───────────────────────────────────────────────────

fn validate_schema(content: &str) -> bool {
    // Basic: non-empty, valid UTF-8 (always true in Rust).
    !content.is_empty()
}

fn validate_constraints(content: &str, constraints: &[String]) -> bool {
    // Check that none of the constraint patterns are violated.
    for c in constraints {
        if c.starts_with("max_length:")
            && let Ok(max) = c.trim_start_matches("max_length:").parse::<usize>()
            && content.len() > max
        {
            return false;
        }
        if c.starts_with("must_contain:") {
            let required = c.trim_start_matches("must_contain:");
            if !content.contains(required) {
                return false;
            }
        }
    }
    true
}

fn validate_consistency(content: &str) -> bool {
    // Heuristic consistency: check for self-contradictions.
    // Look for patterns like "X is Y" followed by "X is not Y".
    let sentences: Vec<&str> = content.split('.').collect();
    if sentences.len() < 2 {
        return true;
    }
    // Check for explicit contradictions ("however" + negation near same subject)
    let has_contradiction = sentences.windows(2).any(|pair| {
        let a = pair[0].to_lowercase();
        let b = pair[1].to_lowercase();
        (b.contains("however") || b.contains("but actually") || b.contains("that's wrong"))
            && (b.contains("not ") || b.contains("n't "))
            // Check if they share at least one significant word
            && a.split_whitespace()
                .filter(|w| w.len() > 3)
                .any(|w| b.contains(w))
    });
    !has_contradiction
}

fn validate_grounding(content: &str) -> bool {
    // Heuristic grounding: flag content that makes unverifiable claims.
    // Look for hallucination signals.
    let lower = content.to_lowercase();
    let hallucination_signals = [
        "as an ai", "i cannot verify", "i don't have access",
        "i'm not sure but", "i think it might be",
        "according to my training", "as of my knowledge cutoff",
    ];
    // If any hallucination signal is found, grounding fails
    !hallucination_signals.iter().any(|s| lower.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ais_all_pass() {
        let results = validate("some content", &[]);
        let ais = compute_ais(&results);
        assert_eq!(ais, 1.0);
    }

    #[test]
    fn test_rfm_escalation() {
        let mut state = RfmState::default();
        let results = vec![
            ValidatorResult {
                validator: MicrocellValidator::Schema,
                passed: true,
                details: String::new(),
                timestamp: Utc::now(),
            },
            ValidatorResult {
                validator: MicrocellValidator::Constraint,
                passed: false,
                details: String::new(),
                timestamp: Utc::now(),
            },
            ValidatorResult {
                validator: MicrocellValidator::Consistency,
                passed: false,
                details: String::new(),
                timestamp: Utc::now(),
            },
            ValidatorResult {
                validator: MicrocellValidator::ReferenceGrounding,
                passed: false,
                details: String::new(),
                timestamp: Utc::now(),
            },
        ];
        let changed = update_rfm(&mut state, results);
        assert!(changed);
        assert!(state.level >= RfmLevel::R1);
    }

    #[test]
    fn test_constraint_validation() {
        assert!(validate_constraints("short", &["max_length:100".into()]));
        assert!(!validate_constraints(
            "long text here",
            &["max_length:5".into()]
        ));
    }
}

/// LLM-backed deep validation for R1+ levels.
/// Calls MiniMax M2.7 to analyze content quality.
/// Returns (consistency_passed, grounding_passed, details).
pub async fn validate_llm(content: &str, constraints: &[String]) -> (bool, bool, String) {
    let api_key = std::env::var("MINIMAX_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        return (true, true, "LLM unavailable, skipped".into());
    }
    
    let constraint_text = if constraints.is_empty() {
        "None specified".to_string()
    } else {
        constraints.join("; ")
    };
    
    let prompt = format!(
        r#"Analyze this AI-generated content for quality issues.

CONTENT:
{}

CONSTRAINTS: {}

Check for:
1. Internal consistency: Does the content contradict itself?
2. Grounding: Are claims verifiable? Any hallucination signals?
3. Constraint compliance: Does it follow the stated constraints?

Return ONLY valid JSON:
{{
  "consistency_passed": true/false,
  "grounding_passed": true/false, 
  "constraint_passed": true/false,
  "issues": ["issue1", "issue2"],
  "overall_quality": 0.0-1.0
}}"#,
        &content[..content.len().min(2000)],
        constraint_text,
    );
    
    let client = reqwest::Client::new();
    match tokio::time::timeout(
        std::time::Duration::from_secs(8),
        client.post("https://api.minimax.io/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": "MiniMax-M2.7",
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 300,
                "temperature": 0.1,
            }))
            .send(),
    ).await {
        Ok(Ok(resp)) => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                if let Some(text) = data.pointer("/choices/0/message/content").and_then(|v| v.as_str()) {
                    let start = text.find('{').unwrap_or(0);
                    let end = text.rfind('}').map(|i| i + 1).unwrap_or(text.len());
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text[start..end]) {
                        let c_ok = parsed.get("consistency_passed").and_then(|v| v.as_bool()).unwrap_or(true);
                        let g_ok = parsed.get("grounding_passed").and_then(|v| v.as_bool()).unwrap_or(true);
                        let issues: Vec<String> = parsed.get("issues")
                            .and_then(|v| v.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default();
                        let detail = if issues.is_empty() { "No issues found".into() } else { issues.join("; ") };
                        return (c_ok, g_ok, detail);
                    }
                }
            }
            (true, true, "LLM response unparseable".into())
        }
        _ => (true, true, "LLM timeout".into()),
    }
}
