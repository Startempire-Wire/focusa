use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::Value;

/// CLI surface: `focusa bloatgaurd report`, `focusa bloatgaurd domain <name>`, `focusa bloatgaurd tokenbloat`, `focusa bloatgaurd token-domain <name>`, `focusa bloatgaurd gate-modes`, and `focusa bloatgaurd gate-mode <name>`.
#[derive(Subcommand)]
pub enum BloatgaurdCmd {
    /// Read the Spec101 Bloatgaurd budget report.
    Report,
    /// Read one Spec101 Bloatgaurd budget domain by slug/name.
    Domain { name: String },
    /// Read Spec101 Tokenbloat Control report for domains 5.9-5.10.
    Tokenbloat,
    /// Read one Spec101 Tokenbloat Control domain by slug/name.
    TokenDomain { name: String },
    /// Read Spec101 Bloatgaurd gate modes A/B/C thresholds/allowlist/report schema.
    GateModes,
    /// Read one Spec101 Bloatgaurd gate mode by code/name.
    GateMode { name: String },
}

pub async fn handle(client: &mut ApiClient, cmd: BloatgaurdCmd) -> anyhow::Result<()> {
    match cmd {
        BloatgaurdCmd::Report => {
            let resp = client.get("/v1/bloatgaurd/report").await?;
            print_report(&resp);
        }
        BloatgaurdCmd::Domain { name } => {
            let resp = client.get(&format!("/v1/bloatgaurd/domain/{name}")).await?;
            print_domain(&resp);
        }
        BloatgaurdCmd::Tokenbloat => {
            let resp = client.get("/v1/bloatgaurd/tokenbloat/report").await?;
            print_token_report(&resp);
        }
        BloatgaurdCmd::TokenDomain { name } => {
            let resp = client
                .get(&format!("/v1/bloatgaurd/tokenbloat/domain/{name}"))
                .await?;
            print_domain(&resp);
        }
        BloatgaurdCmd::GateModes => {
            let resp = client.get("/v1/bloatgaurd/gate-modes/report").await?;
            print_gate_modes(&resp);
        }
        BloatgaurdCmd::GateMode { name } => {
            let resp = client
                .get(&format!("/v1/bloatgaurd/gate-modes/mode/{name}"))
                .await?;
            print_gate_mode(&resp);
        }
    }
    Ok(())
}

fn print_report(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let summary = payload
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let domains = payload
        .get("domains")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    println!("bloatgaurd report {status} | domains={domains}");
    println!("summary: {summary}");
    if let Some(items) = payload.get("domains").and_then(Value::as_array) {
        for domain in items.iter().take(8) {
            let name = domain
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let section = domain.get("section").and_then(Value::as_str).unwrap_or("?");
            let title = domain
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            println!("  {section} {name}: {title}");
        }
    }
}

fn print_token_report(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let summary = payload
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let controls = payload
        .get("controls")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    println!("bloatgaurd tokenbloat {status} | controls={controls}");
    println!("summary: {summary}");
    if let Some(items) = payload.get("controls").and_then(Value::as_array) {
        for domain in items.iter().take(4) {
            let name = domain
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let section = domain.get("section").and_then(Value::as_str).unwrap_or("?");
            let title = domain
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            println!("  {section} {name}: {title}");
        }
    }
}

fn print_gate_modes(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let summary = payload
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let modes = payload
        .get("modes")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    println!("bloatgaurd gate-modes {status} | modes={modes}");
    println!("summary: {summary}");
    if let Some(items) = payload.get("modes").and_then(Value::as_array) {
        for mode in items.iter().take(3) {
            let code = mode.get("code").and_then(Value::as_str).unwrap_or("?");
            let name = mode
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let enforcement = mode
                .get("enforcement")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            println!("  {code} {name}: {enforcement}");
        }
    }
}

fn print_gate_mode(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if let Some(mode) = payload.get("mode") {
        let code = mode.get("code").and_then(Value::as_str).unwrap_or("?");
        let name = mode
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let enforcement = mode
            .get("enforcement")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!("bloatgaurd gate-mode {status} | {code} {name}: {enforcement}");
        if let Some(fields) = mode.get("report_schema_fields").and_then(Value::as_array) {
            println!("  report_schema_fields={}", fields.len());
        }
        if let Some(allowlist) = mode.get("allowlist").and_then(Value::as_array) {
            println!("  allowlist_entries={}", allowlist.len());
        }
    } else {
        let requested = payload
            .get("requested_domain")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!("bloatgaurd gate-mode {status} | requested={requested}");
    }
}

fn print_domain(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if let Some(domain) = payload.get("domain") {
        let name = domain
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let section = domain.get("section").and_then(Value::as_str).unwrap_or("?");
        let title = domain
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!("bloatgaurd domain {status} | {section} {name}: {title}");
        if let Some(checks) = domain.get("checks").and_then(Value::as_array) {
            for check in checks.iter().take(6) {
                if let Some(text) = check.as_str() {
                    println!("  check: {text}");
                }
            }
        }
    } else {
        let requested = payload
            .get("requested_domain")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!("bloatgaurd domain {status} | requested={requested}");
    }
}
