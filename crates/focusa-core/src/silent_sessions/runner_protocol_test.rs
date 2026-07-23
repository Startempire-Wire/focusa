use crate::silent_sessions::*;

#[test]
fn runner_execution_mode_never_falls_back_cross_user() {
    assert_eq!(
        select_runner_execution_mode("wirebot", "wirebot", None).unwrap(),
        RunnerExecutionMode::EmbeddedSameUser
    );
    assert!(select_runner_execution_mode("focusa", "wirebot", None).is_err());
    assert_eq!(
        select_runner_execution_mode("focusa", "wirebot", Some("uid:1000")).unwrap(),
        RunnerExecutionMode::PerUserSocket {
            socket_scope: "uid:1000".into(),
        }
    );
}

#[test]
fn runner_operation_action_digest_is_typed_and_stable() {
    let operation = RunnerOperation::Signal {
        signal: RunnerSignal::Interrupt,
    };
    assert_eq!(operation.required_action(), SilentSessionAction::Interrupt);
    assert_eq!(operation.action_digest().unwrap().len(), 64);
    assert_eq!(
        operation.action_digest().unwrap(),
        operation.action_digest().unwrap()
    );
}
