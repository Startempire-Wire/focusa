use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::Value;

#[derive(Subcommand)]
pub enum AwarenessCmd {
    /// Render a Focusa Utility Card for non-Pi agents/harnesses.
    Card {
        /// Agent adapter id, e.g. openclaw, claude-code, opencode, letta.
        #[arg(long, default_value = "non-pi-agent")]
        adapter_id: String,
        /// Workspace id, e.g. wirebot.
        #[arg(long, default_value = "unknown-workspace")]
        workspace_id: String,
        /// Agent id, e.g. wirebot.
        #[arg(long)]
        agent_id: Option<String>,
        /// Operator id, e.g. verious.smith.
        #[arg(long)]
        operator_id: Option<String>,
        /// Harness/session id for scoped Workpoint lookup.
        #[arg(long)]
        session_id: Option<String>,
        /// Project/workspace root for scoped Workpoint lookup.
        #[arg(long)]
        project_root: Option<String>,
    },
}

fn encode_query(value: &str) -> String {
    value
        .bytes()
        .flat_map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![b as char]
            }
            b' ' => vec!['+'],
            _ => format!("%{b:02X}").chars().collect(),
        })
        .collect()
}

fn push_query(parts: &mut Vec<String>, key: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) {
        parts.push(format!("{}={}", key, encode_query(value)));
    }
}

pub async fn run(cmd: AwarenessCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    match cmd {
        AwarenessCmd::Card {
            adapter_id,
            workspace_id,
            agent_id,
            operator_id,
            session_id,
            project_root,
        } => {
            let mut qs = Vec::new();
            push_query(&mut qs, "adapter_id", Some(&adapter_id));
            push_query(&mut qs, "workspace_id", Some(&workspace_id));
            push_query(&mut qs, "agent_id", agent_id.as_deref());
            push_query(&mut qs, "operator_id", operator_id.as_deref());
            push_query(&mut qs, "session_id", session_id.as_deref());
            push_query(&mut qs, "project_root", project_root.as_deref());
            let path = format!("/v1/awareness/card?{}", qs.join("&"));
            let resp: Value = api.get(&path).await?;
            if json_mode {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else if let Some(card) = resp.get("rendered_card").and_then(|v| v.as_str()) {
                println!("{}", card);
            } else {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        }
    }
    Ok(())
}
