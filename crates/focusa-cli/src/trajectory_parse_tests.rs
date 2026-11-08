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
    // Generic confirmation is resolved at the command boundary so valid
    // command-owned repairs do not require an installer lifecycle (#702).
    // Defining a goal is not such a repair and must still reject this flag.
    let mut parsed = Cli::try_parse_from(args).expect("parse before command-owned validation");
    let error = super::prepare_command_confirmation(&mut parsed)
        .expect_err("global confirmation requires lifecycle action");
    assert!(error.to_string().contains("--lifecycle-action"));
}
