//! Static cross-surface proof: every mandatory Spec 144 family is grounded in
//! the owned API operation registry and reachable through the owned CLI.

#[test]
fn semantic_integrity_families_are_grounded_in_api_and_cli() {
    let api = include_str!("../../focusa-api/src/routes/semantic_integrity.rs");
    let cli = include_str!("../src/commands/semantic_integrity.rs");

    for family in [
        "status",
        "artifact",
        "registry",
        "validation",
        "build",
        "verify",
        "settlement",
        "replay",
        "migration",
        "rollback",
        "vertical",
        "reflex",
    ] {
        assert!(
            api.contains(&format!("\"{family}\"")),
            "API registry missing {family}"
        );
    }
    for operation in [
        "semantic.integrity.status",
        "semantic.integrity.registry",
        "semantic.integrity.validate",
        "semantic_pair.builder.start",
        "semantic_pair.verify.start",
        "semantic_pair.settlement.commit",
        "semantic_pair.replay",
        "semantic_pair.migration.run",
        "semantic_pair.rollback.commit",
        "vertical.bundle.activate",
        "semantic.reflex.visibility",
    ] {
        assert!(
            api.contains(operation),
            "missing stable operation ID {operation}"
        );
    }
    for command in ["Status", "Registry", "Artifacts", "Inspect", "Invoke"] {
        assert!(cli.contains(command), "CLI missing {command}");
    }
    assert!(api.contains("idempotency_required: true"));
    assert!(api.contains("confirmation_required: true"));
    assert!(api.contains("SchemaOnly"));
    assert!(cli.contains("json_output"));
}
