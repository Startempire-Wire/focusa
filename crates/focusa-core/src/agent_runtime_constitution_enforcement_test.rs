use crate::agent_runtime_constitution::*;
use crate::agent_runtime_constitution_enforcement::*;
use std::collections::BTreeSet;

fn claim(id: &str, class: &str) -> InstructionClaim {
    InstructionClaim {
        claim_id: id.into(),
        source_id: "source".into(),
        claim_class: class.into(),
        normalized_text: "require proof".into(),
        source_text_sha256: "a".repeat(64),
        applicability: InstructionApplicability::Applicable,
        scope_ref: "/project".into(),
        condition: None,
        subject: None,
        action: None,
        object: None,
        modality: None,
        exceptions: vec![],
        rationale: None,
        verification_ref: None,
        enforcement_ref: None,
        authority_ref: None,
        trust_ref: None,
        provenance_refs: vec![],
    }
}

#[test]
fn skills_use_explicit_applicability_and_progressive_disclosure() {
    let candidates = vec![
        SkillBinding {
            skill_id: "release-proof".into(),
            source_ref: "skills/release/SKILL.md".into(),
            activation_condition: "release work".into(),
            authority_scope: "project".into(),
        },
        SkillBinding {
            skill_id: "browser".into(),
            source_ref: "skills/browser/SKILL.md".into(),
            activation_condition: "browser work".into(),
            authority_scope: "origin".into(),
        },
    ];
    let applicable = ["release-proof".into()].into_iter().collect();
    let plan = compile_skill_activation_plan("skills-1", candidates, &applicable);
    assert_eq!(plan.bindings.len(), 1);
    assert_eq!(plan.bindings[0].skill_id, "release-proof");
    assert!(plan.excluded.contains_key("browser"));
}

#[test]
fn typed_tool_route_wins_and_mutation_requires_confirmation() {
    let capabilities = vec![
        ToolCapability {
            tool_name: "focusa_release".into(),
            operation_classes: ["release".into()].into_iter().collect(),
            typed: true,
            mutation: true,
            authority_scope: "project".into(),
        },
        ToolCapability {
            tool_name: "shell".into(),
            operation_classes: ["release".into()].into_iter().collect(),
            typed: false,
            mutation: true,
            authority_scope: "host".into(),
        },
    ];
    let routing = compile_tool_routing_plan(
        "routing-1",
        &["release".into()].into_iter().collect(),
        &capabilities,
    )
    .unwrap();
    assert!(routing.allowed_tools.contains("focusa_release"));
    assert!(routing.denied_tools.contains_key("shell"));
    let enforcement = compile_enforcement_plan(
        "enforce-1",
        &[claim("release", "release_authority")],
        vec![],
    );
    assert!(matches!(
        enforce_tool_operation(
            &routing,
            &enforcement,
            "focusa_release",
            "release_publication",
            false
        ),
        EnforcementDecision::RequireConfirmation { .. }
    ));
    assert_eq!(
        enforce_tool_operation(
            &routing,
            &enforcement,
            "focusa_release",
            "release_publication",
            true
        ),
        EnforcementDecision::Allow
    );
    assert!(matches!(
        enforce_tool_operation(&routing, &enforcement, "shell", "release_publication", true),
        EnforcementDecision::Deny { .. }
    ));
}

#[test]
fn missing_operation_route_fails_closed() {
    let result =
        compile_tool_routing_plan("routing-1", &["unknown".into()].into_iter().collect(), &[]);
    assert_eq!(result.unwrap_err(), vec!["missing_tool_route:unknown"]);
}

#[test]
fn validation_matrix_is_change_class_specific() {
    let matrix = compile_validation_matrix(
        "validation-1",
        &[
            "crates/core/src/auth.rs".into(),
            "apps/pi/src/index.ts".into(),
            "docs/current/status.md".into(),
            "crates/core/src/persistence.rs".into(),
        ],
    );
    let ids: BTreeSet<_> = matrix
        .rules
        .iter()
        .map(|rule| rule.rule_id.as_str())
        .collect();
    for required in [
        "format",
        "unit",
        "rust-check",
        "typecheck",
        "lint",
        "docs-drift",
        "security-negative",
        "restart-recovery",
    ] {
        assert!(ids.contains(required), "missing {required}");
    }
}
