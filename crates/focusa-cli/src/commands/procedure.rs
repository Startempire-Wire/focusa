//! Stateless procedure preview and integrity qualification; never dispatches work.
use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use focusa_core::procedure_compiler::{
    ExecutionPlan, PROCEDURE_DIGEST_SCHEMA, PROCEDURE_SCHEMA, Procedure, compile_plan, verify_plan,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const PREVIEW_SCHEMA: &str = "focusa.procedure_preview.v1";
const VERIFICATION_SCHEMA: &str = "focusa.procedure_verification.v1";

#[derive(Args, Debug)]
pub struct ProcedureArgs {
    #[command(subcommand)]
    pub cmd: ProcedureCmd,
}

#[derive(Subcommand, Debug)]
pub enum ProcedureCmd {
    /// Describe this binary's exact local compiler contracts; no daemon or execution.
    Describe,
    /// Compile a local typed procedure into an advisory plan; no execution or grants.
    Compile {
        #[arg(long)]
        definition: PathBuf,
    },
    /// Verify a saved compile preview against its exact source and safety projection.
    Verify {
        #[arg(long)]
        definition: PathBuf,
        /// JSON preview emitted by `procedure compile --json`.
        #[arg(long)]
        plan: PathBuf,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Preview {
    schema: String,
    digest_schema: String,
    canonical: bool,
    execution_authorized: bool,
    plan: ExecutionPlan,
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).context("procedure input could not be read")?;
    serde_json::from_slice(&bytes)
        .context("procedure input does not match the supported JSON contract")
}

pub fn run(cmd: ProcedureCmd, json_mode: bool) -> Result<()> {
    match cmd {
        ProcedureCmd::Describe => {
            let descriptor = serde_json::json!({
                "schema":"focusa.procedure_capabilities.v1",
                "binary_version":env!("CARGO_PKG_VERSION"),
                "definition_schema":PROCEDURE_SCHEMA,"preview_schema":PREVIEW_SCHEMA,
                "verification_schema":VERIFICATION_SCHEMA,"digest_schema":PROCEDURE_DIGEST_SCHEMA,
                "transport":"local_cli","daemon_required":false,
                "canonical":false,"execution_authorized":false,"dispatch_supported":false
            });
            println!(
                "{}",
                if json_mode {
                    serde_json::to_string(&descriptor)?
                } else {
                    serde_json::to_string_pretty(&descriptor)?
                }
            );
        }
        ProcedureCmd::Compile { definition } => {
            let source: Procedure = read_json(&definition)?;
            let plan = compile_plan(&source).map_err(anyhow::Error::msg)?;
            let preview = Preview {
                schema: PREVIEW_SCHEMA.into(),
                digest_schema: PROCEDURE_DIGEST_SCHEMA.into(),
                canonical: false,
                execution_authorized: false,
                plan,
            };
            if json_mode {
                println!("{}", serde_json::to_string(&preview)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&preview)?);
            }
        }
        ProcedureCmd::Verify { definition, plan } => {
            let source: Procedure = read_json(&definition)?;
            let preview: Preview = read_json(&plan)?;
            if preview.schema != PREVIEW_SCHEMA || preview.digest_schema != PROCEDURE_DIGEST_SCHEMA
            {
                bail!("unsupported procedure preview or digest schema");
            }
            if preview.canonical || preview.execution_authorized {
                bail!("procedure preview cannot claim canonical or execution authority");
            }
            verify_plan(&source, &preview.plan).map_err(anyhow::Error::msg)?;
            let result = serde_json::json!({"schema":VERIFICATION_SCHEMA,
                "verified":true,"digest_schema":PROCEDURE_DIGEST_SCHEMA,
                "canonical":false,"execution_authorized":false});
            if json_mode {
                println!("{}", serde_json::to_string(&result)?);
            } else {
                println!("Integrity verified; execution is not authorized.");
            }
        }
    }
    Ok(())
}
