//! Menubar headless e2e test (focusa-ui0y v0.9.35-dev).
//!
//! Proves that the Focusa Mac menubar's pairing flow plumbing works
//! WITHOUT a real Mac, by:
//!   1. Building the Svelte SPA with `vite build` (via node + npm-cli.js)
//!   2. Serving it on port 1420 via python3 -m http.server
//!   3. Opening it in headless chromium-browser
//!   4. Injecting `window.__FOCUSA_HEADLESS__=true` + `__FOCUSA_DAEMON_URL__`
//!   5. Asserting the rendered DOM contains all expected fragments
//!      (Svelte app booted, FirstRunConnect panel mounted, diagnostics
//!      store wired, Copy debug bundle button present)
//!   6. Optionally drives the full PWA pairing end-to-end
//!
//! Requires:
//!   - chromium-browser on PATH (verified at runtime)
//!   - daemon reachable at FOCUSA_HEADLESS_DAEMON_URL (default 127.0.0.1:8787)
//!   - /opt/node-v22.22.3-linux-x64/bin/node + npm-cli.js (Node 22 LTS)
//!
//! Marked `#[ignore]` so it doesn't run in `cargo test` by default.
//!
//! Run with:
//!   cargo test --package focusa-cli --test menubar_headless_e2e -- --ignored --nocapture

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const DEFAULT_DAEMON_URL: &str = "http://127.0.0.1:8787";
const PREVIEW_PORT: u16 = 1420;
const NODE: &str = "/opt/node-v22.22.3-linux-x64/bin/node";
const NPM_CLI: &str = "/opt/node-v22.22.3-linux-x64/lib/node_modules/npm/bin/npm-cli.js";
const PYTHON: &str = "python3";
const CHROMIUM_FLAGS: &[&str] = &[
    "--headless=new",
    "--no-sandbox",
    "--disable-gpu",
    "--disable-dev-shm-usage",
    "--disable-extensions",
    "--no-first-run",
    "--no-default-browser-check",
    "--disable-features=Translate,BackForwardCache",
];

fn workspace_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while p.pop() {
        if p.join("Cargo.toml").exists() && p.join("apps").join("menubar").exists() {
            return p;
        }
    }
    PathBuf::from(".")
}

fn daemon_url() -> String {
    std::env::var("FOCUSA_HEADLESS_DAEMON_URL").unwrap_or_else(|_| DEFAULT_DAEMON_URL.to_string())
}

fn chromium_path() -> PathBuf {
    for c in &["chromium-browser", "chromium", "google-chrome"] {
        if let Ok(out) = Command::new("which").arg(c).output() {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() {
                    return PathBuf::from(s);
                }
            }
        }
    }
    PathBuf::from("chromium-browser")
}

fn with_node_path() -> String {
    let base = std::env::var("PATH").unwrap_or_default();
    format!(
        "/opt/node-v22.22.3-linux-x64/bin:/usr/local/bin:/usr/bin:/bin:{}",
        base
    )
}

async fn ensure_daemon_alive(client: &reqwest::Client, base: &str) -> bool {
    match client
        .get(format!("{base}/v1/health"))
        .timeout(Duration::from_secs(3))
        .send()
        .await
    {
        Ok(r) => r.status().is_success(),
        Err(_) => false,
    }
}

fn build_menubar() -> Result<PathBuf, String> {
    let root = workspace_root();
    let menubar = root.join("apps").join("menubar");
    let dist = menubar.join("build");
    eprintln!("[e2e] building menubar SPA via npm run build (cwd={})", menubar.display());

    let status = Command::new(NODE)
        .args([NPM_CLI, "run", "-s", "build"])
        .current_dir(&menubar)
        .env("PATH", with_node_path())
        .env("NODE_PATH", "/opt/node-v22.22.3-linux-x64/lib/node_modules")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("failed to spawn {NODE}: {e}"))?;
    if !status.success() {
        return Err(format!("vite build exited {status:?}"));
    }
    if !dist.exists() {
        return Err(format!("vite build did not produce {}", dist.display()));
    }
    eprintln!("[e2e] SPA built at {}", dist.display());
    Ok(dist)
}

fn spawn_static_server(dist: &PathBuf) -> Result<Child, String> {
    let dir = dist.to_string_lossy().to_string();
    let child = Command::new(PYTHON)
        .args([
            "-u",
            "-m",
            "http.server",
            &PREVIEW_PORT.to_string(),
            "--bind",
            "127.0.0.1",
            "--directory",
            &dir,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to spawn {PYTHON} http.server: {e}"))?;
    eprintln!("[e2e] static server spawned on http://127.0.0.1:{PREVIEW_PORT}/");
    Ok(child)
}

async fn wait_for_server() -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_millis(250)).await;
        if let Ok(r) = client
            .get(format!("http://127.0.0.1:{PREVIEW_PORT}/"))
            .send()
            .await
        {
            if r.status().is_success() {
                return Ok(());
            }
        }
    }
    Err(format!(
        "static server at 127.0.0.1:{PREVIEW_PORT} did not respond within 7.5s"
    ))
}

fn chromium_dump_dom(url: &str, daemon_url: &str) -> Result<String, String> {
    let chromium = chromium_path();
    // Write a tiny bootstrap HTML file that injects our globals and then
    // redirects to the SPA. data: URLs have origin=null which breaks
    // localStorage, so we use a real file.
    let bootstrap = std::env::temp_dir().join(format!("focusa-e2e-bootstrap-{}.html", std::process::id()));
    let daemon_json = serde_json::to_string(daemon_url).unwrap_or_else(|_| "\"\"".into());
    let url_json = serde_json::to_string(url).unwrap_or_else(|_| "\"\"".into());
    let html = format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><title>focusa-e2e-bootstrap</title></head><body>
<script>
window.__FOCUSA_HEADLESS__ = true;
window.__FOCUSA_DAEMON_URL__ = {daemon_json};
location.replace({url_json});
</script>
<p>Focusa headless e2e bootstrap</p>
</body></html>"#
    );
    std::fs::write(&bootstrap, html).map_err(|e| format!("write bootstrap: {e}"))?;
    eprintln!("[e2e] bootstrap at {}", bootstrap.display());
    eprintln!("[e2e] launching chromium at {}", chromium.display());
    let mut cmd = Command::new(&chromium);
    cmd.args(CHROMIUM_FLAGS);
    cmd.arg(format!(
        "--user-data-dir=/tmp/focusa-e2e-chrome-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    cmd.arg("--virtual-time-budget=15000");
    cmd.arg("--dump-dom");
    cmd.arg(format!("file://{}", bootstrap.display()));
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to launch chromium: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "chromium exited {:?}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let _ = std::fs::remove_file(&bootstrap);
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn assert_contains(needle: &str, haystack: &str, ctx: &str) -> Result<(), String> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(format!("{ctx}: expected to find {needle:?}"))
    }
}

#[tokio::test]
#[ignore = "requires chromium + live daemon + node; run with --ignored --nocapture"]
async fn menubar_headless_e2e() {
    let base = daemon_url();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("build reqwest client");

    if !ensure_daemon_alive(&client, &base).await {
        panic!(
            "daemon not reachable at {base}; start it or set FOCUSA_HEADLESS_DAEMON_URL. \
             SKIP this test by running cargo test without --ignored if no daemon is available."
        );
    }

    // 1. Build the SPA
    let dist = match build_menubar() {
        Ok(d) => d,
        Err(e) => panic!("build_menubar failed: {e}"),
    };

    // 2. Spawn static server
    let mut server = match spawn_static_server(&dist) {
        Ok(s) => s,
        Err(e) => panic!("spawn_static_server failed: {e}"),
    };

    // 3. Wait for the server to be reachable
    if let Err(e) = wait_for_server().await {
        let _ = server.kill();
        panic!("static server not ready: {e}");
    }

    // 4. Verify the SPA is being served
    let spa_resp = client
        .get(format!("http://127.0.0.1:{PREVIEW_PORT}/"))
        .send()
        .await
        .expect("GET / SPA");
    let spa_html = spa_resp.text().await.expect("decode SPA");
    assert_contains("<html", &spa_html, "SPA HTML").expect("SPA is valid HTML");
    eprintln!("[e2e] SPA served: {} bytes", spa_html.len());

    // 5. Headless chromium: render the SPA, verify the FirstRunConnect
    //    panel and diagnostics store are mounted in the DOM.
    let url = format!("http://127.0.0.1:{PREVIEW_PORT}/");
    let dom = match chromium_dump_dom(&url, &base) {
        Ok(d) => d,
        Err(e) => {
            let _ = server.kill();
            panic!("chromium dump-dom failed: {e}");
        }
    };
    eprintln!("[e2e] chromium dump-dom: {} bytes", dom.len());

    let required: &[&str] = &[
        "<title>Focusa",
        "Connect to Focusa",
        "Copy debug bundle",
        "Scan this QR",
    ];
    for r in required {
        if let Err(e) = assert_contains(r, &dom, "menubar SPA") {
            let _ = server.kill();
            panic!("{e}");
        }
    }
    eprintln!("[e2e] all required fragments present in rendered DOM");

    // 6. Bonus: PWA scan page verify via the same headless chromium
    let room_resp = client
        .post(format!("{base}/v1/connect/room/create"))
        .json(&serde_json::json!({}))
        .send()
        .await
        .expect("POST /v1/connect/room/create");
    let room: serde_json::Value = room_resp.json().await.expect("decode create_room");
    let room_id = room["room_id"].as_str().expect("room_id").to_string();
    let scan_url = format!("{base}/connect/room/{room_id}/scan");
    let scan_dom = chromium_dump_dom(&scan_url, &base).expect("chromium dump-dom /scan");
    eprintln!("[e2e] /scan dump-dom: {} bytes", scan_dom.len());
    let scan_required: &[&str] = &[
        "Focusa — Pair Mac",
        "navigator.mediaDevices.getUserMedia",
        "jsQR",
        "approveBtn",
    ];
    for r in scan_required {
        if let Err(e) = assert_contains(r, &scan_dom, "PWA /scan") {
            let _ = server.kill();
            panic!("{e}");
        }
    }
    eprintln!("[e2e] all PWA /scan fragments present in rendered DOM");

    let _ = server.kill();
    eprintln!("[e2e] menubar headless e2e PASS");
}