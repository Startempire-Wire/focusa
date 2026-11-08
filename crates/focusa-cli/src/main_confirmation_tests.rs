use super::*;

fn repair_args() -> Vec<String> {
    let root = std::env::temp_dir().join("focusa-stage-confirmation-parser");
    vec![
        "focusa".into(),
        "workpoint".into(),
        "repair-stage".into(),
        "--project-root".into(),
        root.display().to_string(),
        "--continuity-id".into(),
        "parser-continuity".into(),
        "--stage".into(),
        "verify_outcome".into(),
        "--reason".into(),
        "stage-only recovery".into(),
        "--evidence-ref".into(),
        "test:approved-stage".into(),
        "--idempotency-key".into(),
        "parser-stage-recovery".into(),
    ]
}

fn parse_cli(args: Vec<String>) -> (Cli, clap::Command, clap::ArgMatches) {
    // Exercise the same selected schema and stack allowance as production.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let mut command = Cli::command();
            let matches = command.try_get_matches_from_mut(args)?;
            let cli = Cli::from_arg_matches(&matches)?;
            Ok::<_, clap::Error>((cli, command, matches))
        })
        .unwrap()
        .join()
        .unwrap()
        .expect("confirmation arguments must parse")
}

fn stage_confirmed(cli: &Cli) -> bool {
    match &cli.command {
        Commands::Workpoint(commands::workpoint::WorkpointCmd::RepairStage {
            confirm_lifecycle_repair,
            ..
        }) => *confirm_lifecycle_repair,
        _ => panic!("expected stage-only recovery"),
    }
}

#[test]
fn repair_stage_generic_confirm_does_not_require_installer_lifecycle() {
    let mut args = repair_args();
    args.push("--confirm".into());
    let (mut cli, schema, matches) = parse_cli(args);
    prepare_command_confirmation(&mut cli, &schema, &matches).unwrap();
    assert!(stage_confirmed(&cli));
    assert!(!cli.confirm);
    assert!(cli.lifecycle_action.is_none());
}

#[test]
fn repair_stage_dedicated_confirmation_remains_supported() {
    let mut args = repair_args();
    args.push("--confirm-lifecycle-repair".into());
    let (mut cli, schema, matches) = parse_cli(args);
    prepare_command_confirmation(&mut cli, &schema, &matches).unwrap();
    assert!(stage_confirmed(&cli));
    assert!(cli.lifecycle_action.is_none());
}

#[test]
fn repair_stage_does_not_infer_confirmation() {
    let (mut cli, schema, matches) = parse_cli(repair_args());
    prepare_command_confirmation(&mut cli, &schema, &matches).unwrap();
    assert!(!stage_confirmed(&cli));
}

#[test]
fn generic_confirmation_does_not_approve_an_unrelated_command() {
    for args in [
        vec!["focusa", "status", "--confirm"],
        vec!["focusa", "install", "--confirm"],
        vec![
            "focusa",
            "trajectory",
            "define-goal",
            "--long-term-goal",
            "Fixture goal",
            "--desired-end-state",
            "Fixture proof",
            "--confirm",
        ],
    ] {
        let (mut cli, schema, matches) = parse_cli(args.into_iter().map(str::to_string).collect());
        assert!(!selected_command_owns_confirmation(&schema, &matches));
        assert!(
            prepare_command_confirmation(&mut cli, &schema, &matches)
                .unwrap_err()
                .to_string()
                .contains("requires --lifecycle-action")
        );
    }
}

#[test]
fn installer_confirmation_still_requires_its_existing_guided_flow() {
    let (mut cli, schema, matches) = parse_cli(
        [
            "focusa",
            "install",
            "--lifecycle-action",
            "repair",
            "--confirm",
        ]
        .map(str::to_string)
        .to_vec(),
    );
    prepare_command_confirmation(&mut cli, &schema, &matches).unwrap();
    assert!(cli.confirm);
    assert!(cli.lifecycle_action.is_some());
}

#[test]
fn marker_repair_confirmation_is_owned_by_trajectory_guard_not_installer() {
    let (mut cli, schema, matches) = parse_cli(
        [
            "focusa",
            "project",
            "trajectory-guard",
            "--project-root",
            "/fixture/project",
            "--action",
            "repair",
            "--confirm",
        ]
        .map(str::to_string)
        .to_vec(),
    );
    assert!(selected_command_owns_confirmation(&schema, &matches));
    prepare_command_confirmation(&mut cli, &schema, &matches).unwrap();
    match cli.command {
        Commands::Project(commands::project::ProjectCmd::TrajectoryGuard { confirm, .. }) => {
            assert!(confirm)
        }
        _ => panic!("expected marker guard repair"),
    }
    assert!(!cli.confirm);
    assert!(cli.lifecycle_action.is_none());
}

#[test]
fn selected_command_schema_owns_genesis_bootstrap_and_temporal_confirmation() {
    let cases = [
        vec![
            "focusa",
            "project",
            "genesis",
            "commit",
            "--project-root",
            "/fixture/project",
            "--continuity-id",
            "fixture",
            "--idempotency-key",
            "genesis",
            "--confirm",
        ],
        vec![
            "focusa",
            "project",
            "bootstrap",
            "apply",
            "--project-root",
            "/fixture/project",
            "--project-id",
            "fixture",
            "--canonical-name",
            "Fixture",
            "--continuity-id",
            "fixture",
            "--idempotency-key",
            "bootstrap",
            "--confirm",
        ],
        vec![
            "focusa",
            "temporal",
            "commit",
            "--project-root",
            "/fixture/project",
            "--continuity-id",
            "fixture",
            "--idempotency-key",
            "temporal",
            "--claim-id",
            "claim",
            "--kind",
            "forecast",
            "--subject-ref",
            "subject",
            "--timezone",
            "UTC",
            "--source",
            "operator",
            "--confirm",
        ],
    ];
    for args in cases {
        let (mut cli, schema, matches) = parse_cli(args.into_iter().map(str::to_string).collect());
        assert!(selected_command_owns_confirmation(&schema, &matches));
        prepare_command_confirmation(&mut cli, &schema, &matches).unwrap();
        assert!(
            !cli.confirm,
            "command-owned flags must not become installer approval"
        );
        assert!(
            selected_command_owns_confirmation(&schema, &matches),
            "actual selected confirmation stays intact"
        );
        assert!(cli.lifecycle_action.is_none());
    }
}
