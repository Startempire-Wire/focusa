//! Revoke + re-pair cycle integration test (focusa-ui0y v0.9.39-dev).
//!
//! Runs the full pairing cycle N times against a live daemon. Requires a
//! Focusa daemon reachable at FOCUSA_CYCLE_TEST_URL (default
//! http://127.0.0.1:8787). Marked `#[ignore]` by default so it does not
//! run in `cargo test` unless the daemon is reachable.
//!
//! Run with:
//!   cargo test --package focusa-cli --test revoke_repair_cycle -- --ignored --nocapture
//!
//! Or via the operator-facing CLI (preferred for self-host):
//!   focusa pairing cycle-test --rounds 10

use std::time::Duration;

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8787";
const ROUNDS: usize = 3;

fn base_url() -> String {
    std::env::var("FOCUSA_CYCLE_TEST_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
}

async fn daemon_alive(client: &reqwest::Client, base: &str) -> bool {
    match client
        .get(format!("{base}/v1/health"))
        .timeout(Duration::from_secs(2))
        .send()
        .await
    {
        Ok(r) => r.status().is_success(),
        Err(_) => false,
    }
}

#[tokio::test]
#[ignore = "requires live daemon; run with --ignored --nocapture"]
async fn revoke_repair_cycle_three_rounds() {
    let base = base_url();
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => panic!("build reqwest client: {e}"),
    };

    assert!(
        daemon_alive(&client, &base).await,
        "daemon not reachable at {base}; set FOCUSA_CYCLE_TEST_URL or start daemon"
    );

    let failures = 0usize;
    for round in 1..=ROUNDS {
        eprintln!("[cycle] round {round}/{ROUNDS}");

        // 1. Create room
        let room_id = match client
            .post(format!("{base}/v1/connect/room/create"))
            .json(&serde_json::json!({}))
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => {
                let v: serde_json::Value = r.json().await.unwrap();
                v["room_id"].as_str().unwrap().to_string()
            }
            Ok(r) => panic!("create_room HTTP {}", r.status()),
            Err(e) => panic!("create_room: {e}"),
        };
        eprintln!("  ✓ room created {room_id}");

        // 2. Mac joins
        let j = client
            .post(format!("{base}/v1/connect/room/{room_id}/join"))
            .json(&serde_json::json!({
                "mac_name": "rust-test-mac",
                "mac_nonce": format!("rust-{}", room_id),
            }))
            .send()
            .await
            .expect("mac_join send");
        assert!(j.status().is_success(), "mac_join HTTP {}", j.status());
        eprintln!("  ✓ mac joined");

        // 3. Phone approves
        let a = client
            .post(format!("{base}/v1/connect/room/{room_id}/approve"))
            .json(&serde_json::json!({
                "host": "127.0.0.1",
                "operator_id": "rust-test",
                "completed_by": "rust-test",
            }))
            .send()
            .await
            .expect("approve send");
        assert!(a.status().is_success(), "approve HTTP {}", a.status());
        let body: serde_json::Value = a.json().await.unwrap();
        let device_id = body["device_id"].as_str().unwrap().to_string();
        assert_eq!(body["status"], "completed", "status != completed");
        eprintln!("  ✓ phone approved; device_id={device_id}");

        // 4. Verify completed + token
        let s = client
            .get(format!("{base}/v1/connect/room/{room_id}/status"))
            .send()
            .await
            .expect("status send");
        let body: serde_json::Value = s.json().await.unwrap();
        assert_eq!(body["status"], "completed", "status != completed");
        assert!(
            body["token"]
                .as_str()
                .map(|s| !s.is_empty())
                .unwrap_or(false)
        );
        eprintln!("  ✓ status=completed, token present");

        // 5. Revoke
        let r = client
            .post(format!("{base}/v1/device/pair/revoke"))
            .json(&serde_json::json!({
                "device_id": device_id,
                "host": "rust-test",
                "reason": "rust cycle test",
            }))
            .send()
            .await
            .expect("revoke send");
        assert!(r.status().is_success(), "revoke HTTP {}", r.status());
        let body: serde_json::Value = r.json().await.unwrap();
        let ledger_appended = body["ledger_appended"].as_bool().unwrap_or(false);
        let status_completed = body["status"].as_str() == Some("completed");
        assert!(
            ledger_appended || status_completed,
            "revoke did not confirm: {body}"
        );
        eprintln!("  ✓ revoked");

        // 6. Idempotent re-revoke
        let r2 = client
            .post(format!("{base}/v1/device/pair/revoke"))
            .json(&serde_json::json!({
                "device_id": device_id,
                "host": "rust-test",
            }))
            .send()
            .await
            .expect("re-revoke send");
        assert!(r2.status().is_success(), "re-revoke HTTP {}", r2.status());
        let body: serde_json::Value = r2.json().await.unwrap();
        let ledger_appended = body["ledger_appended"].as_bool().unwrap_or(false);
        let status_completed = body["status"].as_str() == Some("completed");
        assert!(
            ledger_appended || status_completed,
            "idempotent revoke did not confirm: {body}"
        );
        eprintln!("  ✓ re-revoke idempotent");

        // 7. List reflects revoked
        let l = client
            .get(format!("{base}/v1/device/pair/list?host=rust-test"))
            .send()
            .await
            .expect("list send");
        let body: serde_json::Value = l.json().await.unwrap();
        let devices = body["devices"].as_array().expect("devices array");
        let entry = devices
            .iter()
            .find(|d| d["device_id"].as_str() == Some(&device_id))
            .expect("device_id not in list");
        assert_eq!(entry["revoked"], true, "list entry revoked != true");
        eprintln!("  ✓ list shows revoked=true");

        eprintln!("[cycle] round {round}/{ROUNDS}: PASS");
    }

    assert_eq!(failures, 0, "{failures} cycles failed");
}
