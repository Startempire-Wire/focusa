use crate::api_client::ApiClient;
use anyhow::{Context, bail};
use clap::{Args, Subcommand};
use serde_json::{Value, json};

const CONTRACT: &str = "focusa.semantic_integrity.operation.v1";

#[derive(Debug, Subcommand)]
pub enum SemanticIntegrityCmd {
    /// Show truthful integration and degradation status.
    Status(ScopeArgs),
    /// Inspect the stable operation registry (bounded to 100 entries).
    Registry(ListArgs),
    /// List known semantic artifacts (truthfully degraded when storage is absent).
    Artifacts(ListArgs),
    /// Inspect one artifact by stable identifier.
    Inspect(InspectArgs),
    /// Invoke any registered operation through its canonical envelope.
    Invoke(InvokeArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ScopeArgs {
    /// Canonical project root; never inferred by this operation surface.
    #[arg(long)]
    project_root: String,
    /// Canonical continuity identifier.
    #[arg(long)]
    continuity_id: String,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[command(flatten)]
    scope: ScopeArgs,
    /// Opaque numeric cursor returned by the preceding page.
    #[arg(long)]
    cursor: Option<String>,
    /// Page size (server clamps to 1..=100).
    #[arg(long, default_value_t = 50)]
    limit: u16,
    /// Registry family filter (registry only).
    #[arg(long)]
    family: Option<String>,
}

#[derive(Debug, Args)]
pub struct InspectArgs {
    #[command(flatten)]
    scope: ScopeArgs,
    artifact_id: String,
}

#[derive(Debug, Args)]
pub struct InvokeArgs {
    #[command(flatten)]
    scope: ScopeArgs,
    /// Stable operation ID from `semantic-integrity registry`.
    operation_id: String,
    /// JSON object/value passed without local interpretation.
    #[arg(long, default_value = "{}")]
    payload: String,
    /// Required by every mutation.
    #[arg(long)]
    idempotency_key: Option<String>,
    /// Explicit mutation confirmation. Must be the literal `confirm`.
    #[arg(long)]
    confirmation: Option<String>,
}

pub async fn run(cmd: SemanticIntegrityCmd, json_output: bool) -> anyhow::Result<()> {
    let client = ApiClient::new();
    let (method, path, body) = match cmd {
        SemanticIntegrityCmd::Status(scope) => (
            "GET",
            format!("/v1/semantic-integrity/status?{}", scope_query(&scope)),
            None,
        ),
        SemanticIntegrityCmd::Registry(args) => (
            "GET",
            list_path("/v1/semantic-integrity/operations", &args),
            None,
        ),
        SemanticIntegrityCmd::Artifacts(args) => (
            "GET",
            list_path("/v1/semantic-integrity/artifacts", &args),
            None,
        ),
        SemanticIntegrityCmd::Inspect(args) => (
            "GET",
            format!(
                "/v1/semantic-integrity/artifacts/{}?{}",
                urlencoding::encode(&args.artifact_id),
                scope_query(&args.scope)
            ),
            None,
        ),
        SemanticIntegrityCmd::Invoke(args) => {
            let payload: Value =
                serde_json::from_str(&args.payload).context("--payload must be valid JSON")?;
            let path = format!(
                "/v1/semantic-integrity/operations/{}",
                urlencoding::encode(&args.operation_id)
            );
            let body = json!({
                "contract": CONTRACT,
                "operation_id": args.operation_id,
                "scope": {"project_root": args.scope.project_root, "continuity_id": args.scope.continuity_id},
                "payload": payload,
                "idempotency_key": args.idempotency_key,
                "confirmation": args.confirmation,
            });
            ("POST", path, Some(body))
        }
    };

    let url = format!("{}{}", client.base_url(), path);
    let request = match method {
        "GET" => client.http_client().get(&url),
        "POST" => client
            .http_client()
            .post(&url)
            .json(body.as_ref().expect("post body")),
        _ => unreachable!(),
    };
    let response = request
        .send()
        .await
        .context("semantic-integrity API request failed")?;
    let status = response.status();
    let value: Value = response
        .json()
        .await
        .context("semantic-integrity API returned invalid JSON")?;
    render(&value, json_output)?;
    if !status.is_success() {
        bail!("semantic-integrity operation returned HTTP {status}");
    }
    Ok(())
}

fn scope_query(scope: &ScopeArgs) -> String {
    format!(
        "project_root={}&continuity_id={}",
        urlencoding::encode(&scope.project_root),
        urlencoding::encode(&scope.continuity_id)
    )
}

fn list_path(base: &str, args: &ListArgs) -> String {
    let mut query = format!("{}&limit={}", scope_query(&args.scope), args.limit);
    if let Some(cursor) = &args.cursor {
        query.push_str(&format!("&cursor={}", urlencoding::encode(cursor)));
    }
    if let Some(family) = &args.family {
        query.push_str(&format!("&family={}", urlencoding::encode(family)));
    }
    format!("{base}?{query}")
}

fn render(value: &Value, json_output: bool) -> anyhow::Result<()> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(value)?);
        return Ok(());
    }
    if let Some(items) = value.get("items").and_then(Value::as_array) {
        for item in items {
            println!(
                "{}\t{}\t{}",
                text(item, "operation_id"),
                text(item, "family"),
                text(item, "availability")
            );
        }
        if items.is_empty() {
            println!("No materialized items.");
        }
        if value.get("degraded").and_then(Value::as_bool) == Some(true) {
            println!("Degraded: {}", text(value, "degraded_reason"));
        }
        if let Some(cursor) = value.get("next_cursor").and_then(Value::as_str) {
            println!("Next cursor: {cursor}");
        }
    } else {
        println!("Operation: {}", text(value, "operation_id"));
        println!(
            "State: {}{}",
            text(value, "state"),
            if value.get("degraded").and_then(Value::as_bool) == Some(true) {
                " (degraded)"
            } else {
                ""
            }
        );
        println!("{}", text(value, "message"));
        print_refs("Evidence", value.get("evidence_refs"));
        print_refs("Receipts", value.get("receipt_refs"));
    }
    Ok(())
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or("-")
}

fn print_refs(label: &str, refs: Option<&Value>) {
    if let Some(refs) = refs.and_then(Value::as_array).filter(|v| !v.is_empty()) {
        let joined = refs
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        println!("{label}: {joined}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_is_exact_and_url_encoded() {
        let scope = ScopeArgs {
            project_root: "/tmp/a b".into(),
            continuity_id: "c/1".into(),
        };
        assert_eq!(
            scope_query(&scope),
            "project_root=%2Ftmp%2Fa%20b&continuity_id=c%2F1"
        );
    }

    #[test]
    fn list_is_bounded_and_carries_family() {
        let args = ListArgs {
            scope: ScopeArgs {
                project_root: "/p".into(),
                continuity_id: "c".into(),
            },
            cursor: Some("50".into()),
            limit: 100,
            family: Some("verify".into()),
        };
        let path = list_path("/v1/semantic-integrity/operations", &args);
        assert!(
            path.contains("limit=100")
                && path.contains("cursor=50")
                && path.contains("family=verify")
        );
    }

    #[test]
    fn json_contract_name_is_stable() {
        assert_eq!(CONTRACT, "focusa.semantic_integrity.operation.v1");
    }
}
