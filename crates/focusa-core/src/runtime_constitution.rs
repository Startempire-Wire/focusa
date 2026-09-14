//! Runtime Constitution — slice 1 (#256).
//!
//! The behavioral law lives ONCE here, hash-bound and versioned. Every
//! harness fetches it through the daemon (GET /v1/runtime-constitution) or
//! falls back to this embedded copy — never a per-harness hard-coded
//! prompt array. Pi prompt files and adapter guidance are projections,
//! not canonical authority (#256 executive decision).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const RUNTIME_CONSTITUTION_SCHEMA: &str = "focusa.runtime_constitution.v1";
pub const RUNTIME_CONSTITUTION_VERSION: &str = "1.0.0";

/// The canonical behavioral law (single source for all harnesses).
pub const BEHAVIORAL_LAW: &str = "\
## Focusa Cognitive Guidance
You are operating within Focusa, a cognitive runtime that preserves focus and decisions.

RULES:
- Use the focusa_decide tool when you make a significant decision
- Use the focusa_constraint tool ONLY for hard constraints (e.g. 'NEVER delete production data', 'must preserve X')
- Use the focusa_failure tool when something fails
- Do NOT record internal monologue, reasoning, or self-referential notes as constraints
  (e.g. 'cannot advance without operator direction' is NOT a constraint — it's context)
- Check the dynamic Focusa Focus Slice before acting and do not violate its constraints
- Do not contradict decisions in the dynamic Focusa Focus Slice without explanation
- If context was compacted, a scoped canonical Workpoint packet outranks transcript tail
- Project-aware writes fail closed unless the dynamic Focusa Focus Slice verifies project_root + continuity_id authority
- If project identity is ambiguous, infer from bounded repository evidence and ask the operator only when multiple plausible roots remain
- Treat cwd as the coding agent launch location only; it is not consent to bind Focusa or proof of project identity
- Older Focusa projects may lack current markers; consult git, Beads, prior sessions, aliases, and persisted Workpoints before suggesting a new project
- Stay aware of the operator's preferred address, local time, goals, constraints, desired pace, and confirmed timeline; adapt detail and interruption level to canonical operator state
- Never invent urgency or deadlines. Use Focusa temporal authority for consequential time claims and state uncertainty as a range
- For meaningful tasks, record a wall-clock start, make a bounded delivery prediction, observe actual elapsed time at completion, evaluate the prediction, and preserve reusable timing lessons
- Use Focusa tools to accomplish the operator's desired outcome within operator constraints; do not make Focusa mechanics the center of the conversation";

/// The constitution artifact: versioned, hash-bound, ready to serve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeConstitution {
    pub schema: String,
    pub version: String,
    pub constitution_ref: String,
    pub content_digest: String,
    pub behavioral_law: String,
}

pub fn canonical_constitution() -> RuntimeConstitution {
    let mut hasher = Sha256::new();
    hasher.update(RUNTIME_CONSTITUTION_VERSION.as_bytes());
    hasher.update(BEHAVIORAL_LAW.as_bytes());
    let digest = format!("sha256:{}", hex(&hasher.finalize()));
    RuntimeConstitution {
        schema: RUNTIME_CONSTITUTION_SCHEMA.to_string(),
        version: RUNTIME_CONSTITUTION_VERSION.to_string(),
        constitution_ref: format!("runtime-constitution:v{}", RUNTIME_CONSTITUTION_VERSION),
        content_digest: digest,
        behavioral_law: BEHAVIORAL_LAW.to_string(),
    }
}

pub fn verify_constitution(constitution: &RuntimeConstitution) -> Result<(), String> {
    if constitution.schema != RUNTIME_CONSTITUTION_SCHEMA {
        return Err(format!("unexpected schema {}", constitution.schema));
    }
    let canonical = canonical_constitution();
    if constitution.content_digest != canonical.content_digest {
        return Err(format!(
            "digest mismatch: served {} canonical {}",
            constitution.content_digest, canonical.content_digest
        ));
    }
    if constitution.behavioral_law != canonical.behavioral_law {
        return Err("behavioral law text diverged from canonical".to_string());
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_constitution_is_stable() {
        let a = canonical_constitution();
        let b = canonical_constitution();
        assert_eq!(a.content_digest, b.content_digest);
        assert!(a.content_digest.starts_with("sha256:"));
        assert_eq!(a.content_digest.len(), 71);
        assert_eq!(a.version, RUNTIME_CONSTITUTION_VERSION);
        assert!(a.behavioral_law.contains("focusa_decide"));
    }

    #[test]
    fn verify_accepts_canonical_and_rejects_tampering() {
        let mut constitution = canonical_constitution();
        assert_eq!(verify_constitution(&constitution), Ok(()));
        constitution.behavioral_law.push_str("\\n- INJECTED RULE");
        assert!(verify_constitution(&constitution).is_err());
    }

    #[test]
    fn law_roundtrips_byte_exact() {
        let constitution = canonical_constitution();
        let json = serde_json::to_string(&constitution).unwrap();
        let parsed: RuntimeConstitution = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.behavioral_law, BEHAVIORAL_LAW);
    }
}
