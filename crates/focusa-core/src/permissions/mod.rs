//! Capability Permissions — docs/25-26
//!
//! 3 token types: Owner, Agent, Integration.
//! 3 permission classes: Read, Command, Administrative.
//! Scope format: <domain>:<action>.
//! Bearer auth: Authorization: Bearer <token>.

use crate::types::*;
use chrono::Utc;
use uuid::Uuid;

/// Token store.
#[derive(Debug, Default)]
pub struct TokenStore {
    pub tokens: Vec<ApiToken>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self { tokens: vec![] }
    }

    /// Create a new token.
    pub fn create_token(
        &mut self,
        token_type: ApiTokenType,
        scopes: Vec<PermissionScope>,
        ttl_secs: Option<u64>,
    ) -> Uuid {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let expires_at = ttl_secs.map(|s| now + chrono::Duration::seconds(s as i64));

        self.tokens.push(ApiToken {
            token_id: id,
            token_type,
            scopes,
            created_at: now,
            expires_at,
            revoked: false,
        });

        id
    }

    /// Validate a token and check scope.
    pub fn validate(&self, token_id: Uuid, domain: &str, action: &str) -> Result<(), String> {
        let token = self
            .tokens
            .iter()
            .find(|t| t.token_id == token_id)
            .ok_or("Token not found")?;

        if token.revoked {
            return Err("Token revoked".into());
        }

        if let Some(expires) = token.expires_at
            && Utc::now() > expires
        {
            return Err("Token expired".into());
        }

        // Owner tokens have full access.
        if token.token_type == ApiTokenType::Owner {
            return Ok(());
        }

        // Integration tokens are read-only.
        if token.token_type == ApiTokenType::Integration && action != "read" {
            return Err("Integration tokens are read-only".into());
        }

        // Check scope match.
        let has_scope = token.scopes.iter().any(|s| {
            (s.domain == "*" || s.domain == domain) && (s.action == "*" || s.action == action)
        });

        if has_scope {
            Ok(())
        } else {
            Err(format!("Insufficient scope: {}:{}", domain, action))
        }
    }

    /// Revoke a token.
    pub fn revoke(&mut self, token_id: Uuid) -> Result<(), String> {
        let token = self
            .tokens
            .iter_mut()
            .find(|t| t.token_id == token_id)
            .ok_or("Token not found")?;
        token.revoked = true;
        Ok(())
    }

    /// List active (non-revoked, non-expired) tokens.
    pub fn active_tokens(&self) -> Vec<&ApiToken> {
        let now = Utc::now();
        self.tokens
            .iter()
            .filter(|t| !t.revoked && t.expires_at.is_none_or(|e| e > now))
            .collect()
    }
}

/// Classify a permission scope.
pub fn classify_scope(_domain: &str, action: &str) -> PermissionClass {
    match action {
        "read" | "list" | "inspect" => PermissionClass::Read,
        "create" | "update" | "delete" | "execute" => PermissionClass::Command,
        "admin" | "configure" | "revoke" | "grant" => PermissionClass::Administrative,
        _ => PermissionClass::Read, // Default to least privilege.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owner_full_access() {
        let mut store = TokenStore::new();
        let id = store.create_token(ApiTokenType::Owner, vec![], None);
        assert!(store.validate(id, "anything", "anything").is_ok());
    }

    #[test]
    fn test_integration_read_only() {
        let mut store = TokenStore::new();
        let id = store.create_token(
            ApiTokenType::Integration,
            vec![PermissionScope {
                domain: "*".into(),
                action: "*".into(),
            }],
            None,
        );
        assert!(store.validate(id, "focus", "read").is_ok());
        assert!(store.validate(id, "focus", "update").is_err());
    }

    #[test]
    fn test_revoke() {
        let mut store = TokenStore::new();
        let id = store.create_token(ApiTokenType::Agent, vec![], None);
        store.revoke(id).unwrap();
        assert!(store.validate(id, "x", "read").is_err());
    }
}
