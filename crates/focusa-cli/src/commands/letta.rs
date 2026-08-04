use crate::api_client::ApiClient;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum LettaCmd {
    /// Show adapter availability, bounded authority, evidence, and recovery.
    Status,
}

pub async fn run(command: LettaCmd, output_json: bool) -> anyhow::Result<()> {
    match command {
        LettaCmd::Status => {
            let status = ApiClient::new().get("/v1/letta/status").await?;
            if output_json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else {
                println!(
                    "Focusa Letta: availability={} identity={} active_operation={} evidence_refs={} recovery={} next={} controls={}",
                    status["availability"].as_str().unwrap_or("unknown"),
                    status["identity"].as_str().unwrap_or("none"),
                    status["active_operation"].as_str().unwrap_or("none"),
                    status["evidence_refs"].as_array().map_or(0, Vec::len),
                    status
                        .pointer("/recovery/required")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                    status
                        .pointer("/recovery/next_action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("inspect"),
                    status["controls"].as_array().map_or(0, Vec::len)
                );
                println!(
                    "Mutation controls remain disabled unless canonical status marks them supported."
                );
            }
        }
    }
    Ok(())
}
