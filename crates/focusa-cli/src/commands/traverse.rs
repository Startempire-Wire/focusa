//! Spec96 bounded traversal CLI parity commands.

use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::{Value, json};

#[derive(Subcommand)]
pub enum TraverseCmd {
    /// Read a bounded traversal slice from a Focusa surface.
    Read(TraverseArgs),
    /// Verify traversal tags without returning payload items.
    VerifyTags {
        #[arg(long)]
        surface: String,
        #[arg(long, default_value = "tags_verify")]
        selector: String,
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
}

#[derive(clap::Args, Clone)]
pub struct TraverseArgs {
    #[arg(long)]
    pub surface: String,
    #[arg(long, default_value = "window")]
    pub selector: String,
    #[arg(long)]
    pub anchor: Option<String>,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long)]
    pub cursor: Option<String>,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub depth: Option<u32>,
    #[arg(long)]
    pub radius: Option<u32>,
    #[arg(long = "field")]
    pub fields: Vec<String>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long)]
    pub tag_mode: Option<String>,
    #[arg(long)]
    pub include_payload: bool,
    #[arg(long)]
    pub include_rehydrate_refs: bool,
    #[arg(long)]
    pub budget_tokens: Option<u32>,
}

fn traverse_body(args: TraverseArgs) -> Value {
    json!({
        "surface": args.surface,
        "selector": args.selector,
        "anchor": args.anchor,
        "query": args.query,
        "cursor": args.cursor,
        "limit": args.limit,
        "depth": args.depth,
        "radius": args.radius,
        "fields": args.fields,
        "tags": args.tags,
        "tag_mode": args.tag_mode,
        "include_payload": args.include_payload,
        "include_rehydrate_refs": args.include_rehydrate_refs,
        "budget_tokens": args.budget_tokens,
    })
}

fn print_summary(resp: &Value) {
    let status = resp
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let surface = resp
        .get("surface")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let selector = resp
        .get("selector")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let returned = resp
        .pointer("/traversal/returned")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let total = resp
        .pointer("/traversal/total")
        .and_then(Value::as_u64)
        .unwrap_or(returned);
    let truncated = resp
        .pointer("/traversal/truncated")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    println!(
        "traverse: status={status} surface={surface} selector={selector} returned={returned}/{total} truncated={truncated}"
    );
    if let Some(next_cursor) = resp
        .pointer("/traversal/next_cursor")
        .and_then(Value::as_str)
    {
        println!("  next_cursor: {next_cursor}");
    }
    if let Some(stale) = resp.get("stale_tags").and_then(Value::as_array) {
        println!("  stale_tags: {}", stale.len());
    }
}

pub async fn run(cmd: TraverseCmd, json_output: bool) -> anyhow::Result<()> {
    let api = ApiClient::new();
    let resp = match cmd {
        TraverseCmd::Read(args) => api.post("/v1/traverse", &traverse_body(args)).await?,
        TraverseCmd::VerifyTags {
            surface,
            selector,
            tags,
        } => {
            api.post(
                "/v1/traverse/verify-tags",
                &json!({
                    "surface": surface,
                    "selector": selector,
                    "tags": tags,
                }),
            )
            .await?
        }
    };
    if json_output {
        println!("{}", serde_json::to_string_pretty(&resp)?);
    } else {
        print_summary(&resp);
    }
    Ok(())
}
