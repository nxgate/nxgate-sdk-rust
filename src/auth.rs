use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::NXGateError;
use crate::types::{TokenRequest, TokenResponse};

/// Gerencia o token OAuth2, renovando automaticamente quando expirado.
#[derive(Debug)]
pub struct TokenManager {
    client_id: String,
    client_secret: String,
    base_url: String,
    http: reqwest::Client,
    token: Arc<RwLock<Option<CachedToken>>>,
}

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl TokenManager {
    pub fn new(
        client_id: String,
        client_secret: String,
        base_url: String,
        http: reqwest::Client,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            base_url,
            http,
            token: Arc::new(RwLock::new(None)),
        }
    }

    /// Retorna um token válido, renovando se necessário.
    pub async fn get_token(&self) -> Result<String, NXGateError> {
        // Tenta ler o token cacheado
        {
            let guard = self.token.read().await;
            if let Some(ref cached) = *guard {
                // Margem de 60 segundos antes da expiração
                if chrono::Utc::now() < cached.expires_at - chrono::Duration::seconds(60) {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // Token expirado ou inexistente: renova
        self.refresh_token().await
    }

    /// Força a renovação do token.
    async fn refresh_token(&self) -> Result<String, NXGateError> {
        let mut guard = self.token.write().await;

        // Double-check: outro task pode ter renovado enquanto esperávamos o lock
        if let Some(ref cached) = *guard {
            if chrono::Utc::now() < cached.expires_at - chrono::Duration::seconds(60) {
                return Ok(cached.access_token.clone());
            }
        }

        let url = format!("{}/oauth2/token", self.base_url);
        let body = TokenRequest {
            grant_type: "client_credentials",
            client_id: &self.client_id,
            client_secret: &self.client_secret,
        };

        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(NXGateError::Auth(format!(
                "Falha na autenticação (HTTP {}): {}",
                status, body
            )));
        }

        let token_resp: TokenResponse = resp.json().await?;
        let expires_at =
            chrono::Utc::now() + chrono::Duration::seconds(token_resp.expires_in as i64);

        let access_token = token_resp.access_token.clone();

        *guard = Some(CachedToken {
            access_token: access_token.clone(),
            expires_at,
        });

        Ok(access_token)
    }

    /// Invalida o token cacheado (útil após receber 401).
    pub async fn invalidate(&self) {
        let mut guard = self.token.write().await;
        *guard = None;
    }
}
