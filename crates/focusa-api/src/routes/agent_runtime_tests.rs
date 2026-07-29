#[test]
fn spec140_api_surface_is_complete_and_namespaced() {
    let runtime = include_str!("agent_runtime.rs");
    let delivery = include_str!("agent_runtime_delivery.rs");
    let combined = format!("{runtime}\n{delivery}");
    for route in [
        "/v1/agent-runtime/instructions/scan",
        "/v1/agent-runtime/instructions/sources",
        "/v1/agent-runtime/instructions/claims",
        "/v1/agent-runtime/instructions/conflicts",
        "/v1/agent-runtime/instructions/reconcile",
        "/v1/agent-runtime/instructions/simulate",
        "/v1/agent-runtime/instructions/effective",
        "/v1/agent-runtime/instructions/drift",
        "/v1/agent-runtime/constitutions/draft",
        "/v1/agent-runtime/constitutions/{id}",
        "/v1/agent-runtime/constitutions/{id}/preview",
        "/v1/agent-runtime/constitutions/{id}/approve",
        "/v1/agent-runtime/constitutions/{id}/activate",
        "/v1/agent-runtime/constitutions/{id}/revoke",
        "/v1/agent-runtime/constitutions/{id}/rollback",
        "/v1/agent-runtime/compile/system-prompt",
        "/v1/agent-runtime/compile/agents-md",
        "/v1/agent-runtime/compile/skills",
        "/v1/agent-runtime/compile/target",
        "/v1/agent-runtime/variants/{id}",
        "/v1/agent-runtime/evaluations",
        "/v1/agent-runtime/evaluations/{id}",
        "/v1/agent-runtime/delivery/preview",
        "/v1/agent-runtime/delivery/commit",
        "/v1/agent-runtime/delivery/verify",
        "/v1/agent-runtime/delivery/status",
    ] {
        assert!(combined.contains(route), "missing Spec 140 route {route}");
    }
}

#[test]
fn mutations_are_permission_confirmation_receipt_and_idempotency_gated() {
    let runtime = include_str!("agent_runtime.rs");
    let delivery = include_str!("agent_runtime_delivery.rs");
    assert!(runtime.contains("work-loop:write"));
    assert!(runtime.contains("idempotency_key_required"));
    assert!(delivery.contains("operator_confirmation_and_evidence_required"));
    assert!(delivery.contains("operator_confirmation_and_receipt_required"));
    assert!(delivery.contains("unverified_artifact_delivery_forbidden"));
    assert!(delivery.contains("activated_by_runtime_agent\":false"));
}

#[test]
fn runtime_studio_exposes_every_spec140_workbench() {
    let studio = include_str!("agent_runtime_studio.rs");
    assert!(studio.contains("/v1/agent-runtime/studio"));
    for panel in [
        "role-grounding",
        "source-inventory",
        "conflict-workbench",
        "prompt-composition",
        "prompt-modes",
        "environment-variants",
        "skills-tools",
        "execution-boundaries",
        "targets",
        "delivery",
        "activation",
        "rollback",
    ] {
        assert!(studio.contains(panel), "Runtime Studio missing {panel}");
    }
    assert!(studio.contains("a2ui_messages"));
    assert!(studio.contains("self_activation_forbidden"));
}

#[test]
fn preview_and_simulation_are_non_mutating() {
    let runtime = include_str!("agent_runtime.rs");
    let delivery = include_str!("agent_runtime_delivery.rs");
    assert!(runtime.matches("\"committed\":false").count() >= 2);
    assert!(delivery.matches("\"committed\":false").count() >= 3);
    assert!(runtime.contains("regenerated\":false"));
}
