//! Provider credential token vault and secure server-side header injection.

use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Local secure token vault holding provider API keys.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TokenVault {
    anthropic_key: RwLock<Option<String>>,
    openai_key: RwLock<Option<String>>,
    deepseek_key: RwLock<Option<String>>,
}

impl TokenVault {
    pub fn new() -> Self {
        // Read keys from environment if available
        let anthropic_key = std::env::var("BURNRATE_ANTHROPIC_KEY")
            .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
            .ok();
        let openai_key = std::env::var("BURNRATE_OPENAI_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .ok();
        let deepseek_key = std::env::var("BURNRATE_DEEPSEEK_KEY")
            .or_else(|_| std::env::var("DEEPSEEK_API_KEY"))
            .ok();

        Self {
            anthropic_key: RwLock::new(anthropic_key),
            openai_key: RwLock::new(openai_key),
            deepseek_key: RwLock::new(deepseek_key),
        }
    }

    pub fn set_anthropic_key(&self, key: impl Into<String>) {
        if let Ok(mut lock) = self.anthropic_key.write() {
            *lock = Some(key.into());
        }
    }

    pub fn set_openai_key(&self, key: impl Into<String>) {
        if let Ok(mut lock) = self.openai_key.write() {
            *lock = Some(key.into());
        }
    }

    /// Check if vault has credentials configured.
    pub fn has_credentials(&self) -> bool {
        let has_ant = self.anthropic_key.read().map(|k| k.is_some()).unwrap_or(false);
        let has_oai = self.openai_key.read().map(|k| k.is_some()).unwrap_or(false);
        has_ant || has_oai
    }

    /// Inject authenticated authorization headers for upstream forwards.
    #[allow(clippy::collapsible_if)]
    pub fn inject_auth_headers(
        &self,
        headers: &mut axum::http::HeaderMap,
        model_name: &str,
    ) -> bool {
        if model_name.contains("claude") {
            if let Ok(lock) = self.anthropic_key.read() {
                if let Some(ref key) = *lock {
                    if let Ok(val) = axum::http::HeaderValue::from_str(key) {
                        headers.insert(axum::http::HeaderName::from_static("x-api-key"), val);
                        return true;
                    }
                }
            }
        } else if let Ok(lock) = self.openai_key.read() {
            if let Some(ref key) = *lock {
                let bearer = format!("Bearer {}", key);
                if let Ok(val) = axum::http::HeaderValue::from_str(&bearer) {
                    headers.insert(axum::http::header::AUTHORIZATION, val);
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_vault_injection() {
        let vault = TokenVault::default();
        vault.set_anthropic_key("sk-ant-vault-test-secret-12345");
        vault.set_openai_key("sk-proj-vault-test-secret-67890");

        assert!(vault.has_credentials());

        let mut headers = axum::http::HeaderMap::new();
        let injected_ant = vault.inject_auth_headers(&mut headers, "claude-3-7-sonnet");
        assert!(injected_ant);
        assert_eq!(
            headers.get("x-api-key").unwrap(),
            "sk-ant-vault-test-secret-12345"
        );

        let mut oai_headers = axum::http::HeaderMap::new();
        let injected_oai = vault.inject_auth_headers(&mut oai_headers, "gpt-4o");
        assert!(injected_oai);
        assert_eq!(
            oai_headers.get(axum::http::header::AUTHORIZATION).unwrap(),
            "Bearer sk-proj-vault-test-secret-67890"
        );
    }
}
