//! Stateless procedure preview and integrity qualification; never dispatches work.
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use focusa_core::procedure_compiler::{
    PROCEDURE_DIGEST_SCHEMA, PROCEDURE_PREVIEW_SCHEMA as PREVIEW_SCHEMA, PROCEDURE_SCHEMA,
    PROCEDURE_VERIFICATION_SCHEMA as VERIFICATION_SCHEMA, Procedure, ProcedurePreview as Preview,
    compile_preview, verify_preview,
};
use std::path::{Path, PathBuf};

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
            let preview = compile_preview(&source).map_err(anyhow::Error::msg)?;
            if json_mode {
                println!("{}", serde_json::to_string(&preview)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&preview)?);
            }
        }
        ProcedureCmd::Verify { definition, plan } => {
            let source: Procedure = read_json(&definition)?;
            let preview: Preview = read_json(&plan)?;
            let result = verify_preview(&source, &preview).map_err(anyhow::Error::msg)?;
            if json_mode {
                println!("{}", serde_json::to_string(&result)?);
            } else {
                println!("Integrity verified; execution is not authorized.");
            }
        }
    }
    Ok(())
}
