use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::Value;

/// CLI surface: `focusa bloatgaurd report` and `focusa bloatgaurd domain <name>`.
#[derive(Subcommand)]
pub enum BloatgaurdCmd {
    /// Read the Spec101 Bloatgaurd budget report.
    Report,
    /// Read one Spec101 Bloatgaurd budget domain by slug/name.
    Domain { name: String },
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
