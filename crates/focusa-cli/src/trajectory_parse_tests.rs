use super::{Cli, Commands};
use clap::Parser;

fn goal_args() -> Vec<&'static str> {
    vec![
        "focusa",
        "trajectory",
        "define-goal",
        "--long-term-goal",
        "Preserve scoped trajectory authority",
        "--desired-end-state",
        "Committed goals are readable",
        "--project-root",
        "/tmp/issue621-parser-only",
        "--continuity-id",
        "issue621-parser",
        "--operator-confirmed",
    ]
}

#[test]
fn issue621_complete_goal_confirmation_does_not_require_lifecycle_action() {
    // Parser only: never invokes a provider, daemon, or mutation handler.
    let parsed = Cli::try_parse_from(goal_args()).unwrap();
    assert!(!parsed.confirm);
    assert!(parsed.lifecycle_action.is_none());
    assert!(matches!(
        parsed.command,
        Commands::Trajectory(crate::commands::trajectory::TrajectoryCmd::DefineGoal {
            operator_confirmed: true,
            ..
        })
    ));
}

#[test]
fn issue621_global_confirmation_still_requires_lifecycle_action() {
    let mut args = goal_args();
    args.push("--confirm");
    let error = Cli::try_parse_from(args)
        .err()
        .expect("global confirmation requires lifecycle action");
    assert_eq!(
        error.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
    assert!(error.to_string().contains("--lifecycle-action"));
}
