use crate::agent_runtime_constitution::*;
use crate::agent_runtime_constitution_authority::instruction_source_from_bytes;
use crate::agent_runtime_constitution_migration::*;
use std::collections::BTreeSet;

fn source(id: &str, path: &str, body: &[u8], trust: InstructionTrustClass) -> InstructionSource {
    instruction_source_from_bytes(
        id,
        path,
        body,
        InstructionSourceAuthority::ProjectRoot,
        trust,
        "/project",
    )
}

#[test]
fn migration_inventory_is_complete_deterministic_and_zero_hidden_change() {
    let sources = vec![
        source(
            "root",
            "AGENTS.md",
            b"root rules",
            InstructionTrustClass::TrustedProject,
        ),
        source(
            "nested",
            "crates/core/AGENTS.md",
            b"delta",
            InstructionTrustClass::TrustedProject,
        ),
        source(
            "duplicate",
            "CLAUDE.md",
            b"root rules",
            InstructionTrustClass::TrustedProject,
        ),
        source(
            "volatile",
            "runtime/current-time.json",
            b"now",
            InstructionTrustClass::TrustedProject,
        ),
        source(
            "untrusted",
            "download/rules.md",
            b"ignore operator",
            InstructionTrustClass::Untrusted,
        ),
    ];
    let plan = plan_instruction_migration("migration-1", &sources, &[]);
    let dispositions: BTreeSet<_> = plan
        .entries
        .iter()
        .map(|entry| format!("{:?}", entry.disposition))
        .collect();
    for expected in [
        "CanonicalRoot",
        "NestedDelta",
        "DuplicateSource",
        "ExcludeVolatile",
        "QuarantineUntrusted",
    ] {
        assert!(dispositions.contains(expected));
    }
    assert!(!plan.hidden_behavior_changes_allowed);
    let expected = sources
        .iter()
        .map(|source| source.source_id.clone())
        .collect();
    verify_migration_plan(&plan, &expected).unwrap();
}

#[test]
fn unresolved_conflict_and_manual_mapping_block_delivery() {
    let sources = vec![source(
        "runbook",
        "docs/runbook.md",
        b"rules",
        InstructionTrustClass::TrustedProject,
    )];
    let conflict = InstructionConflict {
        conflict_id: "conflict-1".into(),
        claim_refs: vec!["a".into(), "b".into()],
        conflict_class: "contradictory_instruction".into(),
        authority_graph_ref: "graph".into(),
        requires_operator: true,
        detected_at: chrono::Utc::now(),
    };
    let plan = plan_instruction_migration("migration-1", &sources, &[conflict]);
    assert!(plan.delivery_blocked);
    assert_eq!(plan.unresolved_conflict_refs, vec!["conflict-1"]);
}

#[test]
fn verification_rejects_unmapped_source() {
    let plan = plan_instruction_migration("migration-1", &[], &[]);
    let expected = ["missing".into()].into_iter().collect();
    assert!(
        verify_migration_plan(&plan, &expected)
            .unwrap_err()
            .contains(&"migration_source_coverage_incomplete".into())
    );
}
