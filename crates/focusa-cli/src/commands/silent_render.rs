//! Stable human and JSON rendering for `focusa silent`.

use anyhow::Result;
use serde_json::{Value, json};

pub(super) const CLI_SCHEMA: &str = "focusa.silent_cli.v1";

fn redact(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                let sensitive = ["secret", "token", "credential", "authorization", "api_key"]
                    .iter()
                    .any(|needle| key.to_ascii_lowercase().contains(needle));
                if sensitive {
                    *value = Value::String("[REDACTED]".into());
                } else {
                    redact(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact),
        _ => {}
    }
}

fn wrap(command: &str, mut result: Value) -> Value {
    redact(&mut result);
    json!({
        "schema": CLI_SCHEMA,
        "command": command,
        "status": result.get("status").cloned().unwrap_or_else(|| json!("completed")),
        "canonical": result.get("canonical").cloned().unwrap_or(Value::Null),
        "side_effects": result.get("side_effects").cloned().unwrap_or_else(|| json!([])),
        "session_id": result.pointer("/data/session/id").or_else(|| result.pointer("/session/id")).or_else(|| result.get("session_id")).cloned(),
        "run_id": result.pointer("/data/run/id").or_else(|| result.pointer("/run/id")).or_else(|| result.get("run_id")).cloned(),
        "process_status": result.pointer("/data/process_status").or_else(|| result.get("process_status")).cloned(),
        "completion_status": result.pointer("/data/completion_status").or_else(|| result.get("completion_status")).cloned(),
        "result": result,
    })
}

pub(super) fn print_result(command: &str, result: Value, json_output: bool) -> Result<()> {
    let envelope = wrap(command, result);
    if command == "doctor"
        && envelope["result"]["schema"]
            == CLI_SCHEMA.replace("silent_cli.v1", "silent_cli_doctor.v1")
    {
        if json_output {
            println!("{}", serde_json::to_string_pretty(&envelope["result"])?);
            return Ok(());
        }
        println!("Silent Session Doctor");
        println!(
            "Status: {}",
            envelope["result"]["status"].as_str().unwrap_or("blocked")
        );
        if let Some(checks) = envelope["result"]["data"]["checks"].as_array() {
            for check in checks {
                println!(
                    "[{}] {}: {}",
                    check["status"].as_str().unwrap_or("blocked"),
                    check["component"].as_str().unwrap_or("unknown"),
                    check["summary"].as_str().unwrap_or("No summary.")
                );
            }
        }
        println!(
            "Retry: {}",
            envelope["result"]["retry"]["posture"]
                .as_str()
                .unwrap_or("recover_then_recheck")
        );
        return Ok(());
    }
    if matches!(command, "send" | "steer" | "follow-up" | "key") {
        if json_output {
            println!("{}", serde_json::to_string_pretty(&envelope["result"])?);
            return Ok(());
        }
        let result = &envelope["result"];
        println!(
            "Session: {}",
            result["data"]["session_id"].as_str().unwrap_or("unknown")
        );
        println!(
            "Run: {}",
            result["data"]["run_id"].as_str().unwrap_or("unknown")
        );
        println!(
            "Operation: {}",
            result["data"]["operation"].as_str().unwrap_or(command)
        );
        println!(
            "Replayed: {}",
            result["data"]["replayed"].as_bool().unwrap_or(false)
        );
        println!(
            "Side effects: {}",
            result["side_effects"].as_array().map_or(0, Vec::len)
        );
        for receipt in result["receipt_refs"].as_array().into_iter().flatten() {
            println!("{}", receipt.as_str().unwrap_or_default());
        }
        return Ok(());
    }
    if json_output {
        println!("{}", serde_json::to_string_pretty(&envelope)?);
        return Ok(());
    }
    println!(
        "silent {command} → {}",
        envelope["status"].as_str().unwrap_or("completed")
    );
    for key in [
        "session_id",
        "run_id",
        "process_status",
        "completion_status",
    ] {
        if !envelope[key].is_null() {
            println!("{key}={}", envelope[key]);
        }
    }
    if let Some(side_effects) = envelope["side_effects"].as_array()
        && !side_effects.is_empty()
    {
        println!("side_effects={}", serde_json::to_string(side_effects)?);
    }
    if command == "list"
        || command == "show"
        || command == "doctor"
        || command.starts_with("profile ")
        || command.starts_with("preset ")
        || command.starts_with("config ")
    {
        println!("{}", serde_json::to_string_pretty(&envelope["result"])?);
    }
    Ok(())
}
