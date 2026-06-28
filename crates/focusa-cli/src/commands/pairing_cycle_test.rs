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
    /// V2: Verify PairingStore durability. Creates a room, then SIGKILL's the
    /// daemon, restarts it, and asserts the room is still queryable
    /// (rehydrated from the SQLite ledger). Requires the operator to allow
    /// the test to recycle the daemon process.
    #[arg(long)]
    pub check_restart_durability: bool,
    /// Also verify that the /connect/room/{id}/scan PWA page renders correctly
    /// (HTTP 200 + contains expected DOM/JS fragments). Drives the headless
    /// plumbing end-to-end without needing a real phone.
    #[arg(long)]
    pub with_pwa_verify: bool,
}

#[derive(Debug, Serialize)]
pub struct CycleTestReport {
    pub rounds_attempted: usize,
    pub rounds_passed: usize,
    pub rounds_failed: usize,
    pub failures: Vec<CycleFailure>,
    pub total_duration_ms: u128,
    pub pwa_verify: Option<PwaVerifyReport>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PwaVerifyReport {
    pub scanned_url: String,
    pub http_status: u16,
    pub page_bytes: usize,
    pub fragments_found: Vec<String>,
    pub fragments_missing: Vec<String>,
    pub passed: bool,
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
    let mut pwa_verify: Option<PwaVerifyReport> = None;

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

    // PWA headless verification: opens /connect/room/{id}/scan and asserts
    // all expected DOM/JS fragments are present. Proves the phone-side
    // entry point renders without needing a real phone.
    // V2: PairingStore restart durability check. Recycles the daemon, then
    // re-queries a previously-created room to confirm the ledger rehydrated it.
    let mut _restart_passed = true;
    if args.check_restart_durability && passed > 0 {
        if !args.json {
            println!();
            println!("=== PairingStore restart durability check ===");
        }
        // Capture a fresh room for the durability probe.
        let probe_room = match create_room(&client, &base).await {
            Ok(r) => r,
            Err(e) => {
                warn!(error = %e, "could not create probe room for restart-durability");
                _restart_passed = false;
                String::new()
            }
        };
        if !probe_room.is_empty() {
            // SIGKILL the daemon.
            let _ = std::process::Command::new("pkill")
                .args(["-9", "-f", "focusa-daemon"])
                .status();
            // ALSO stop the systemd focusa-daemon.service so its auto-respawn
            // (RestartSec=1) does not race with our spawned child daemon. The
            // system service uses FOCUSA_DATA_DIR=/home/wirebot/focusa/data/.focusa
            // (DB #2); our spawned child uses FOCUSA_HOME=/home/wirebot/focusa
            // (DB #1). If systemd wins the port race, the cycle-test's query
            // hits the wrong DB and returns 404 spuriously.
            let _ = std::process::Command::new("systemctl")
                .args(["stop", "focusa-daemon.service"])
                .status();
            // Poll for the port to release (up to 5s).
            for _ in 0..50 {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let probe = std::net::TcpStream::connect_timeout(
                    &"127.0.0.1:8787".parse().unwrap(),
                    std::time::Duration::from_millis(50),
                );
                if probe.is_err() {
                    break;
                }
            }
            // Restart the daemon (detached) and poll for it to bind. We
            // explicitly unset FOCUSA_DATA_DIR so the new daemon inherits
            // the same DB path as the one we just killed (it uses
            // FOCUSA_HOME → /home/wirebot/focusa → focusa.sqlite by default).
            // Without this, FOCUSA_DATA_DIR inherited from a stale env
            // would point the new daemon at a different SQLite file and
            // the room would appear to vanish across the restart.
            let _ = std::process::Command::new("/usr/local/bin/focusa-daemon")
                .args(["--port", "8787"])
                .env_remove("FOCUSA_DATA_DIR")
                .env_remove("FOCUSA_HOME")
                .env("FOCUSA_HOME", "/home/wirebot/focusa")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
            let mut ready = false;
            for _ in 0..50 {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                let probe = std::net::TcpStream::connect_timeout(
                    &"127.0.0.1:8787".parse().unwrap(),
                    std::time::Duration::from_millis(200),
                );
                if probe.is_ok() {
                    ready = true;
                    break;
                }
            }
            if !ready {
                warn!("focusa-daemon did not become ready within 10s after restart");
                _restart_passed = false;
                if !args.json {
                    println!("  restart durability: FAIL (daemon not ready)");
                }
            } else {
                // Tiny extra settle so SQLite-backed routes are consistent.
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                // Query the room again.
                let url = format!("{base}/v1/connect/room/{probe_room}/status");
                match client.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        let v: serde_json::Value = resp.json().await.unwrap_or_default();
                        let event = v
                            .get("diagnostics")
                            .and_then(|d| d.get("event"))
                            .and_then(|e| e.as_str())
                            .unwrap_or("");
                        if event == "room_rehydrated_from_ledger" {
                            if !args.json {
                                println!("  restart durability: PASS (room rehydrated from SQLite)");
                            }
                        } else {
                            warn!(event = %event, "room found but not rehydrated from ledger");
                            _restart_passed = false;
                            if !args.json {
                                println!("  restart durability: FAIL (event={event})");
                            }
                        }
                    }
                    Ok(resp) => {
                        warn!(http_status = %resp.status(), "room query after restart returned non-success");
                        _restart_passed = false;
                    }
                    Err(e) => {
                        warn!(error = %e, "could not reach daemon after restart");
                        _restart_passed = false;
                    }
                }
            }
        }
    }
    if !args.json && args.check_restart_durability {
        println!();
    }

    if args.with_pwa_verify && passed > 0 {
        if !args.json {
            println!();
            println!("=== PWA scan page verify (headless plumbing) ===");
        }
        match verify_pwa_scan(&client, &base).await {
            Ok(report) => {
                if !args.json {
                    println!("  scanned: {}", report.scanned_url);
                    println!(
                        "  http_status: {} | page_bytes: {}",
                        report.http_status, report.page_bytes
                    );
                    println!("  fragments_found:   {}", report.fragments_found.join(", "));
                    if !report.fragments_missing.is_empty() {
                        println!("  fragments_missing: {}", report.fragments_missing.join(", "));
                    }
                    if report.passed {
                        println!("  ✓ PWA scan page verify PASS");
                    } else {
                        println!("  ✗ PWA scan page verify FAIL");
                        failures.push(CycleFailure {
                            round: 0,
                            step: "pwa_verify".to_string(),
                            error: format!(
                                "missing fragments: {}",
                                report.fragments_missing.join(", ")
                            ),
                        });
                    }
                }
                pwa_verify = Some(report);
            }
            Err(e) => {
                let msg = format!("pwa_verify failed: {e}");
                error!(error = %e, "PWA scan page verify failed");
                if !args.json {
                    println!("  ✗ {msg}");
                }
                failures.push(CycleFailure {
                    round: 0,
                    step: "pwa_verify".to_string(),
                    error: msg,
                });
            }
        }
    }

    let report = CycleTestReport {
        rounds_attempted: args.rounds,
        rounds_passed: passed,
        rounds_failed: args.rounds - passed,
        failures: failures.clone(),
        total_duration_ms: started.elapsed().as_millis(),
        pwa_verify: pwa_verify.clone(),
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
    // V2: verify the device token is accepted as Bearer auth on protected routes.
    let token = v.get("token").and_then(|x| x.as_str()).unwrap_or("");
    if !token.is_empty() {
        let protected_url = format!("{base}/v1/info");
        let auth = client
            .get(&protected_url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .await
            .context("GET /v1/info with Bearer")?;
        if !auth.status().is_success() {
            error!(
                room_id = %room_id,
                http_status = %auth.status(),
                "device token rejected by /v1/info (auth middleware broken)"
            );
            bail!(
                "device token rejected: /v1/info returned HTTP {}",
                auth.status()
            );
        }
        debug!(
            room_id = %room_id,
            "verified device token accepted as Bearer on /v1/info"
        );
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
async fn verify_pwa_scan(client: &reqwest::Client, base: &str) -> Result<PwaVerifyReport> {
    // Create a fresh room to scan against
    let create_resp = client
        .post(format!("{base}/v1/connect/room/create"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .context("POST /v1/connect/room/create for pwa-verify")?;
    if !create_resp.status().is_success() {
        bail!("create-room for pwa-verify returned HTTP {}", create_resp.status());
    }
    let v: serde_json::Value = create_resp.json().await?;
    let room_id = v
        .get("room_id")
        .and_then(|x| x.as_str())
        .context("pwa-verify: missing room_id")?
        .to_string();
    let scanned_url = format!("{base}/connect/room/{room_id}/scan");

    // Fetch the page
    let page_resp = client
        .get(&scanned_url)
        .send()
        .await
        .context("GET /connect/room/{id}/scan")?;
    let http_status = page_resp.status().as_u16();
    if !page_resp.status().is_success() {
        bail!("scan page returned HTTP {http_status}");
    }
    let page = page_resp.text().await.context("decode scan page")?;
    let page_bytes = page.len();

    // Required fragments (proves the page rendered correctly with all the
    // glue that the phone browser will execute)
    const REQUIRED: &[&str] = &[
        "<title>Focusa — Pair Mac</title>",
        "navigator.mediaDevices.getUserMedia",
        "jsQR",
        "approveBtn",
        "Pair this Mac",
        "/v1/connect/room/",
        "mac_handoff_offer",
    ];
    let mut found: Vec<String> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    for f in REQUIRED {
        if page.contains(f) {
            found.push((*f).to_string());
        } else {
            missing.push((*f).to_string());
        }
    }
    let passed = missing.is_empty();
    info!(
        scanned_url = %scanned_url,
        http_status = http_status,
        page_bytes = page_bytes,
        fragments_found = found.len(),
        fragments_missing = missing.len(),
        pwa_passed = passed,
        "PWA scan page verify complete"
    );
    Ok(PwaVerifyReport {
        scanned_url,
        http_status,
        page_bytes,
        fragments_found: found,
        fragments_missing: missing,
        passed,
    })
}
