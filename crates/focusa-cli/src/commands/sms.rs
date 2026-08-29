//! Customer-owned communications intelligence CLI (Plan 180).
//! Thin adapter: all policy, connector custody, audit, and persistence remain
//! in the daemon/private broker. Message bodies for send are stdin-only.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};
use std::io::{self, Read};

#[derive(Subcommand)]
pub enum SmsCmd {
    /// Value-free connector and encrypted-checkpoint health.
    Health,
    /// Customer-owned enrollment/pairing status without profile details.
    Enrollment,
    /// List authorized thread summaries.
    Threads {
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    /// Read a bounded authorized thread.
    Read {
        thread_handle: String,
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    /// Search authorized messages.
    Search {
        query: String,
        #[arg(long, default_value_t = 50)]
        limit: u32,
    },
    /// Send a message. Body is read from stdin and never accepted in argv.
    Send {
        recipient_handle: String,
        #[arg(long)]
        idempotency_key: String,
        #[arg(long)]
        grant_id: String,
        #[arg(long)]
        consumer_ref: String,
        #[arg(long)]
        confirm: bool,
    },
    /// Register an active provider OTP challenge before requesting delivery.
    #[command(name = "otp-challenge")]
    OtpChallenge {
        #[arg(long)]
        provider: String,
        #[arg(long)]
        target_handle: String,
        #[arg(long)]
        consumer_ref: String,
        #[arg(long, default_value_t = 300)]
        ttl_seconds: u64,
    },
    /// Inject one eligible OTP directly into its exact bound target.
    #[command(name = "otp-inject")]
    OtpInject {
        #[arg(long)]
        challenge_handle: String,
        #[arg(long)]
        target_handle: String,
        #[arg(long)]
        consumer_ref: String,
    },
    /// Atomically checkpoint connector state.
    Checkpoint,
    /// Read bounded value-free event metadata.
    Events {
        #[arg(long)]
        since: Option<String>,
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    /// Revoke connector and grants. Explicit confirmation required.
    Revoke {
        #[arg(long)]
        connector_id: String,
        #[arg(long)]
        confirm: bool,
    },
}

fn render(value: Value, json_mode: bool) -> anyhow::Result<()> {
    if json_mode {
        println!("{}", serde_json::to_string(&value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&value)?);
    }
    Ok(())
}

pub async fn run(cmd: SmsCmd, json_mode: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let response = match cmd {
        SmsCmd::Health => api.get("/v1/sms/health").await?,
        SmsCmd::Enrollment => api.get("/v1/sms/enrollment").await?,
        SmsCmd::Threads { limit } => api.get(&format!("/v1/sms/threads?limit={limit}")).await?,
        SmsCmd::Read { thread_handle, limit } => api.get(&format!("/v1/sms/threads/{}/messages?limit={limit}", urlencoding::encode(&thread_handle))).await?,
        SmsCmd::Search { query, limit } => api.get(&format!("/v1/sms/search?query={}&limit={limit}", urlencoding::encode(&query))).await?,
        SmsCmd::Send { recipient_handle, idempotency_key, grant_id, consumer_ref, confirm } => {
            if !confirm { anyhow::bail!("send requires --confirm"); }
            let mut body = String::new(); io::stdin().read_to_string(&mut body)?;
            if body.trim().is_empty() { anyhow::bail!("message body on stdin must be non-empty"); }
            api.post("/v1/sms/send", &json!({"recipient_handles":[recipient_handle],"body":body,"idempotency_key":idempotency_key,"grant_id":grant_id,"consumer_ref":consumer_ref,"confirm":true})).await?
        }
        SmsCmd::OtpChallenge { provider, target_handle, consumer_ref, ttl_seconds } => api.post("/v1/sms/otp/challenges", &json!({"provider":provider,"target_handle":target_handle,"consumer_ref":consumer_ref,"ttl_seconds":ttl_seconds})).await?,
        SmsCmd::OtpInject { challenge_handle, target_handle, consumer_ref } => api.post("/v1/sms/otp/inject", &json!({"challenge_handle":challenge_handle,"target_handle":target_handle,"consumer_ref":consumer_ref})).await?,
        SmsCmd::Checkpoint => api.post("/v1/sms/checkpoint", &json!({"confirm":true})).await?,
        SmsCmd::Events { since, limit } => {
            let mut path = format!("/v1/sms/events?limit={limit}"); if let Some(value)=since { path.push_str("&since="); path.push_str(&urlencoding::encode(&value)); } api.get(&path).await?
        }
        SmsCmd::Revoke { connector_id, confirm } => {
            if !confirm { anyhow::bail!("revoke requires --confirm"); }
            api.post("/v1/sms/revoke", &json!({"connector_id":connector_id,"confirm":true})).await?
        }
    };
    render(response, json_mode)
}
