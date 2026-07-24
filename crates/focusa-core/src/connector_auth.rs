//! Portable OAuth2 and operating-system keyring lifecycle for connectors.
//!
//! Secrets never enter serializable Focusa records, logs, Evidence, or errors.

use keyring::Entry;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, TokenUrl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorOAuthConfig {
    pub connector_id: String,
    pub client_id: String,
    pub authorization_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub scopes: Vec<String>,
    pub keyring_service: String,
    pub keyring_account: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorAuthorizationRequest {
    pub connector_id: String,
    pub authorization_url: String,
    pub csrf_ref: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectorCredentialStatus {
    pub connector_id: String,
    pub status: String,
    pub recovery_action: String,
}

pub struct ConnectorAuthLifecycle {
    config: ConnectorOAuthConfig,
}

impl ConnectorAuthLifecycle {
    pub fn new(config: ConnectorOAuthConfig) -> Result<Self, ConnectorCredentialStatus> {
        if config.connector_id.trim().is_empty()
            || config.client_id.trim().is_empty()
            || config.keyring_service.trim().is_empty()
            || config.keyring_account.trim().is_empty()
        {
            return Err(status(&config.connector_id, "invalid_config", "repair_connector_configuration"));
        }
        AuthUrl::new(config.authorization_url.clone())
            .and_then(|_| TokenUrl::new(config.token_url.clone()))
            .and_then(|_| RedirectUrl::new(config.redirect_url.clone()))
            .map_err(|_| status(&config.connector_id, "invalid_config", "repair_connector_configuration"))?;
        Ok(Self { config })
    }

    pub fn begin_authorization(&self) -> Result<ConnectorAuthorizationRequest, ConnectorCredentialStatus> {
        let client = BasicClient::new(ClientId::new(self.config.client_id.clone()))
            .set_auth_uri(AuthUrl::new(self.config.authorization_url.clone()).map_err(|_| self.invalid())?)
            .set_token_uri(TokenUrl::new(self.config.token_url.clone()).map_err(|_| self.invalid())?)
            .set_redirect_uri(RedirectUrl::new(self.config.redirect_url.clone()).map_err(|_| self.invalid())?);
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let mut request = client.authorize_url(CsrfToken::new_random).set_pkce_challenge(challenge);
        for scope in &self.config.scopes {
            request = request.add_scope(Scope::new(scope.clone()));
        }
        let (url, csrf) = request.url();
        self.entry("pkce")?.set_password(verifier.secret()).map_err(|_| self.keyring_failure())?;
        self.entry("csrf")?.set_password(csrf.secret()).map_err(|_| self.keyring_failure())?;
        Ok(ConnectorAuthorizationRequest {
            connector_id: self.config.connector_id.clone(),
            authorization_url: url.to_string(),
            csrf_ref: format!("keyring:{}:csrf", self.config.connector_id),
            expires_in_seconds: 600,
        })
    }

    pub fn store_access_token(&self, access_token: &str) -> Result<ConnectorCredentialStatus, ConnectorCredentialStatus> {
        if access_token.trim().is_empty() {
            return Err(status(&self.config.connector_id, "token_missing", "reauthorize_connector"));
        }
        self.entry("access")?.set_password(access_token).map_err(|_| self.keyring_failure())?;
        Ok(status(&self.config.connector_id, "authorized", "none"))
    }

    pub fn access_token(&self) -> Result<String, ConnectorCredentialStatus> {
        self.entry("access")?.get_password().map_err(|_| status(&self.config.connector_id, "authorization_required", "reauthorize_connector"))
    }

    pub fn revoke(&self) -> ConnectorCredentialStatus {
        for suffix in ["access", "pkce", "csrf"] {
            if let Ok(entry) = self.entry(suffix) {
                let _ = entry.delete_credential();
            }
        }
        status(&self.config.connector_id, "revoked", "authorize_connector")
    }

    fn entry(&self, suffix: &str) -> Result<Entry, ConnectorCredentialStatus> {
        Entry::new(
            &self.config.keyring_service,
            &format!("{}:{suffix}", self.config.keyring_account),
        )
        .map_err(|_| self.keyring_failure())
    }

    fn invalid(&self) -> ConnectorCredentialStatus {
        status(&self.config.connector_id, "invalid_config", "repair_connector_configuration")
    }

    fn keyring_failure(&self) -> ConnectorCredentialStatus {
        status(&self.config.connector_id, "keyring_unavailable", "repair_os_keyring")
    }
}

fn status(connector_id: &str, state: &str, recovery: &str) -> ConnectorCredentialStatus {
    ConnectorCredentialStatus {
        connector_id: connector_id.into(),
        status: state.into(),
        recovery_action: recovery.into(),
    }
}
