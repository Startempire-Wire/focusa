//! `focusa callgraph export` — read a local CallGraph definition JSON and
//! project it through one typed export (#287 slice 2 CLI).

use clap::Args;
use serde_json::Value;

#[derive(Args, Debug)]
pub struct CallgraphArgs {
    #[command(subcommand)]
    pub cmd: CallgraphCmd,
}

#[derive(clap::Subcommand, Debug)]
pub enum CallgraphCmd {
    /// Export a CallGraph definition file through a typed projection.
    Export(ExportArgs),
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Path to a focusa.callgraph.v1 definition JSON file.
    #[arg(long)]
    pub definition: String,
    /// jsonl | todo.txt | dot | csv | tsv | mermaid
    #[arg(long, default_value = "jsonl")]
    pub format: String,
    /// Print the manifest instead of the body.
    #[arg(long, default_value_t = false)]
    pub manifest_only: bool,
}

pub async fn run(cmd: CallgraphCmd, json_mode: bool) -> anyhow::Result<()> {
    match cmd {
        CallgraphCmd::Export(args) => {
            let raw = std::fs::read_to_string(&args.definition)?;
            let graph: focusa_core::callgraph::FocusaCallGraphDefinition =
                serde_json::from_str(&raw)
                    .map_err(|error| anyhow::anyhow!("definition unparsable: {error}"))?;
            let (format_name, lossless, omissions) = match args.format.as_str() {
                "jsonl" => ("jsonl".to_string(), true, vec![]),
                "todo.txt" => (
                    "todo.txt".to_string(),
                    false,
                    vec!["edge semantics flattened to dep: tags".to_string()],
                ),
                "dot" => ("dot".to_string(), true, vec![]),
                "csv" => ("csv".to_string(), true, vec![]),
                "tsv" => ("tsv".to_string(), true, vec![]),
                "mermaid" => ("mermaid".to_string(), true, vec![]),
                other => anyhow::bail!(
                    "unknown format {other}; supported: jsonl, todo.txt, dot, csv, tsv, mermaid"
                ),
            };
            let projection = focusa_core::callgraph_export::CallGraphExportProjection::new(
                graph,
                vec![],
                &format_name,
                lossless,
                omissions,
            );
            if args.manifest_only {
                let value = serde_json::to_value(&projection.manifest)?;
                if json_mode {
                    println!("{}", serde_json::to_string_pretty(&value)?);
                } else {
                    println!("{}", serde_json::to_string(&value)?);
                }
                return Ok(());
            }
            let body = match args.format.as_str() {
                "jsonl" => focusa_core::callgraph_export::export_jsonl(&projection),
                "todo.txt" => focusa_core::callgraph_export::export_todo_txt(&projection),
                "dot" => focusa_core::callgraph_export::export_dot(&projection),
                "csv" => focusa_core::callgraph_export::export_csv(&projection, ','),
                "tsv" => focusa_core::callgraph_export::export_csv(&projection, '\t'),
                "mermaid" => focusa_core::callgraph_export::export_mermaid(&projection),
                _ => unreachable!("format validated above"),
            };
            print!("{body}");
            if json_mode {
                let _ = Value::Null;
            }
            Ok(())
        }
    }
}
