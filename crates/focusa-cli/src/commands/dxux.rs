use crate::api_client::ApiClient;
use clap::Subcommand;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Subcommand)]
pub enum DxuxCmd {
    /// Read Spec105 DX/UX implementation report.
    Report,
    /// Read one DXUX requirement by id, e.g. DXUX-004.
    Requirement { id: String },
    /// Read compact continuation/doability digest.
    Digest,
}

pub async fn handle(client: &mut ApiClient, cmd: DxuxCmd) -> anyhow::Result<()> {
    match cmd {
        DxuxCmd::Report => print_report(&client.get("/v1/dxux/report").await?),
        DxuxCmd::Requirement { id } => {
            print_requirement(&client.get(&format!("/v1/dxux/requirement/{id}")).await?)
        }
        DxuxCmd::Digest => println!(
            "{}",
            serde_json::to_string_pretty(&client.get("/v1/dxux/digest").await?)?
        ),
    }
    Ok(())
}

pub async fn explain(failure: String) -> anyhow::Result<()> {
    let client = ApiClient::new();
    let encoded = failure.replace('/', "%2F");
    let payload = client.get(&format!("/v1/dxux/explain/{encoded}")).await?;
    let summary = payload
        .get("root_cause_summary")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let confidence = payload
        .get("confidence")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    println!("explain completed | confidence={confidence}");
    println!("root_cause: {summary}");
    if let Some(commands) = payload.get("recovery_commands").and_then(Value::as_array) {
        for command in commands {
            if let Some(text) = command.as_str() {
                println!("  recover: {text}");
            }
        }
    }
    Ok(())
}

fn is_preflight_root(path: &Path) -> bool {
    path.join("Cargo.toml").is_file()
        && path
            .join("scripts/validate-focusa-tool-contracts.mjs")
            .is_file()
        && path
            .join("tests/spec101_bloatgaurd_budgets_static_test.py")
            .is_file()
}

fn find_preflight_root(candidates: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    for candidate in candidates {
        for ancestor in candidate.ancestors() {
            if is_preflight_root(ancestor) {
                return Some(ancestor.to_path_buf());
            }
        }
    }
    None
}

fn resolve_preflight_root() -> anyhow::Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(root) = std::env::var_os("FOCUSA_SOURCE_ROOT") {
        candidates.push(PathBuf::from(root));
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        candidates.push(exe);
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    find_preflight_root(candidates).ok_or_else(|| {
        anyhow::anyhow!(
            "Focusa source checkout not found; run preflight from the repository or set FOCUSA_SOURCE_ROOT"
        )
    })
}

/// CLI usage: `focusa preflight`.
pub async fn preflight() -> anyhow::Result<()> {
    let root = resolve_preflight_root()?;
    let commands = [
        "cargo test --workspace",
        "cargo clippy --workspace -- -D warnings",
        "node scripts/validate-focusa-tool-contracts.mjs",
        "python3 tests/spec101_bloatgaurd_budgets_static_test.py",
        "scripts/enforce_bd_closure_evidence.sh",
    ];
    println!(
        "preflight started | root={} commands={}",
        root.display(),
        commands.len()
    );
    for command in commands {
        println!("preflight running: {command}");
        let status = Command::new("bash")
            .arg("-lc")
            .arg(command)
            .current_dir(&root)
            .status()?;
        if !status.success() {
            anyhow::bail!("preflight failed: {command} exit={status}");
        }
    }
    println!("preflight completed | status=ok");
    Ok(())
}

fn print_report(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let count = payload
        .get("requirements")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    println!("dxux report {status} | requirements={count}");
    if let Some(requirements) = payload.get("requirements").and_then(Value::as_array) {
        for req in requirements.iter().take(12) {
            let id = req.get("id").and_then(Value::as_str).unwrap_or("unknown");
            let title = req
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            println!("  {id}: {title}");
        }
    }
}

fn print_requirement(payload: &Value) {
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if let Some(req) = payload.get("requirement") {
        let id = req.get("id").and_then(Value::as_str).unwrap_or("unknown");
        let title = req
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        println!("dxux requirement {status} | {id}: {title}");
    } else {
        println!("dxux requirement {status}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_root_resolves_from_nested_candidate() {
        let root =
            std::env::temp_dir().join(format!("focusa-preflight-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("scripts")).unwrap();
        std::fs::create_dir_all(root.join("tests")).unwrap();
        std::fs::create_dir_all(root.join("target/debug")).unwrap();
        std::fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
        std::fs::write(root.join("scripts/validate-focusa-tool-contracts.mjs"), "").unwrap();
        std::fs::write(
            root.join("tests/spec101_bloatgaurd_budgets_static_test.py"),
            "",
        )
        .unwrap();
        let resolved = find_preflight_root([root.join("target/debug/focusa")]);
        assert_eq!(resolved.as_deref(), Some(root.as_path()));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn preflight_root_rejects_unrelated_directory() {
        let unrelated =
            std::env::temp_dir().join(format!("focusa-preflight-unrelated-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&unrelated);
        std::fs::create_dir_all(&unrelated).unwrap();
        assert!(find_preflight_root([unrelated.clone()]).is_none());
        let _ = std::fs::remove_dir_all(unrelated);
    }
}
