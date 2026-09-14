//! `focusa remote` — controller-side RemoteWorkspaceBinding CLI (#89 slice 4).
//!
//! bind/status/revoke surface the daemon's binding routes typed earlier
//! (slice 3). This is the controller's view: identity is immutable, and
//! revocation is a typed transition — nothing is ever deleted.

use clap::{Args, Subcommand};
use serde_json::{Value, json};

#[derive(Args, Debug)]
pub struct RemoteArgs {
    #[command(subcommand)]
    pub cmd: RemoteCmd,
}

#[derive(Subcommand, Debug)]
pub enum RemoteCmd {
    /// Bind a remote workspace (SSH checkout) to this controller daemon.
    Bind(BindArgs),
    /// List bindings (optionally filtered by status).
    Status(StatusArgs),
    /// Revoke a binding (typed transition; never deleted).
    Revoke(RevokeArgs),
}

#[derive(Args, Debug)]
pub struct BindArgs {
    /// Binding id (stable identity).
    #[arg(long)]
    pub binding_id: String,
    /// Project id.
    #[arg(long)]
    pub project_id: String,
    /// Repository remote (git URL).
    #[arg(long)]
    pub repo_remote: String,
    /// SSH host.
    #[arg(long)]
    pub host: String,
    /// SSH user.
    #[arg(long)]
    pub user: String,
    /// SSH port.
    #[arg(long, default_value = "22")]
    pub port: u16,
    /// Canonical remote project root.
    #[arg(long)]
    pub remote_root: String,
    /// Continuity id for the workstream.
    #[arg(long)]
    pub continuity_id: String,
    /// Principal (e.g. team:planmarr).
    #[arg(long)]
    pub principal: Option<String>,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Filter by status (pending/verified/stale/revoked).
    #[arg(long)]
    pub status: Option<String>,
}

#[derive(Args, Debug)]
pub struct RevokeArgs {
    /// Binding id.
    #[arg(long)]
    pub binding_id: String,
    /// Revocation reason.
    #[arg(long, default_value = "operator")]
    pub reason: String,
}

pub async fn run(cmd: RemoteCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = crate::api_client::ApiClient::new();
    let result: Value = match cmd {
        RemoteCmd::Bind(args) => {
            let binding = json!({
                "schema": "focusa.remote_workspace_binding.v1",
                "binding_id": args.binding_id,
                "controller": {
                    "daemon_identity": "anchor-server",
                    "controller_origin": "cli"
                },
                "project": {
                    "project_id": args.project_id,
                    "repo_remote": args.repo_remote
                },
                "transport": {
                    "kind": "ssh",
                    "host": args.host,
                    "user": args.user,
                    "port": args.port,
                    "host_reference": null,
                    "verified_at": null,
                    "verification_evidence": []
                },
                "roots": {
                    "canonical_remote_root": args.remote_root,
                    "deploy_root": null,
                    "working_subpath": null,
                    "worktree_identity": null
                },
                "session": {
                    "continuity_id": args.continuity_id,
                    "principal": args.principal
                },
                "state": {
                    "status": "pending",
                    "freshness": null,
                    "revocation": null
                }
            });
            api.post("/v1/remote-workspaces/bindings", &binding).await?
        }
        RemoteCmd::Status(args) => {
            let query = args
                .status
                .as_deref()
                .map(|status| format!("?status={status}"))
                .unwrap_or_default();
            api.get(&format!("/v1/remote-workspaces/bindings{query}"))
                .await?
        }
        RemoteCmd::Revoke(args) => {
            api.post(
                "/v1/remote-workspaces/bindings/revoke",
                &json!({"binding_id": args.binding_id, "reason": args.reason}),
            )
            .await?
        }
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
