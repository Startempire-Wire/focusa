//! Spec124 command hierarchy and migration help.

use clap::{Args, Subcommand};
use serde_json::{Value, json};

const SPEC141_CLI_PROJECTION: &str =
    include_str!("../../../../docs/contracts/spec141/generated-capability-v2/cli-commands.json");

#[derive(Args, Debug)]
pub struct HelpArgs {
    /// Help topic. Use `all` for inventory or `migration` for old → new commands.
    #[command(subcommand)]
    pub topic: Option<HelpTopic>,
}

#[derive(Subcommand, Debug)]
pub enum HelpTopic {
    /// Full curated command inventory.
    All,
    /// Project workflow commands.
    Project,
    /// Workpoint continuity commands.
    Workpoint,
    /// Old → new command migration map.
    Migration,
}

const MIGRATIONS: &[(&str, &str, &str)] = &[
    (
        "focusa init",
        "focusa project new / focusa setup init",
        "deprecated alias",
    ),
    ("focusa onboard", "focusa setup wizard", "deprecated alias"),
    (
        "focusa preflight",
        "focusa setup doctor / focusa quality preflight",
        "deprecated alias",
    ),
    ("focusa pair", "focusa pairing start", "deprecated alias"),
    (
        "focusa pairing-doctor",
        "focusa pairing doctor",
        "deprecated alias",
    ),
    (
        "focusa pairing-transport",
        "focusa pairing transport",
        "deprecated alias",
    ),
    (
        "focusa pairing-wizard",
        "focusa pairing wizard",
        "deprecated alias",
    ),
    ("focusa stack", "focusa focus stack", "deprecated alias"),
    (
        "focusa start",
        "focusa lifecycle start",
        "planned lifecycle grouping",
    ),
    (
        "focusa stop",
        "focusa lifecycle stop",
        "planned lifecycle grouping",
    ),
    (
        "focusa install",
        "focusa lifecycle install",
        "planned lifecycle grouping",
    ),
    (
        "focusa uninstall",
        "focusa lifecycle uninstall",
        "planned lifecycle grouping",
    ),
    (
        "focusa upgrade",
        "focusa lifecycle upgrade",
        "planned lifecycle grouping",
    ),
    (
        "focusa install-service",
        "focusa lifecycle install-service",
        "planned lifecycle grouping",
    ),
    (
        "focusa codesign",
        "focusa lifecycle codesign",
        "planned lifecycle grouping",
    ),
];

fn inventory_lines() -> Vec<&'static str> {
    vec![
        "focusa project              Project dashboard, discovery, selection, creation, settings",
        "focusa first-mission        Guided project → Workpoint → proof → Mission Deck handoff",
        "focusa setup                Setup wizard/init/doctor aliases",
        "focusa deck                 User-facing Mission Deck launcher",
        "focusa status               Operator/agent status cards",
        "focusa workpoint            Workpoint checkpoint/resume/evidence continuity",
        "focusa trajectory           Per-project north-star trajectory surfaces",
        "focusa focus                Focus stack and Focus State",
        "focusa pairing              Phone/Mac/device pairing namespace",
        "focusa action               Context Authority and mutation preflight",
        "focusa doctor               Health, contracts, runtime, scope, and recovery checks",
        "focusa release              Release proof orchestration",
        "focusa silent               Durable Silent Session create/control/observe/config/retention",
        "focusa update               Trusted OTA inventory/policy/apply/rollback status",
        "focusa uninstall            Remove managed software; preserve data with --keep-data",
        "focusa tui                  Terminal Mission Deck or headless diagnostics",
        "focusa help migration       Old → new command map",
    ]
}

fn print_migration() {
    println!("FOCUSA COMMAND MIGRATION\n");
    println!(
        "Deprecated aliases warn for 90 days after their canonical replacement ships in a tagged release.\n"
    );
    for (old, new, note) in MIGRATIONS {
        println!("{old:<28} → {new:<42} ({note})");
    }
    println!("\nNext:");
    println!("  focusa help all");
    println!("  focusa project");
    println!("  focusa first-mission --dry-run --json");
}

pub fn print_root_help() {
    println!("FOCUSA QUICK HELP\n");
    println!("Start here:");
    println!("  focusa about               What Focusa does and the core concepts");
    println!("  focusa project             Select or create the active project");
    println!("  focusa first-mission       Guided project → Workpoint → proof flow");
    println!("  focusa status              Current daemon, project, and work status");
    println!("  focusa deck                Open the Mission Deck");
    println!("  focusa doctor              Diagnose setup and recovery issues");
    println!("\nInstall:");
    println!("  focusa install --preflight --json");
    println!("  focusa install --dry-run --json");
    println!("\nMore:");
    println!("  focusa help all            Curated command inventory");
    println!("  focusa help migration      Deprecated → canonical commands");
    println!("  focusa <command> --help    Command-specific options");
    println!("\nGlobal options: --json --config <PATH> --verbose --quiet --version");
}

fn print_all() {
    println!("FOCUSA COMMANDS\n");
    for line in inventory_lines() {
        println!("  {line}");
    }
    println!("\nNext:");
    println!("  focusa project");
    println!("  focusa first-mission --dry-run --json");
    println!("  focusa help migration");
}

fn print_project() {
    println!("FOCUSA PROJECT HELP\n");
    println!("  focusa project");
    println!(
        "  focusa project list|discover|use|bind|switch|current|status|remove|new|templates|settings"
    );
    println!(
        "\nAuthority: selected CLI project is convenience only; scoped mutations still require verified project_root."
    );
}

fn print_workpoint() {
    println!("FOCUSA WORKPOINT HELP\n");
    println!("  focusa workpoint current");
    println!("  focusa workpoint checkpoint");
    println!("  focusa workpoint resume");
    println!("  focusa workpoint link-evidence");
    println!("\nUse Workpoints as continuation contracts, not transcript-tail memory.");
}

pub fn warn_alias(old: &str, new: &str) {
    eprintln!("Deprecated alias. Use: {new} (called: {old})");
}

pub fn run(args: HelpArgs, json_output: bool) -> anyhow::Result<()> {
    let topic = args.topic.unwrap_or(HelpTopic::All);
    if json_output {
        let generated: Value = serde_json::from_str(SPEC141_CLI_PROJECTION)?;
        let migrations: Vec<_> = MIGRATIONS
            .iter()
            .map(|(old, new, note)| json!({"old": old, "new": new, "note": note}))
            .collect();
        let payload = json!({
            "schema": "focusa.command_help.v1",
            "status": "completed",
            "topic": format!("{:?}", topic),
            "inventory": inventory_lines(),
            "agent_capability_registry_digest": generated.get("registry_digest"),
            "agent_command_count": generated.get("commands").and_then(Value::as_array).map(Vec::len).unwrap_or_default(),
            "agent_commands": generated.get("commands").cloned().unwrap_or_else(|| json!([])),
            "migrations": migrations,
            "next": ["focusa project", "focusa first-mission --dry-run --json", "focusa help migration"]
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    match topic {
        HelpTopic::All => print_all(),
        HelpTopic::Project => print_project(),
        HelpTopic::Workpoint => print_workpoint(),
        HelpTopic::Migration => print_migration(),
    }
    Ok(())
}
