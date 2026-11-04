use super::*;

fn repair_args() -> Vec<String> {
    // Parsing is pure: this placeholder path is never inspected or mutated.
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

fn parse_cli(args: Vec<String>) -> Cli {
    // Match the production parser's stack allowance on Windows and Unix.
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || Cli::try_parse_from(args))
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
    let mut cli = parse_cli(args);
    prepare_command_confirmation(&mut cli).expect("stage recovery is not an installer flow");
    assert!(stage_confirmed(&cli));
    assert!(!cli.confirm);
    assert!(cli.lifecycle_action.is_none());
}

#[test]
fn repair_stage_dedicated_confirmation_remains_supported() {
    let mut args = repair_args();
    args.push("--confirm-lifecycle-repair".into());
    let mut cli = parse_cli(args);
    prepare_command_confirmation(&mut cli).unwrap();
    assert!(stage_confirmed(&cli));
    assert!(cli.lifecycle_action.is_none());
}

#[test]
fn repair_stage_does_not_infer_confirmation() {
    let mut cli = parse_cli(repair_args());
    prepare_command_confirmation(&mut cli).unwrap();
    assert!(!stage_confirmed(&cli));
}

#[test]
fn generic_confirmation_does_not_approve_an_unrelated_command() {
    let mut cli = parse_cli(
        ["focusa", "status", "--confirm"]
            .map(str::to_string)
            .to_vec(),
    );
    let error = prepare_command_confirmation(&mut cli).unwrap_err();
    assert!(error.to_string().contains("requires --lifecycle-action"));
}

#[test]
fn installer_confirmation_still_requires_its_existing_guided_flow() {
    let mut cli = parse_cli(
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
    prepare_command_confirmation(&mut cli).unwrap();
    assert!(cli.confirm);
    assert!(cli.lifecycle_action.is_some());
}
