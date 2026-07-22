use super::*;

fn target() -> DeliveryTarget {
    DeliveryTarget {
        run_id: SilentSessionRunId::new(),
        generation: RunGeneration::first(),
        approval_id: ApprovalId::new(),
        idempotency_key: "delivery-1".into(),
    }
}

#[test]
fn request_digest_binds_kind_content_and_approval() {
    let target = target();
    let first = request_hash(&target, DeliveryKind::Input, &json!({"text": "first"}));
    let second = request_hash(&target, DeliveryKind::Input, &json!({"text": "second"}));
    let steer = request_hash(&target, DeliveryKind::Steer, &json!({"text": "first"}));
    let mut other_approval = target.clone();
    other_approval.approval_id = ApprovalId::new();
    let approved = request_hash(
        &other_approval,
        DeliveryKind::Input,
        &json!({"text": "first"}),
    );
    assert_ne!(first, second);
    assert_ne!(first, steer);
    assert_ne!(first, approved);
}

#[test]
fn steering_requires_workpoint_governance_before_runner_delivery() {
    let effects = DeliveryKind::Steer.side_effects("digest");
    assert_eq!(effects.len(), 2);
    assert_eq!(effects[0], "workpoint_steering_request:digest");
    assert_eq!(effects[1], "runner_steer_request:digest");
}

#[test]
fn only_follow_up_can_queue_while_paused() {
    assert!(DeliveryKind::FollowUp.accepts(SilentSessionLifecycle::Paused));
    assert!(!DeliveryKind::Input.accepts(SilentSessionLifecycle::Paused));
    assert!(!DeliveryKind::Keys.accepts(SilentSessionLifecycle::Paused));
}

#[test]
fn text_validation_is_bounded() {
    assert!(validate_text("work", "text").is_ok());
    assert!(validate_text(" ", "text").is_err());
    assert!(validate_text(&"x".repeat(MAX_TEXT_BYTES + 1), "text").is_err());
}
