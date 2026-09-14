//! Screenshot Evidence Share settings authority.
//!
//! GET/PUT `/api/screenshot/settings` — the canonical effective configuration
//! the menubar Evidence Sharing section (PR #441, apps/menubar Settings.svelte)
//! consumes. The daemon owns the settings; consumers store only connection and
//! UI-local state.
//!
//! Contract (field names are pinned by the consumer contract test below):
//! - GET  200 → `{ "revision": u64, "values": { "enablement": { "auto_screenshot": bool },
//!   "image": { "quality": u32 (40..=100) }, "presentation": { "theme": "system"|"light"|"dark" } } }`
//! - PUT  body `{ "expected_revision": u64, "values": { ...same shape... } }`
//!   - 200 → same shape as GET with the incremented revision
//!   - 409 → `{ "error": ..., "current_revision": u64 }` when `expected_revision` is stale
//!   - 400 → `{ "error": ... }` for malformed or out-of-range values
//!
//! Defaults: `auto_screenshot = false` (evidence capture is opt-in), `quality = 80`,
//! `theme = "system"`. State persists atomically (temp write + rename) across
//! daemon restarts.

use crate::server::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const SETTINGS_ENDPOINT: &str = "/api/screenshot/settings";
pub const MIN_QUALITY: i64 = 40;
pub const MAX_QUALITY: i64 = 100;
pub const THEMES: [&str; 3] = ["system", "light", "dark"];

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
struct Enablement {
    #[serde(default)]
    auto_screenshot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Image {
    #[serde(default = "default_quality")]
    quality: i64,
}

fn default_quality() -> i64 {
    80
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Presentation {
    #[serde(default = "default_theme")]
    theme: String,
}

fn default_theme() -> String {
    "system".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SettingsValues {
    #[serde(default)]
    enablement: Enablement,
    #[serde(default = "default_image")]
    image: Image,
    #[serde(default = "default_presentation")]
    presentation: Presentation,
}

fn default_image() -> Image {
    Image {
        quality: default_quality(),
    }
}

fn default_presentation() -> Presentation {
    Presentation {
        theme: default_theme(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SettingsState {
    revision: u64,
    values: SettingsValues,
}

fn defaults() -> SettingsState {
    SettingsState {
        revision: 1,
        values: SettingsValues {
            enablement: Enablement {
                auto_screenshot: false,
            },
            image: default_image(),
            presentation: default_presentation(),
        },
    }
}

fn settings_path(data_dir: &str) -> PathBuf {
    Path::new(data_dir).join("screenshot-settings.json")
}

fn load_from_disk(path: &Path) -> SettingsState {
    match std::fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|_| defaults()),
        Err(_) => defaults(),
    }
}

fn persist_atomically(path: &Path, state: &SettingsState) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    let raw = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, raw).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

fn validate_values(values: &Value) -> Result<SettingsValues, (StatusCode, Value)> {
    let parsed: SettingsValues = serde_json::from_value(values.clone()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            json!({ "error": format!("malformed settings values: {e}") }),
        )
    })?;
    if !(MIN_QUALITY..=MAX_QUALITY).contains(&parsed.image.quality) {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({ "error": format!("image quality must be between {MIN_QUALITY} and {MAX_QUALITY}") }),
        ));
    }
    if !THEMES.contains(&parsed.presentation.theme.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            json!({ "error": format!("presentation theme must be one of: {}", THEMES.join(", ")) }),
        ));
    }
    Ok(parsed)
}

/// Apply one PUT with expected-revision optimistic concurrency.
/// Returns the new state, or (status, error body).
async fn apply_put(
    dir: &str,
    expected_revision: u64,
    values: Value,
) -> Result<SettingsState, (StatusCode, Value)> {
    let parsed = validate_values(&values)?;
    let _guard = WRITE_LOCK.lock().await;
    let path = settings_path(dir);
    let current = load_from_disk(&path);
    if current.revision != expected_revision {
        return Err((
            StatusCode::CONFLICT,
            json!({
                "error": format!(
                    "expected_revision {expected_revision} does not match current revision {}",
                    current.revision
                ),
                "current_revision": current.revision,
            }),
        ));
    }
    let next = SettingsState {
        revision: current.revision.saturating_add(1),
        values: parsed,
    };
    persist_atomically(&path, &next).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "error": format!("failed to persist settings: {e}") }),
        )
    })?;
    Ok(next)
}

static WRITE_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn get_settings(State(state): State<Arc<AppState>>) -> Json<Value> {
    let state = load_from_disk(&settings_path(&state.config.data_dir));
    Json(serde_json::to_value(state).unwrap_or_else(|_| json!({})))
}

#[derive(Debug, Deserialize)]
struct PutRequest {
    expected_revision: u64,
    values: Value,
}

async fn put_settings(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PutRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match apply_put(&state.config.data_dir, body.expected_revision, body.values).await {
        Ok(next) => Ok(Json(
            serde_json::to_value(next).unwrap_or_else(|_| json!({})),
        )),
        Err((status, err)) => Err((status, Json(err))),
    }
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route(SETTINGS_ENDPOINT, get(get_settings).put(put_settings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_dir() -> (TempDir, String) {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().to_string_lossy().to_string();
        (dir, path)
    }

    fn sample_values(quality: i64, theme: &str, auto: bool) -> Value {
        json!({
            "enablement": { "auto_screenshot": auto },
            "image": { "quality": quality },
            "presentation": { "theme": theme },
        })
    }

    #[tokio::test]
    async fn get_returns_defaults_with_revision_one() {
        let (_dir, path) = temp_dir();
        let state = load_from_disk(&settings_path(&path));
        assert_eq!(state.revision, 1);
        assert!(!state.values.enablement.auto_screenshot);
        assert_eq!(state.values.image.quality, 80);
        assert_eq!(state.values.presentation.theme, "system");
    }

    #[tokio::test]
    async fn put_roundtrip_updates_values_and_bumps_revision() {
        let (_dir, path) = temp_dir();
        let next = apply_put(&path, 1, sample_values(95, "dark", true))
            .await
            .expect("put ok");
        assert_eq!(next.revision, 2);
        assert!(next.values.enablement.auto_screenshot);
        assert_eq!(next.values.image.quality, 95);
        assert_eq!(next.values.presentation.theme, "dark");
        // persisted
        let reloaded = load_from_disk(&settings_path(&path));
        assert_eq!(reloaded, next);
    }

    #[tokio::test]
    async fn stale_expected_revision_conflicts_with_current_revision() {
        let (_dir, path) = temp_dir();
        let _ = apply_put(&path, 1, sample_values(80, "system", false)).await;
        let err = apply_put(&path, 1, sample_values(60, "light", true))
            .await
            .expect_err("stale revision must conflict");
        assert_eq!(err.0, StatusCode::CONFLICT);
        assert_eq!(err.1["current_revision"], 2);
    }

    #[tokio::test]
    async fn out_of_range_quality_and_unknown_theme_fail_closed() {
        let (_dir, path) = temp_dir();
        let bad_quality = apply_put(&path, 1, sample_values(30, "system", false)).await;
        assert_eq!(bad_quality.unwrap_err().0, StatusCode::BAD_REQUEST);
        let bad_theme = apply_put(&path, 1, sample_values(80, "neon", false)).await;
        assert_eq!(bad_theme.unwrap_err().0, StatusCode::BAD_REQUEST);
        // nothing persisted
        let state = load_from_disk(&settings_path(&path));
        assert_eq!(state.revision, 1);
    }

    /// Consumer contract: the merged menubar Settings section (PR #441) reads
    /// exactly these fields from this endpoint. Field drift breaks the UI.
    #[test]
    fn merged_menubar_client_still_binds_this_contract() {
        let client = include_str!("../../../../apps/menubar/src/lib/components/Settings.svelte");
        for token in [
            SETTINGS_ENDPOINT,
            "expected_revision",
            "revision",
            "enablement",
            "auto_screenshot",
            "image",
            "quality",
            "presentation",
            "theme",
        ] {
            assert!(
                client.contains(token),
                "merged menubar client no longer references '{token}' — update the consumer contract"
            );
        }
    }
}
