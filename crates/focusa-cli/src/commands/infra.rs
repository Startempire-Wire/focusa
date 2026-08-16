//! `focusa infra` — read-only project infrastructure inventory and the
//! preview-only adoption plan (#255). The scan always runs against the
//! local target workspace (never inferred from a remote host's absence).

use clap::{Args, Subcommand};
use std::collections::BTreeMap;

#[derive(Args, Debug)]
pub struct InfraArgs {
    #[command(subcommand)]
    pub cmd: InfraCmd,
}

#[derive(Subcommand, Debug)]
pub enum InfraCmd {
    /// Scan the workspace and print the detected infrastructure inventory.
    Inventory(InventoryArgs),
    /// Build the PREVIEW-ONLY adoption plan from the current inventory.
    Adopt(InventoryArgs),
}

#[derive(Args, Debug)]
pub struct InventoryArgs {
    /// Workspace root (defaults to the current directory).
    #[arg(long)]
    pub root: Option<String>,
    /// Operator provider overrides as concern=provider pairs (repeatable).
    #[arg(long)]
    pub override_provider: Vec<String>,
}

pub async fn run(cmd: InfraCmd, json_mode: bool) -> anyhow::Result<()> {
    let (args, adopt) = match cmd {
        InfraCmd::Inventory(args) => (args, false),
        InfraCmd::Adopt(args) => (args, true),
    };
    let root = args
        .root
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });
    let mut overrides = BTreeMap::new();
    for pair in &args.override_provider {
        let (concern, provider) = pair
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("override must be concern=provider, got {pair}"))?;
        overrides.insert(concern.to_string(), provider.to_string());
    }

    let inventory = focusa_core::infrastructure_inventory::scan_infrastructure(
        std::path::Path::new(&root),
    );

    if json_mode {
        let value = if adopt {
            serde_json::to_value(focusa_core::infrastructure_inventory::build_adoption_plan(
                &inventory,
                &overrides,
            ))?
        } else {
            serde_json::to_value(&inventory)?
        };
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    for (concern, detection) in &inventory.detections {
        let evidence = if detection.evidence_paths.is_empty() {
            "-".to_string()
        } else {
            detection.evidence_paths.join(", ")
        };
        println!(
            "{:<24} {:<10} {}",
            concern,
            detection.outcome.as_str(),
            evidence
        );
    }

    if adopt {
        println!();
        let plan = focusa_core::infrastructure_inventory::build_adoption_plan(
            &inventory,
            &overrides,
        );
        println!("adoption plan (PREVIEW ONLY):");
        for decision in &plan.decisions {
            println!(
                "  {:<24} {:<28} {} ({})",
                decision.concern,
                decision.action,
                decision
                    .selected_provider
                    .as_deref()
                    .unwrap_or("none"),
                decision.selection_basis
            );
        }
        if plan.requires_operator_approval {
            println!("operator approval required before any mutation");
        }
        if !plan.missing_capabilities.is_empty() {
            println!(
                "proposed additions: {}",
                plan.missing_capabilities.join(", ")
            );
        }
    }
    Ok(())
}
