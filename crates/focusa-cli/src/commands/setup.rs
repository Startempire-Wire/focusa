//! Spec124 setup namespace.

use crate::commands::first_mission::{self, FirstMissionArgs};
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum SetupCmd {
    /// Guided setup path: project discovery/selection → optional First Mission.
    Wizard(FirstMissionArgs),
    /// Show canonical setup init migration hint.
    Init,
    /// Show setup doctor migration hint.
    Doctor,
}

pub async fn run(cmd: SetupCmd, json_output: bool) -> anyhow::Result<()> {
    match cmd {
        SetupCmd::Wizard(args) => first_mission::run(args, json_output).await,
        SetupCmd::Init => {
            let payload = json!({
                "schema": "focusa.setup.v1",
                "status": "alias",
                "command": "setup init",
                "canonical": "focusa init",
                "next": "focusa init --quickstart"
            });
            if json_output {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("focusa setup init → focusa init --quickstart");
            }
            Ok(())
        }
        SetupCmd::Doctor => {
            let payload = json!({
                "schema": "focusa.setup.v1",
                "status": "alias",
                "command": "setup doctor",
                "canonical": "focusa doctor",
                "next": "focusa doctor"
            });
            if json_output {
                println!("{}", serde_json::to_string_pretty(&payload)?);
            } else {
                println!("focusa setup doctor → focusa doctor");
            }
            Ok(())
        }
    }
}
