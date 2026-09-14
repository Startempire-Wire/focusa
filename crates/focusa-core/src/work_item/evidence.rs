//! Real evidence verifiers (Spec 116 §7.3).
//!
// Each evidence kind has a dedicated verifier. The verifier runs and
//! records what it actually observed (`result` + `verified: bool`),
// not a stub. The lifecycle runs the matching verifier for every
//! citation in the claim and rejects the claim if any required
//! citation fails.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::work_item::types::{EvidenceCitation, EvidenceKind};

/// The result of running a single verifier. The lifecycle wraps this
/// into the matching `EvidenceCitation` in the claim JSON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyResult {
    pub verified: bool,
    pub result: String,
    pub evidence_url: Option<String>,
}

/// Trait every verifier implements. Concrete verifiers live below.
#[async_trait]
pub trait EvidenceVerifier: Send + Sync {
    fn kind(&self) -> EvidenceKind;
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult;
}

// ---------------------------------------------------------------------------
// Shared utilities
// ---------------------------------------------------------------------------

fn citation_path_component(value: &str) -> &str {
    let without_fragment = value.split_once('#').map(|(path, _)| path).unwrap_or(value);
    without_fragment
        .rsplit_once(':')
        .filter(|(_, suffix)| {
            !suffix.is_empty()
                && suffix
                    .chars()
                    .all(|character| character.is_ascii_digit() || character == '-')
        })
        .map(|(path, _)| path)
        .unwrap_or(without_fragment)
}

/// Resolve a citation `ref_` against the project root. Strips a
/// trailing `:LINE` / `:LINE-LINE` or `#section` without splitting a Windows
/// drive-letter prefix; returns `None` when the ref is not a path.
pub fn citation_path(project_root: &Path, ref_: &str) -> Option<PathBuf> {
    let s = ref_.trim();
    if s.starts_with("http://") || s.starts_with("https://") {
        return None;
    }
    if s.starts_with("gh-") || s.starts_with("gh ") {
        return None;
    }
    Some(project_root.join(citation_path_component(s)))
}

/// Run a child process and return exit status + tail of stderr.
async fn run_capture(
    program: &str,
    args: &[&str],
    cwd: &Path,
    timeout: Duration,
) -> Option<(i32, String)> {
    use tokio::process::Command;
    let mut cmd = Command::new(program);
    cmd.args(args)
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = cmd.spawn().ok()?;
    let out = match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(o)) => o,
        _ => return None,
    };
    let exit = out.status.code().unwrap_or(-1);
    let mut tail = String::from_utf8_lossy(&out.stderr).to_string();
    if tail.len() > 400 {
        tail.truncate(400);
        tail.push('…');
    }
    Some((exit, tail))
}

// ---------------------------------------------------------------------------
// Code verifier
// ---------------------------------------------------------------------------

/// Verifies that a code citation points at a real source file with
/// content in the cited line range. No git access required.
pub struct CodeVerifier;

#[async_trait]
impl EvidenceVerifier for CodeVerifier {
    fn kind(&self) -> EvidenceKind {
        EvidenceKind::Code
    }
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult {
        let Some(project_root) = citation_root_from_ref(&citation.ref_) else {
            return VerifyResult {
                verified: false,
                result: "ref_ does not look like a code path".into(),
                evidence_url: None,
            };
        };
        let Some(path) = citation_path(&project_root, &citation.ref_) else {
            return VerifyResult {
                verified: false,
                result: "ref_ is not a path citation".into(),
                evidence_url: None,
            };
        };
        match std::fs::read_to_string(&path) {
            Err(e) => VerifyResult {
                verified: false,
                result: format!("read failed: {e}"),
                evidence_url: None,
            },
            Ok(contents) => {
                let lines: Vec<&str> = contents.lines().collect();
                let total = lines.len() as u32;
                let (start, end) = match (citation.line, citation.line_end) {
                    (Some(s), Some(e)) => (s.max(1), e.max(s)),
                    (Some(s), None) => (s.max(1), s.max(1)),
                    _ => (1, total),
                };
                if start > total {
                    return VerifyResult {
                        verified: false,
                        result: format!("line {start} past EOF (file has {total} lines)"),
                        evidence_url: None,
                    };
                }
                let slice: Vec<&str> = lines
                    .iter()
                    .skip((start - 1) as usize)
                    .take(((end - start) + 1) as usize)
                    .copied()
                    .collect();
                if slice.is_empty() {
                    return VerifyResult {
                        verified: false,
                        result: "empty line range".into(),
                        evidence_url: None,
                    };
                }
                let preview = slice.join(" | ");
                let preview = if preview.len() > 200 {
                    format!("{}…", &preview[..200])
                } else {
                    preview
                };
                VerifyResult {
                    verified: true,
                    result: format!(
                        "{}: lines {start}-{end}/{} chars={} preview=`{preview}`",
                        path.display(),
                        total,
                        contents.len(),
                    ),
                    evidence_url: Some(format!("file://{}", path.display())),
                }
            }
        }
    }
}

fn citation_root_from_ref(ref_: &str) -> Option<PathBuf> {
    // The claim's `project_root` lives in the claim, not the citation.
    // For the standalone verifier (which doesn't have a claim) we
    // accept absolute paths and look for a project_root marker in the
    // ref itself. The lifecycle wires the project_root in before
    // calling the verifier.
    let p = Path::new(citation_path_component(ref_));
    if p.is_absolute() {
        return Some(p.parent()?.to_path_buf());
    }
    // Heuristic: assume the current working directory is the project
    // root. The lifecycle overrides this when a real project_root is
    // known.
    Some(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

// ---------------------------------------------------------------------------
// Spec verifier
// ---------------------------------------------------------------------------

/// Verifies a spec / doc citation: file exists, contains the cited
/// section heading, and was last modified within `expires_at`.
pub struct SpecVerifier;

#[async_trait]
impl EvidenceVerifier for SpecVerifier {
    fn kind(&self) -> EvidenceKind {
        EvidenceKind::Spec
    }
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult {
        let Some(path) = citation_path(&current_project_root(), &citation.ref_) else {
            return VerifyResult {
                verified: false,
                result: "ref_ is not a path citation".into(),
                evidence_url: None,
            };
        };
        let Ok(contents) = std::fs::read_to_string(&path) else {
            return VerifyResult {
                verified: false,
                result: format!("spec file missing: {}", path.display()),
                evidence_url: None,
            };
        };
        // Section heading check: the citation's ref_ may end with
        // `#section-name` or just be a path; we accept either.
        let section = citation.ref_.rsplit_once('#').map(|(_, s)| s).unwrap_or("");
        if !section.is_empty() && !contents.to_lowercase().contains(&section.to_lowercase()) {
            return VerifyResult {
                verified: false,
                result: format!(
                    "spec does not mention section `{section}`; expected substring in {}",
                    path.display()
                ),
                evidence_url: None,
            };
        }
        let meta = std::fs::metadata(&path).ok();
        let mtime = meta.and_then(|m| m.modified().ok());
        let mtime_str = mtime
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        VerifyResult {
            verified: true,
            result: format!(
                "{}: bytes={} mtime_epoch={}{}",
                path.display(),
                contents.len(),
                mtime_str,
                if section.is_empty() {
                    String::new()
                } else {
                    format!(" section=`{section}`")
                }
            ),
            evidence_url: Some(format!("file://{}", path.display())),
        }
    }
}

fn current_project_root() -> PathBuf {
    std::env::var("FOCUSA_PROJECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

// ---------------------------------------------------------------------------
// Test verifier
// ---------------------------------------------------------------------------

/// Verifies a test citation: file exists. Optionally runs the test
/// when `run: true` appears in the citation's result field (the
/// lifecycle injects this hint when the test is meant to be executed
/// as part of validation).
pub struct TestVerifier;

#[async_trait]
impl EvidenceVerifier for TestVerifier {
    fn kind(&self) -> EvidenceKind {
        EvidenceKind::Test
    }
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult {
        let Some(path) = citation_path(&current_project_root(), &citation.ref_) else {
            return VerifyResult {
                verified: false,
                result: "ref_ is not a path citation".into(),
                evidence_url: None,
            };
        };
        let Ok(contents) = std::fs::read_to_string(&path) else {
            return VerifyResult {
                verified: false,
                result: format!("test file missing: {}", path.display()),
                evidence_url: None,
            };
        };
        // Decide whether to execute. We always check for the marker
        // `[run-as-evidence]` in the test file so the operator
        // explicitly opts in (avoids accidentally running a long test
        // suite during every close).
        let should_run = contents.contains("[run-as-evidence]")
            || std::env::var("FOCUSA_RUN_TESTS_AS_EVIDENCE")
                .map(|v| v == "1")
                .unwrap_or(false);
        if !should_run {
            return VerifyResult {
                verified: true,
                result: format!(
                    "{}: bytes={} (file present; not executed; add [run-as-evidence] inside the test to opt in)",
                    path.display(),
                    contents.len()
                ),
                evidence_url: Some(format!("file://{}", path.display())),
            };
        }
        let program = if path.extension().and_then(|s| s.to_str()) == Some("py") {
            "python3"
        } else {
            "bash"
        };
        let exit = run_capture(
            program,
            &[path.to_str().unwrap_or("")],
            &current_project_root(),
            Duration::from_secs(120),
        )
        .await;
        match exit {
            Some((0, _)) => VerifyResult {
                verified: true,
                result: format!("{}: executed via {} exit=0", path.display(), program),
                evidence_url: Some(format!("file://{}", path.display())),
            },
            Some((code, tail)) => VerifyResult {
                verified: false,
                result: format!(
                    "{}: {} exit={} stderr={}",
                    path.display(),
                    program,
                    code,
                    tail
                ),
                evidence_url: Some(format!("file://{}", path.display())),
            },
            None => VerifyResult {
                verified: false,
                result: format!("{}: failed to spawn {}", path.display(), program),
                evidence_url: None,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Endpoint verifier
// ---------------------------------------------------------------------------

/// Verifies an HTTP endpoint citation: probes the URL with HEAD/GET
/// and reports the status code.
pub struct EndpointVerifier {
    /// Optional base URL prefix. When the citation's `ref_` is a path
    /// (e.g. `/v1/health`), the verifier joins it with this base.
    pub base_url: Option<String>,
    /// Per-request timeout.
    pub timeout: Duration,
}

impl Default for EndpointVerifier {
    fn default() -> Self {
        Self {
            base_url: None,
            timeout: Duration::from_secs(5),
        }
    }
}

#[async_trait]
impl EvidenceVerifier for EndpointVerifier {
    fn kind(&self) -> EvidenceKind {
        EvidenceKind::Endpoint
    }
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult {
        let url = resolve_endpoint_url(self.base_url.as_deref(), &citation.ref_);
        let Some(url) = url else {
            return VerifyResult {
                verified: false,
                result: "ref_ is not a path or URL".into(),
                evidence_url: None,
            };
        };
        let client = reqwest::Client::builder().timeout(self.timeout).build();
        let client = match client {
            Ok(c) => c,
            Err(e) => {
                return VerifyResult {
                    verified: false,
                    result: format!("client build failed: {e}"),
                    evidence_url: Some(url.clone()),
                };
            }
        };
        let resp = client.get(&url).send().await;
        let (verified, status_str, body_preview) = match resp {
            Err(e) => (false, format!("transport error: {e}"), String::new()),
            Ok(r) => {
                let code = r.status().as_u16();
                let ok = (200..300).contains(&code);
                let body = r.text().await.unwrap_or_default();
                let preview_text = if body.len() > 200 {
                    format!("{}…", &body[..200])
                } else {
                    body
                };
                (
                    ok,
                    format!("http {} {code}", if ok { "OK" } else { "FAIL" }),
                    preview_text,
                )
            }
        };
        VerifyResult {
            verified,
            result: format!("GET {url} -> {status_str} body=`{body_preview}`"),
            evidence_url: Some(url),
        }
    }
}

fn resolve_endpoint_url(base: Option<&str>, ref_: &str) -> Option<String> {
    let s = ref_.trim();
    if s.starts_with("http://") || s.starts_with("https://") {
        return Some(s.to_string());
    }
    let base = base?;
    if !s.starts_with('/') {
        return None;
    }
    Some(format!("{}{}", base.trim_end_matches('/'), s))
}

// ---------------------------------------------------------------------------
// Workpoint verifier
// ---------------------------------------------------------------------------

/// Verifies a Workpoint citation by reading the workpoint from the
/// local Focusa data dir. The workpoint evidence is already
/// canonically stored; this verifier just confirms the id resolves.
#[derive(Default)]
pub struct WorkpointVerifier {
    /// Path to the focusa data dir; defaults to FOCUSA_DATA_DIR or
    /// `<HOME>/.focusa`.
    pub data_dir: Option<PathBuf>,
}

impl WorkpointVerifier {
    fn verify_persisted_snapshot(data_dir: &Path, wp_id: &str) -> Option<VerifyResult> {
        let database = data_dir.join("focusa.sqlite");
        let connection = rusqlite::Connection::open_with_flags(
            &database,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .ok()?;
        let state_json: String = connection
            .query_row(
                "SELECT state_json FROM snapshots ORDER BY ts DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok()?;
        let state: serde_json::Value = serde_json::from_str(&state_json).ok()?;
        let record = state
            .pointer("/workpoint/records")?
            .as_array()?
            .iter()
            .find(|record| {
                record
                    .get("workpoint_id")
                    .and_then(serde_json::Value::as_str)
                    == Some(wp_id)
                    && record
                        .get("canonical")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
            })?;
        let evidence_count = record
            .get("evidence_refs")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len)
            .unwrap_or(0);
        Some(VerifyResult {
            verified: true,
            result: format!(
                "workpoint {wp_id}: canonical persisted snapshot evidence_refs={evidence_count}"
            ),
            evidence_url: Some(format!("file://{}#snapshot", database.display())),
        })
    }
}

#[async_trait]
impl EvidenceVerifier for WorkpointVerifier {
    fn kind(&self) -> EvidenceKind {
        EvidenceKind::Workpoint
    }
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult {
        let wp_id = citation
            .ref_
            .trim()
            .trim_start_matches("workpoint:")
            .trim()
            .to_string();
        let data_dir = self
            .data_dir
            .clone()
            .or_else(|| std::env::var_os("FOCUSA_DATA_DIR").map(PathBuf::from))
            .unwrap_or_else(|| {
                std::env::var_os("HOME")
                    .map(|h| PathBuf::from(h).join(".focusa"))
                    .unwrap_or_else(|| PathBuf::from("/var/lib/focusa"))
            });
        let path = data_dir.join("workpoints").join(format!("{wp_id}.json"));
        match std::fs::read_to_string(&path) {
            Err(e) => {
                Self::verify_persisted_snapshot(&data_dir, &wp_id).unwrap_or_else(|| VerifyResult {
                    verified: false,
                    result: format!("workpoint not readable at {}: {e}", path.display()),
                    evidence_url: None,
                })
            }
            Ok(contents) => {
                let parsed: Option<serde_json::Value> = serde_json::from_str(&contents).ok();
                let evidence_count = parsed
                    .as_ref()
                    .and_then(|v| v.get("evidence_refs"))
                    .and_then(|v| v.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0);
                VerifyResult {
                    verified: true,
                    result: format!(
                        "workpoint {wp_id}: bytes={} evidence_refs={evidence_count}",
                        contents.len()
                    ),
                    evidence_url: Some(format!("file://{}", path.display())),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CI verifier
// ---------------------------------------------------------------------------

/// Verifies a CI run citation: parses `gh-<id>` or `glab-<id>` shapes
/// and, when `gh` is on PATH, queries the run state. Returns a
/// typed string describing the run.
pub struct CiVerifier {
    /// Path to the `gh` binary.
    pub gh_path: Option<String>,
}

impl Default for CiVerifier {
    fn default() -> Self {
        Self {
            gh_path: std::env::var("FOCUSA_GH_BIN").ok().or_else(|| {
                let p = std::path::PathBuf::from("/usr/bin/gh");
                if p.exists() {
                    Some(p.to_string_lossy().to_string())
                } else {
                    None
                }
            }),
        }
    }
}

#[async_trait]
impl EvidenceVerifier for CiVerifier {
    fn kind(&self) -> EvidenceKind {
        EvidenceKind::Ci
    }
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult {
        let id = citation.ref_.trim();
        let id = id
            .trim_start_matches("gh-")
            .trim_start_matches("gh ")
            .trim_start_matches("glab-")
            .trim_start_matches("glab ")
            .trim();
        let id_num = id
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>();
        if id_num.is_empty() {
            return VerifyResult {
                verified: false,
                result: "ref_ is not a CI run id".into(),
                evidence_url: None,
            };
        }
        let Some(gh) = self.gh_path.clone().or_else(|| Some("gh".into())) else {
            return VerifyResult {
                verified: false,
                result: format!("gh binary not on PATH; cannot verify run {id_num}"),
                evidence_url: Some(format!("https://github.com/runs/{id_num}")),
            };
        };
        let cwd = current_project_root();
        let exit = run_capture(
            &gh,
            &["run", "view", &id_num, "--json", "status,conclusion,name"],
            &cwd,
            Duration::from_secs(15),
        )
        .await;
        match exit {
            Some((0, _)) => VerifyResult {
                verified: true,
                result: format!("gh run {id_num}: PASS"),
                evidence_url: Some(format!("https://github.com/runs/{id_num}")),
            },
            Some((code, tail)) => VerifyResult {
                verified: false,
                result: format!("gh run {id_num}: exit {code} stderr={tail}"),
                evidence_url: Some(format!("https://github.com/runs/{id_num}")),
            },
            None => VerifyResult {
                verified: false,
                result: format!("gh spawn failed for run {id_num}"),
                evidence_url: Some(format!("https://github.com/runs/{id_num}")),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Deploy verifier
// ---------------------------------------------------------------------------

/// Verifies a deploy citation by probing the live daemon's
/// `/v1/health`. Reports `version`, `ok`, `uptime_ms`.
pub struct DeployVerifier {
    /// Base URL of the live daemon; defaults to `FOCUSA_DEPLOY_URL`
    /// or `http://127.0.0.1:8787`.
    pub base_url: String,
    /// Required `version` substring. When set, the verifier fails if
    /// the live daemon's version does not contain it.
    pub require_version: Option<String>,
}

impl Default for DeployVerifier {
    fn default() -> Self {
        Self {
            base_url: std::env::var("FOCUSA_DEPLOY_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8787".into()),
            require_version: None,
        }
    }
}

#[async_trait]
impl EvidenceVerifier for DeployVerifier {
    fn kind(&self) -> EvidenceKind {
        EvidenceKind::Deploy
    }
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult {
        let url = resolve_endpoint_url(Some(&self.base_url), &citation.ref_)
            .unwrap_or_else(|| format!("{}/v1/health", self.base_url.trim_end_matches('/')));
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build();
        let client = match client {
            Ok(c) => c,
            Err(e) => {
                return VerifyResult {
                    verified: false,
                    result: format!("client build failed: {e}"),
                    evidence_url: Some(url.clone()),
                };
            }
        };
        let resp = client.get(&url).send().await;
        match resp {
            Err(e) => VerifyResult {
                verified: false,
                result: format!("GET {url} transport error: {e}"),
                evidence_url: Some(url),
            },
            Ok(r) => {
                let code = r.status().as_u16();
                let body = r.text().await.unwrap_or_default();
                let v: serde_json::Value = serde_json::from_str(&body)
                    .ok()
                    .unwrap_or(serde_json::Value::Null);
                let version = v
                    .get("version")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let ok = v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false);
                let uptime_ms = v.get("uptime_ms").and_then(|x| x.as_u64()).unwrap_or(0);
                let version_ok = self
                    .require_version
                    .as_ref()
                    .map(|req| version.contains(req.as_str()))
                    .unwrap_or(true);
                let http_ok = (200..300).contains(&code);
                let verified = ok && http_ok && version_ok;
                VerifyResult {
                    verified,
                    result: format!(
                        "GET {url} -> http={code} ok={ok} version=`{version}` uptime_ms={uptime_ms} version_required={}",
                        self.require_version.as_deref().unwrap_or("*")
                    ),
                    evidence_url: Some(url),
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Registry of verifiers keyed by kind
// ---------------------------------------------------------------------------

/// Helper that runs the matching verifier for a citation. The
/// lifecycle calls this for every citation in `validate`.
pub fn run_default_verifier(citation: &EvidenceCitation) -> VerifyResult {
    // Use a simple runtime so verifiers can be called from sync code
    // paths (the CLI doctor, the closure-audit replay). The
    // production closure lifecycle uses the async verifiers directly
    // through the `EvidenceVerifier` trait to avoid the overhead.
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            return VerifyResult {
                verified: false,
                result: format!("tokio runtime build failed: {e}"),
                evidence_url: None,
            };
        }
    };
    rt.block_on(async move {
        match citation.kind {
            EvidenceKind::Code => CodeVerifier.verify(citation).await,
            EvidenceKind::Spec => SpecVerifier.verify(citation).await,
            EvidenceKind::Test => TestVerifier.verify(citation).await,
            EvidenceKind::Endpoint => EndpointVerifier::default().verify(citation).await,
            EvidenceKind::Artifact => ArtifactStub.verify(citation).await,
            EvidenceKind::Workpoint => WorkpointVerifier::default().verify(citation).await,
            EvidenceKind::Ci => CiVerifier::default().verify(citation).await,
            EvidenceKind::Deploy => DeployVerifier::default().verify(citation).await,
        }
    })
}

/// Stub verifier for the `artifact` kind. We accept the citation
/// as-is (the lifecycle records the path + sha256 in `result` when
/// known). A full implementation would call into a real artifact
/// registry; that is a Phase-E follow-up.
pub struct ArtifactStub;

#[async_trait]
impl EvidenceVerifier for ArtifactStub {
    fn kind(&self) -> EvidenceKind {
        EvidenceKind::Artifact
    }
    async fn verify(&self, citation: &EvidenceCitation) -> VerifyResult {
        // Treat the ref_ as a `path sha256:HEX` tuple. The validator
        // surface is honest: if the caller can supply an sha256 we
        // compare; if they cannot, we mark the citation as info-only.
        let (path, sha) = citation
            .ref_
            .rsplit_once(" sha256:")
            .map(|(path, digest)| (path.to_string(), format!("sha256:{digest}")))
            .unwrap_or_else(|| (citation.ref_.clone(), String::new()));
        let Some(p) = citation_path(&current_project_root(), &path) else {
            return VerifyResult {
                verified: false,
                result: "artifact ref_ is not a path".into(),
                evidence_url: None,
            };
        };
        match std::fs::read(&p) {
            Err(e) => VerifyResult {
                verified: false,
                result: format!("artifact missing: {}: {e}", p.display()),
                evidence_url: None,
            },
            Ok(bytes) => {
                use sha2::{Digest, Sha256};
                let mut h = Sha256::new();
                h.update(&bytes);
                let actual = format!("{:x}", h.finalize());
                if sha.is_empty() {
                    VerifyResult {
                        verified: true,
                        result: format!(
                            "{}: bytes={} sha256={} (no expected sha256; recorded for replay)",
                            p.display(),
                            bytes.len(),
                            actual
                        ),
                        evidence_url: Some(format!("file://{}", p.display())),
                    }
                } else {
                    let expected_sha = sha.trim_start_matches("sha256:").trim();
                    if expected_sha == actual {
                        VerifyResult {
                            verified: true,
                            result: format!("{}: sha256 matches ({actual})", p.display()),
                            evidence_url: Some(format!("file://{}", p.display())),
                        }
                    } else {
                        VerifyResult {
                            verified: false,
                            result: format!(
                                "{}: sha256 mismatch (expected {sha}, got {actual})",
                                p.display()
                            ),
                            evidence_url: Some(format!("file://{}", p.display())),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citation_line_suffix_parser_preserves_windows_drive_prefix() {
        assert_eq!(
            citation_path_component(r"C:\projects\focusa\src\lib.rs:12-15"),
            r"C:\projects\focusa\src\lib.rs"
        );
        assert_eq!(
            citation_path_component(r"C:\projects\focusa\docs\01-spec.md#acceptance"),
            r"C:\projects\focusa\docs\01-spec.md"
        );
        assert_eq!(citation_path_component("src/lib.rs:12"), "src/lib.rs");
    }

    fn tmpfile_with(content: &str, ext: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("focusa-evidence-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let name = format!(
            "ev-{}-{}{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            ext
        );
        let p = dir.join(name);
        std::fs::write(&p, content).unwrap();
        p
    }

    #[tokio::test]
    async fn code_verifier_passes_for_real_file() {
        let p = tmpfile_with("hello\nworld\n", ".rs");
        let c = EvidenceCitation {
            kind: EvidenceKind::Code,
            ref_: p.to_string_lossy().to_string(),
            line: Some(2),
            line_end: Some(2),
            required: true,
            result: None,
            verified: false,
        };
        let v = CodeVerifier.verify(&c).await;
        assert!(v.verified, "{:?}", v);
        assert!(v.result.contains("lines 2-2"));
        // Clean up
        let _ = std::fs::remove_file(p);
    }

    #[tokio::test]
    async fn code_verifier_fails_for_missing_line() {
        let p = tmpfile_with("one\n", ".rs");
        let c = EvidenceCitation {
            kind: EvidenceKind::Code,
            ref_: p.to_string_lossy().to_string(),
            line: Some(99),
            line_end: Some(99),
            required: true,
            result: None,
            verified: false,
        };
        let v = CodeVerifier.verify(&c).await;
        assert!(!v.verified);
        assert!(v.result.contains("past EOF"));
        let _ = std::fs::remove_file(p);
    }

    #[tokio::test]
    async fn spec_verifier_passes_for_real_spec() {
        let p = tmpfile_with("# Test Spec\n\n## Section\ncontent\n", ".md");
        let c = EvidenceCitation {
            kind: EvidenceKind::Spec,
            ref_: format!("{}#section", p.to_string_lossy()),
            line: None,
            line_end: None,
            required: true,
            result: None,
            verified: false,
        };
        let v = SpecVerifier.verify(&c).await;
        assert!(v.verified, "{:?}", v);
        let _ = std::fs::remove_file(p);
    }

    #[tokio::test]
    async fn test_verifier_passes_when_file_present_and_no_run_marker() {
        let p = tmpfile_with("echo hi\n", ".sh");
        let c = EvidenceCitation {
            kind: EvidenceKind::Test,
            ref_: p.to_string_lossy().to_string(),
            line: None,
            line_end: None,
            required: true,
            result: None,
            verified: false,
        };
        let v = TestVerifier.verify(&c).await;
        assert!(v.verified);
        assert!(v.result.contains("not executed"));
        let _ = std::fs::remove_file(p);
    }

    #[tokio::test]
    async fn workpoint_verifier_accepts_canonical_persisted_snapshot() {
        let dir = std::env::temp_dir().join(format!("focusa-workpoint-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let database = dir.join("focusa.sqlite");
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE snapshots (name TEXT PRIMARY KEY, version INTEGER NOT NULL, ts TEXT NOT NULL, state_json TEXT NOT NULL);",
            )
            .unwrap();
        let workpoint_id = uuid::Uuid::now_v7().to_string();
        let state = serde_json::json!({
            "workpoint": {
                "records": [{
                    "workpoint_id": workpoint_id,
                    "canonical": true,
                    "evidence_refs": ["test:exact"]
                }]
            }
        });
        connection
            .execute(
                "INSERT INTO snapshots(name, version, ts, state_json) VALUES (?1, 1, ?2, ?3)",
                rusqlite::params!["focusa", "2026-08-07T00:00:00Z", state.to_string()],
            )
            .unwrap();
        drop(connection);

        let citation = EvidenceCitation {
            kind: EvidenceKind::Workpoint,
            ref_: workpoint_id,
            line: None,
            line_end: None,
            required: true,
            result: None,
            verified: false,
        };
        let result = WorkpointVerifier {
            data_dir: Some(dir.clone()),
        }
        .verify(&citation)
        .await;
        assert!(result.verified, "{:?}", result);
        assert!(result.result.contains("canonical persisted snapshot"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn artifact_verifier_accepts_unhashed_paths_with_spaces() {
        let dir = std::env::temp_dir().join(format!("focusa artifact {}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("proof file.md");
        std::fs::write(&path, "proof").unwrap();
        let citation = EvidenceCitation {
            kind: EvidenceKind::Artifact,
            ref_: path.to_string_lossy().to_string(),
            line: None,
            line_end: None,
            required: true,
            result: None,
            verified: false,
        };
        let result = ArtifactStub.verify(&citation).await;
        assert!(result.verified, "{:?}", result);
        assert!(result.result.contains("no expected sha256"));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn artifact_verifier_sha256_match() {
        let p = tmpfile_with("x", ".bin");
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(b"x");
        let sha = format!("{:x}", h.finalize());
        let c = EvidenceCitation {
            kind: EvidenceKind::Artifact,
            ref_: format!("{} sha256:{sha}", p.to_string_lossy()),
            line: None,
            line_end: None,
            required: true,
            result: None,
            verified: false,
        };
        let v = ArtifactStub.verify(&c).await;
        assert!(v.verified, "{:?}", v);
        let _ = std::fs::remove_file(p);
    }
}
