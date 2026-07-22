use focusa_core::silent_sessions::ConfigRevisionId;

use super::*;

fn target() -> MutationTarget {
    MutationTarget {
        run_id: SilentSessionRunId::new(),
        generation: RunGeneration::first(),
        approval_id: ApprovalId::new(),
        idempotency_key: "config-1".into(),
    }
}

#[test]
fn rollback_digest_binds_target_revision_and_approval() {
    let first = RollbackBody {
        target: target(),
        target_revision_id: ConfigRevisionId::new(),
    };
    let mut other_revision = first.clone();
    other_revision.target_revision_id = ConfigRevisionId::new();
    let mut other_approval = first.clone();
    other_approval.target.approval_id = ApprovalId::new();
    assert_ne!(
        hash_request("rollback", &first),
        hash_request("rollback", &other_revision)
    );
    assert_ne!(
        hash_request("rollback", &first),
        hash_request("rollback", &other_approval)
    );
}
