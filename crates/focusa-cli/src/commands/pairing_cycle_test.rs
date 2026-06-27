//! focusa pairing cycle-test — Revoke + re-pair cycle harness (focusa-ui0y v0.9.35-dev).
//!
//! Runs the full pairing cycle N times against a live daemon:
//!   1. Create room via focusa pairing create-room
//!   2. Mac joins via /v1/connect/room/{id}/join
//!   3. Phone approves via /v1/connect/room/{id}/approve
//!   4. Verify status=completed + token minted
//!   5. Revoke via /v1/device/pair/revoke
//!   6. Idempotency: revoke again, must still return revoked=true
//!   7. List reflects revoked state
//!
//! Replaces the bash test cycle from `docs/57-focusa-pairing-revoke-and-repair.md` §6.
//! Specs:
//!   - docs/55 §7 (revoke + re-pair)
//!   - docs/57 §6 (test cycle)

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::Serialize;
use std::time::Duration;
use tracing::{debug, error, info, warn};

const DEFAULT_ROUNDS: usize = 10;
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8787";

#[derive(Parser, Debug, Clone)]
pub struct CycleTestArgs {
    /// Number of pairing cycles to run.
    #[arg(long, default_value_t = DEFAULT_ROUNDS)]
    pub rounds: usize,
    /// Per-cycle timeout in seconds.
    #[arg(long, default_value_t = DEFAULT_TIMEOUT_SECS)]
    pub timeout: u64,
    /// Base URL of the Focusa daemon (overrides FOCUSA_DAEMON_URL).
    #[arg(long, default_value = DEFAULT_BASE_URL)]
    pub base_url: String,
    /// Host label for paired-device records (default: "cycle-test").
    #[arg(long, default_value = "cycle-test")]
    pub host: String,
    /// Mac name to use across cycles (default: "cycle-mac").
    #[arg(long, default_value = "cycle-mac")]
    pub mac_name: String,
    /// Print machine-readable JSON output.
    #[arg(long)]
    pub json: bool,
    /// Stop on the first failure (default: collect all failures).
    #[arg(long)]
    pub fail_fast: bool,
}

#[derive(Debug, Serialize)]
pub struct CycleTestReport {
    pub rounds_attempted: usize,
    pub rounds_passed: usize,
    pub rounds_failed: usize,
    pub failures: Vec<CycleFailure>,
    pub total_duration_ms: u128,
}

#[derive(Debug, Serialize, Clone)]
pub struct CycleFailure {
    pub round: usize,
    pub step: String,
    pub error: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct StepOutcome {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

pub async fn run(args: CycleTestArgs) -> Result<()> {
    let base = if std::env::var("FOCUSA_DAEMON_URL").is_ok() && args.base_url == DEFAULT_BASE_URL {
        std::env::var("FOCUSA_DAEMON_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
    } else {
        args.base_url.clone()
    };

    let started = std::time::Instant::now();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(args.timeout))
        .build()
        .context("build reqwest client")?;

    // 0. Daemon health gate
    let health = client
        .get(format!("{base}/v1/health"))
        .send()
        .await
        .with_context(|| format!("GET {base}/v1/health"))?;
    if !health.status().is_success() {
        error!(daemon_url = %base, http_status = %health.status(), "daemon health check failed");
        bail!("daemon health check returned HTTP {}", health.status());
    }
    let health_json: serde_json::Value = health.json().await.context("decode health")?;
    if !args.json {
        println!(
            "✓  daemon alive (v{}) at {}",
            health_json
                .get("version")
                .and_then(|x| x.as_str())
                .unwrap_or("?"),
            base
        );
    }

    let mut failures: Vec<CycleFailure> = Vec::new();
    let mut passed = 0usize;

    for round in 1..=args.rounds {
        if !args.json {
            println!("\n=== round {round} of {} ===", args.rounds);
        }
        let mut round_failed = false;
        let mut macro_err = |step: &str, e: String| {
            error!(
                round = round,
                step = %step,
                error = %e,
                "cycle-test round failure"
            );
            failures.push(CycleFailure {
                round,
                step: step.to_string(),
                error: e.clone(),
            });
            if !args.json {
                eprintln!("  ✗ {step}: {e}");
            }
            round_failed = true;
        };

        // Step 1: Create room
        let room = match create_room(&client, &base).await {
            Ok(r) => r,
            Err(e) => {
                macro_err("create_room", e.to_string());
                if args.fail_fast {
                    break;
                }
                continue;
            }
        };
        if !args.json {
            println!("  ✓ room created: {}", short(&room));
        }

        // Step 2: Mac joins
        if let Err(e) = mac_join(&client, &base, &room, &args.mac_name).await {
            macro_err("mac_join", e.to_string());
            if args.fail_fast {
                break;
            }
            continue;
        }
        if !args.json {
            println!("  ✓ mac joined");
        }

        // Step 3: Phone approves
        let device_id = match phone_approve(&client, &base, &room).await {
            Ok(id) => id,
            Err(e) => {
                macro_err("phone_approve", e.to_string());
                if args.fail_fast {
                    break;
                }
                continue;
            }
        };
        if !args.json {
            println!("  ✓ phone approved; device_id={}", short(&device_id));
        }

        // Step 4: Verify completed + token minted
        if let Err(e) = verify_completed(&client, &base, &room).await {
            macro_err("verify_completed", e.to_string());
            if args.fail_fast {
                break;
            }
            continue;
        }
        if !args.json {
            println!("  ✓ status=completed, token present");
        }

        // Step 5: Revoke
        if let Err(e) = revoke(&client, &base, &device_id, &args.host).await {
            macro_err("revoke", e.to_string());
            if args.fail_fast {
                break;
            }
            continue;
        }
        if !args.json {
            println!("  ✓ revoked");
        }

        // Step 6: Idempotent re-revoke
        if let Err(e) = revoke(&client, &base, &device_id, &args.host).await {
            macro_err("idempotent_revoke", e.to_string());
            if args.fail_fast {
                break;
            }
            continue;
        }
        if !args.json {
            println!("  ✓ re-revoke idempotent");
        }

        // Step 7: List reflects revoked
        if let Err(e) = list_shows_revoked(&client, &base, &device_id, &args.host).await {
            macro_err("list_reflects_revoked", e.to_string());
            if args.fail_fast {
                break;
            }
            continue;
        }
        if !args.json {
            println!("  ✓ list shows revoked=true");
        }

        if !round_failed {
            passed += 1;
        }
        if !args.json {
            println!("  round {round}: PASS");
        }
    }

    let report = CycleTestReport {
        rounds_attempted: args.rounds,
        rounds_passed: passed,
        rounds_failed: args.rounds - passed,
        failures: failures.clone(),
        total_duration_ms: started.elapsed().as_millis(),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!();
        println!("=== cycle-test summary ===");
        println!("rounds attempted: {}", report.rounds_attempted);
        println!("rounds passed:    {}", report.rounds_passed);
        println!("rounds failed:    {}", report.rounds_failed);
        println!("duration:         {} ms", report.total_duration_ms);
        if !failures.is_empty() {
            println!();
            println!("failures:");
            for f in &failures {
                println!("  round {}: [{}] {}", f.round, f.step, f.error);
            }
        }
    }

    if report.rounds_failed > 0 {
        error!(
            rounds_attempted = report.rounds_attempted,
            rounds_passed = report.rounds_passed,
            rounds_failed = report.rounds_failed,
            duration_ms = report.total_duration_ms,
            "cycle-test completed with failures"
        );
        std::process::exit(1);
    }
    info!(
        rounds_attempted = report.rounds_attempted,
        rounds_passed = report.rounds_passed,
        duration_ms = report.total_duration_ms,
        "cycle-test all rounds passed"
    );
    Ok(())
}

fn short(s: &str) -> String {
    s.chars().take(8).collect()
}

async fn create_room(client: &reqwest::Client, base: &str) -> Result<String> {
    let resp = client
        .post(format!("{base}/v1/connect/room/create"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .context("POST /v1/connect/room/create")?;
    if !resp.status().is_success() {
        error!(http_status = %resp.status(), "create_room returned non-success");
        bail!("HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await.context("decode create_room response")?;
    let room_id = v
        .get("room_id")
        .and_then(|x| x.as_str())
        .context("missing room_id")?
        .to_string();
    debug!(room_id = %room_id, "room created");
    Ok(room_id)
}

async fn mac_join(
    client: &reqwest::Client,
    base: &str,
    room_id: &str,
    mac_name: &str,
) -> Result<()> {
    let url = format!("{base}/v1/connect/room/{room_id}/join");
    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "mac_name": mac_name,
            "mac_nonce": format!("cycle-{}", room_id),
        }))
        .send()
        .await
        .context("POST /v1/connect/room/{id}/join")?;
    if !resp.status().is_success() {
        error!(room_id = %room_id, mac_name = %mac_name, http_status = %resp.status(), "mac_join returned non-success");
        bail!("HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await.context("decode mac_join response")?;
    let status = v
        .get("status")
        .and_then(|x| x.as_str())
        .context("missing status")?;
    if status != "mac_seen" {
        warn!(room_id = %room_id, mac_name = %mac_name, status = %status, "mac_join unexpected status");
        bail!("expected mac_seen, got {status}");
    }
    debug!(room_id = %room_id, mac_name = %mac_name, "mac joined room");
    Ok(())
}

async fn phone_approve(
    client: &reqwest::Client,
    base: &str,
    room_id: &str,
) -> Result<String> {
    let url = format!("{base}/v1/connect/room/{room_id}/approve");
    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "host": "127.0.0.1",
            "operator_id": "cycle-test",
            "completed_by": "cycle-test",
        }))
        .send()
        .await
        .context("POST /v1/connect/room/{id}/approve")?;
    if !resp.status().is_success() {
        error!(room_id = %room_id, http_status = %resp.status(), "phone_approve returned non-success");
        bail!("HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await.context("decode approve response")?;
    let status = v
        .get("status")
        .and_then(|x| x.as_str())
        .context("missing status")?;
    if status != "completed" {
        warn!(room_id = %room_id, status = %status, "phone_approve unexpected status");
        bail!("expected completed, got {status}");
    }
    let device_id = v
        .get("device_id")
        .and_then(|x| x.as_str())
        .context("missing device_id")?
        .to_string();
    debug!(room_id = %room_id, device_id = %device_id, "phone approved, token minted");
    Ok(device_id)
}

async fn verify_completed(
    client: &reqwest::Client,
    base: &str,
    room_id: &str,
) -> Result<()> {
    let url = format!("{base}/v1/connect/room/{room_id}/status");
    let resp = client.get(&url).send().await.context("GET status")?;
    if !resp.status().is_success() {
        error!(room_id = %room_id, http_status = %resp.status(), "verify_completed status GET returned non-success");
        bail!("HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await.context("decode status")?;
    let status = v
        .get("status")
        .and_then(|x| x.as_str())
        .context("missing status")?;
    if status != "completed" {
        warn!(room_id = %room_id, status = %status, "verify_completed unexpected status");
        bail!("expected completed, got {status}");
    }
    let has_token = v
        .get("token")
        .map(|x| x.is_string() && !x.as_str().unwrap_or("").is_empty())
        .unwrap_or(false);
    if !has_token {
        error!(room_id = %room_id, "token missing or empty after approve");
        bail!("token missing or empty");
    }
    debug!(room_id = %room_id, "verified status=completed + token present");
    Ok(())
}

async fn revoke(
    client: &reqwest::Client,
    base: &str,
    device_id: &str,
    host: &str,
) -> Result<()> {
    let url = format!("{base}/v1/device/pair/revoke");
    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "device_id": device_id,
            "host": host,
            "reason": "cycle-test revoke",
        }))
        .send()
        .await
        .context("POST /v1/device/pair/revoke")?;
    if !resp.status().is_success() {
        error!(device_id = %device_id, host = %host, http_status = %resp.status(), "revoke returned non-success");
        bail!("HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await.context("decode revoke response")?;
    let ledger_appended = v
        .get("ledger_appended")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    let status_completed = v
        .get("status")
        .and_then(|x| x.as_str())
        .map(|s| s == "completed")
        .unwrap_or(false);
    if !ledger_appended && !status_completed {
        bail!(
            "revoke did not confirm revocation: {}",
            serde_json::to_string(&v).unwrap_or_default()
        );
    }
    Ok(())
}

async fn list_shows_revoked(
    client: &reqwest::Client,
    base: &str,
    device_id: &str,
    host: &str,
) -> Result<()> {
    let url = format!("{base}/v1/device/pair/list?host={host}");
    let resp = client.get(&url).send().await.context("GET list")?;
    if !resp.status().is_success() {
        error!(device_id = %device_id, host = %host, http_status = %resp.status(), "pair_list returned non-success");
        bail!("HTTP {}", resp.status());
    }
    let v: serde_json::Value = resp.json().await.context("decode list")?;
    let devices = v
        .get("devices")
        .and_then(|x| x.as_array())
        .context("missing devices array")?;
    let found = devices
        .iter()
        .find(|d| d.get("device_id").and_then(|x| x.as_str()) == Some(device_id))
        .context("device_id not found in list")?;
    let revoked = found
        .get("revoked")
        .and_then(|x| x.as_bool())
        .unwrap_or(false);
    if !revoked {
        warn!(device_id = %device_id, "list entry present but revoked=false");
        bail!("list shows device but revoked=false");
    }
    debug!(device_id = %device_id, "list reflects revoked=true");
    Ok(())
}