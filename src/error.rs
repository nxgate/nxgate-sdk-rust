use thiserror::Error;

/// Enum de erros do SDK NXGATE.
#[derive(Debug, Error)]
pub enum NXGateError {
    /// Erro de requisição HTTP (rede, TLS, timeout, etc.).
    #[error("Erro HTTP: {0}")]
    Http(#[from] reqwest::Error),

    /// Erro ao serializar/deserializar JSON.
    #[error("Erro JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Erro retornado pela API com status HTTP e corpo da resposta.
    #[error("Erro da API (HTTP {status}): {body}")]
    Api { status: u16, body: String },

    /// Falha na autenticação OAuth2.
    #[error("Erro de autenticação: {0}")]
    Auth(String),

    /// Erro de configuração do builder (campos obrigatórios ausentes).
    #[error("Erro de configuração: {0}")]
    Config(String),

    /// Erro ao computar HMAC.
    #[error("Erro HMAC: {0}")]
    Hmac(String),

    /// Erro ao parsear webhook.
    #[error("Erro ao processar webhook: {0}")]
    Webhook(String),

    /// Todas as tentativas de retry falharam.
    #[error("Máximo de tentativas excedido após {attempts} tentativas: {last_error}")]
    MaxRetries { attempts: u32, last_error: String },
}
