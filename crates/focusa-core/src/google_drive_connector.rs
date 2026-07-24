//! Google Drive Context connector built on the typed connector/auth lifecycle.

use crate::connector_auth::{ConnectorAuthLifecycle, ConnectorCredentialStatus};
use crate::connectors::{ConnectorErrorEnvelope, ConnectorHealth, ConnectorHealthStatus};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

const DRIVE_FILES_URL: &str = "https://www.googleapis.com/drive/v3/files";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleDriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(rename = "modifiedTime")]
    pub modified_time: Option<String>,
    pub size: Option<String>,
    #[serde(rename = "webViewLink")]
    pub web_view_link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleDriveListResponse {
    files: Vec<GoogleDriveFile>,
    next_page_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleDriveDiscovery {
    pub schema: String,
    pub connector_id: String,
    pub files: Vec<GoogleDriveFile>,
    pub next_page_token: Option<String>,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleDriveImportCandidate {
    pub schema: String,
    pub connector_id: String,
    pub project_root: String,
    pub continuity_id: String,
    pub attachment_id: String,
    pub source_system: String,
    pub source_ref: String,
    pub source_url: String,
    pub name: String,
    pub mime_type: String,
    pub evidence_refs: Vec<String>,
}

pub struct GoogleDriveConnector {
    connector_id: String,
    project_root: String,
    continuity_id: String,
    attachment_id: String,
    auth: ConnectorAuthLifecycle,
    client: Client,
}

impl GoogleDriveConnector {
    pub fn new(
        connector_id: String,
        project_root: String,
        continuity_id: String,
        attachment_id: String,
        auth: ConnectorAuthLifecycle,
    ) -> Result<Self, ConnectorErrorEnvelope> {
        if connector_id.trim().is_empty()
            || project_root.trim().is_empty()
            || continuity_id.trim().is_empty()
            || attachment_id.trim().is_empty()
        {
            return Err(error(&connector_id, "scope_missing", "Google Drive connector requires exact Focusa scope"));
        }
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|_| error(&connector_id, "client_init", "Google Drive client initialization failed"))?;
        Ok(Self { connector_id, project_root, continuity_id, attachment_id, auth, client })
    }

    pub async fn health(&self) -> ConnectorHealth {
        let started = Instant::now();
        let token = match self.auth.access_token() {
            Ok(token) => token,
            Err(_) => return ConnectorHealth {
                status: ConnectorHealthStatus::Unauthorized,
                checked_at: chrono::Utc::now().to_rfc3339(),
                latency_ms: 0,
                message: "Google Drive authorization required".into(),
            },
        };
        let status = self.client.get(DRIVE_FILES_URL)
            .bearer_auth(token)
            .query(&[("pageSize", "1"), ("fields", "files(id)")])
            .send().await;
        let (status, message) = match status {
            Ok(response) if response.status().is_success() => (ConnectorHealthStatus::Ready, "Google Drive connector ready"),
            Ok(response) if response.status().as_u16() == 401 || response.status().as_u16() == 403 => (ConnectorHealthStatus::Unauthorized, "Google Drive authorization expired or revoked"),
            Ok(_) => (ConnectorHealthStatus::Degraded, "Google Drive health returned a non-success status"),
            Err(_) => (ConnectorHealthStatus::Offline, "Google Drive health request failed"),
        };
        ConnectorHealth { status, checked_at: chrono::Utc::now().to_rfc3339(), latency_ms: started.elapsed().as_millis() as u64, message: message.into() }
    }

    pub async fn discover(
        &self,
        query: Option<&str>,
        page_token: Option<&str>,
        page_size: u16,
    ) -> Result<GoogleDriveDiscovery, ConnectorErrorEnvelope> {
        let token = self.token()?;
        let bounded_page_size = page_size.clamp(1, 100).to_string();
        let mut request = self.client.get(DRIVE_FILES_URL).bearer_auth(token).query(&[
            ("pageSize", bounded_page_size.as_str()),
            ("fields", "nextPageToken,files(id,name,mimeType,modifiedTime,size,webViewLink)"),
        ]);
        if let Some(value) = query { request = request.query(&[("q", value)]); }
        if let Some(value) = page_token { request = request.query(&[("pageToken", value)]); }
        let response = request.send().await.map_err(|_| error(&self.connector_id, "transport", "Google Drive discovery failed"))?;
        if !response.status().is_success() { return Err(status_error(&self.connector_id, response.status().as_u16())); }
        let payload = response.json::<GoogleDriveListResponse>().await.map_err(|_| error(&self.connector_id, "invalid_response", "Google Drive discovery response was invalid"))?;
        Ok(GoogleDriveDiscovery {
            schema: "focusa.google_drive_discovery.v1".into(),
            connector_id: self.connector_id.clone(),
            files: payload.files,
            next_page_token: payload.next_page_token,
            evidence_ref: format!("connector:{}:drive-discovery", self.connector_id),
        })
    }

    pub async fn import_candidate(&self, file_id: &str) -> Result<GoogleDriveImportCandidate, ConnectorErrorEnvelope> {
        if file_id.trim().is_empty() { return Err(error(&self.connector_id, "file_id_missing", "Google Drive file id is required")); }
        let token = self.token()?;
        let url = format!("{DRIVE_FILES_URL}/{file_id}");
        let response = self.client.get(&url).bearer_auth(token).query(&[("fields", "id,name,mimeType,modifiedTime,size,webViewLink")]).send().await.map_err(|_| error(&self.connector_id, "transport", "Google Drive import lookup failed"))?;
        if !response.status().is_success() { return Err(status_error(&self.connector_id, response.status().as_u16())); }
        let file = response.json::<GoogleDriveFile>().await.map_err(|_| error(&self.connector_id, "invalid_response", "Google Drive file metadata was invalid"))?;
        Ok(GoogleDriveImportCandidate {
            schema: "focusa.workspace_artifact_intake.request.v1".into(),
            connector_id: self.connector_id.clone(),
            project_root: self.project_root.clone(),
            continuity_id: self.continuity_id.clone(),
            attachment_id: self.attachment_id.clone(),
            source_system: "connector".into(),
            source_ref: format!("google-drive:{}", file.id),
            source_url: file.web_view_link.clone().unwrap_or(url),
            name: file.name,
            mime_type: file.mime_type,
            evidence_refs: vec![format!("connector:{}:google-drive:{}", self.connector_id, file.id)],
        })
    }

    pub fn revoke(&self) -> ConnectorCredentialStatus { self.auth.revoke() }

    fn token(&self) -> Result<String, ConnectorErrorEnvelope> {
        self.auth.access_token().map_err(|status| error(&self.connector_id, &status.status, &status.recovery_action))
    }
}

fn status_error(connector_id: &str, code: u16) -> ConnectorErrorEnvelope {
    let class = if code == 401 || code == 403 { "authorization_required" } else if code == 429 { "rate_limited" } else { "http_status" };
    error(connector_id, class, "Google Drive returned a non-success status")
}

fn error(connector_id: &str, class: &str, message: &str) -> ConnectorErrorEnvelope {
    ConnectorErrorEnvelope {
        schema: "focusa.connector_error.v1".into(),
        connector_id: connector_id.into(),
        status: "blocked".into(),
        failure_class: class.into(),
        message: message.into(),
        retriable: matches!(class, "rate_limited" | "transport"),
        retry_after_ms: if class == "rate_limited" { Some(1_000) } else { None },
    }
}
