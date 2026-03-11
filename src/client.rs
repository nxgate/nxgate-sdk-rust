use reqwest::StatusCode;

use crate::auth::TokenManager;
use crate::error::NXGateError;
use crate::hmac::HmacSigner;
use crate::types::*;

const DEFAULT_BASE_URL: &str = "https://api.nxgate.com.br";
const MAX_RETRY_ATTEMPTS: u32 = 2;

/// Builder para construir o `NXGateClient` com padrão builder.
#[derive(Debug, Default)]
pub struct NXGateClientBuilder {
    client_id: Option<String>,
    client_secret: Option<String>,
    hmac_secret: Option<String>,
    base_url: Option<String>,
}

impl NXGateClientBuilder {
    /// Define o client_id OAuth2 (obrigatório).
    pub fn client_id(mut self, id: &str) -> Self {
        self.client_id = Some(id.to_string());
        self
    }

    /// Define o client_secret OAuth2 (obrigatório).
    pub fn client_secret(mut self, secret: &str) -> Self {
        self.client_secret = Some(secret.to_string());
        self
    }

    /// Define o secret HMAC para assinatura de requisições (opcional).
    pub fn hmac_secret(mut self, secret: &str) -> Self {
        self.hmac_secret = Some(secret.to_string());
        self
    }

    /// Define a URL base da API (padrão: https://api.nxgate.com.br).
    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_string());
        self
    }

    /// Constrói o `NXGateClient`.
    pub fn build(self) -> Result<NXGateClient, NXGateError> {
        let client_id = self
            .client_id
            .ok_or_else(|| NXGateError::Config("client_id é obrigatório".into()))?;
        let client_secret = self
            .client_secret
            .ok_or_else(|| NXGateError::Config("client_secret é obrigatório".into()))?;

        let base_url = self
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| NXGateError::Config(format!("Falha ao criar HTTP client: {}", e)))?;

        let hmac_signer = self.hmac_secret.map(|secret| {
            HmacSigner::new(secret, client_id.clone())
        });

        let token_manager =
            TokenManager::new(client_id, client_secret, base_url.clone(), http.clone());

        Ok(NXGateClient {
            http,
            base_url,
            token_manager,
            hmac_signer,
        })
    }
}

/// Cliente principal do SDK NXGATE PIX.
///
/// Gerencia autenticação OAuth2, assinatura HMAC (opcional) e chamadas à API.
#[derive(Debug)]
pub struct NXGateClient {
    http: reqwest::Client,
    base_url: String,
    token_manager: TokenManager,
    hmac_signer: Option<HmacSigner>,
}

impl NXGateClient {
    /// Inicia o builder do cliente.
    pub fn builder() -> NXGateClientBuilder {
        NXGateClientBuilder::default()
    }

    // -----------------------------------------------------------------------
    // PIX Gerar (cash-in)
    // -----------------------------------------------------------------------

    /// Gera uma cobrança PIX (cash-in) e retorna o QR Code.
    pub async fn pix_generate(
        &self,
        request: PixGenerateRequest,
    ) -> Result<PixGenerateResponse, NXGateError> {
        let path = "/pix/gerar";
        let body = serde_json::to_string(&request)?;
        let resp = self.post_with_retry(path, &body).await?;
        let result: PixGenerateResponse = serde_json::from_str(&resp)?;
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // PIX Sacar (cash-out)
    // -----------------------------------------------------------------------

    /// Realiza um saque PIX (cash-out).
    pub async fn pix_withdraw(
        &self,
        request: PixWithdrawRequest,
    ) -> Result<PixWithdrawResponse, NXGateError> {
        let path = "/pix/sacar";
        let body = serde_json::to_string(&request)?;
        let resp = self.post_with_retry(path, &body).await?;
        let result: PixWithdrawResponse = serde_json::from_str(&resp)?;
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Balance
    // -----------------------------------------------------------------------

    /// Consulta o saldo da conta.
    pub async fn get_balance(&self) -> Result<BalanceResponse, NXGateError> {
        let path = "/v1/balance";
        let resp = self.get_with_retry(path).await?;
        let result: BalanceResponse = serde_json::from_str(&resp)?;
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Transactions
    // -----------------------------------------------------------------------

    /// Consulta uma transação pelo tipo e ID.
    pub async fn get_transaction(
        &self,
        tx_type: TransactionType,
        txid: &str,
    ) -> Result<TransactionResponse, NXGateError> {
        let path = format!("/v1/transactions?type={}&txid={}", tx_type, txid);
        let resp = self.get_with_retry(&path).await?;
        let result: TransactionResponse = serde_json::from_str(&resp)?;
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Internal: request execution with retry on 503
    // -----------------------------------------------------------------------

    async fn post_with_retry(
        &self,
        path: &str,
        body: &str,
    ) -> Result<String, NXGateError> {
        let mut last_error = String::new();

        for attempt in 0..=MAX_RETRY_ATTEMPTS {
            if attempt > 0 {
                let delay = std::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
            }

            let token = self.token_manager.get_token().await?;
            let url = format!("{}{}", self.base_url, path);

            let mut req = self
                .http
                .post(&url)
                .bearer_auth(&token)
                .header("Content-Type", "application/json")
                .body(body.to_string());

            if let Some(ref signer) = self.hmac_signer {
                let headers = signer.sign("POST", path, body)?;
                req = req
                    .header("X-Client-ID", &headers.x_client_id)
                    .header("X-HMAC-Signature", &headers.x_hmac_signature)
                    .header("X-HMAC-Timestamp", &headers.x_hmac_timestamp)
                    .header("X-HMAC-Nonce", &headers.x_hmac_nonce);
            }

            let resp = req.send().await?;
            let status = resp.status();

            if status == StatusCode::UNAUTHORIZED {
                self.token_manager.invalidate().await;
                last_error = format!("HTTP 401 Unauthorized");
                continue;
            }

            if status == StatusCode::SERVICE_UNAVAILABLE {
                last_error = format!("HTTP 503 Service Unavailable");
                continue;
            }

            let response_body = resp.text().await?;

            if !status.is_success() {
                return Err(NXGateError::Api {
                    status: status.as_u16(),
                    body: response_body,
                });
            }

            return Ok(response_body);
        }

        Err(NXGateError::MaxRetries {
            attempts: MAX_RETRY_ATTEMPTS + 1,
            last_error,
        })
    }

    async fn get_with_retry(&self, path: &str) -> Result<String, NXGateError> {
        let mut last_error = String::new();

        for attempt in 0..=MAX_RETRY_ATTEMPTS {
            if attempt > 0 {
                let delay = std::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
            }

            let token = self.token_manager.get_token().await?;
            let url = format!("{}{}", self.base_url, path);

            let mut req = self.http.get(&url).bearer_auth(&token);

            if let Some(ref signer) = self.hmac_signer {
                let headers = signer.sign("GET", path, "")?;
                req = req
                    .header("X-Client-ID", &headers.x_client_id)
                    .header("X-HMAC-Signature", &headers.x_hmac_signature)
                    .header("X-HMAC-Timestamp", &headers.x_hmac_timestamp)
                    .header("X-HMAC-Nonce", &headers.x_hmac_nonce);
            }

            let resp = req.send().await?;
            let status = resp.status();

            if status == StatusCode::UNAUTHORIZED {
                self.token_manager.invalidate().await;
                last_error = format!("HTTP 401 Unauthorized");
                continue;
            }

            if status == StatusCode::SERVICE_UNAVAILABLE {
                last_error = format!("HTTP 503 Service Unavailable");
                continue;
            }

            let response_body = resp.text().await?;

            if !status.is_success() {
                return Err(NXGateError::Api {
                    status: status.as_u16(),
                    body: response_body,
                });
            }

            return Ok(response_body);
        }

        Err(NXGateError::MaxRetries {
            attempts: MAX_RETRY_ATTEMPTS + 1,
            last_error,
        })
    }
}
