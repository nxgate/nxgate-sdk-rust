use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// OAuth2
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub(crate) struct TokenRequest<'a> {
    pub grant_type: &'a str,
    pub client_id: &'a str,
    pub client_secret: &'a str,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

// ---------------------------------------------------------------------------
// PIX Gerar (cash-in)
// ---------------------------------------------------------------------------

/// Usuário para split de pagamento.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SplitUser {
    pub username: String,
    pub percentage: f64,
}

/// Requisição para gerar cobrança PIX (cash-in).
#[derive(Debug, Clone, Serialize, Default)]
pub struct PixGenerateRequest {
    /// Valor da cobrança em reais (ex: 100.50).
    pub valor: f64,
    /// Nome completo do pagador.
    pub nome_pagador: String,
    /// CPF ou CNPJ do pagador.
    pub documento_pagador: String,
    /// Forçar dados do pagador mesmo se já cadastrado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forcar_pagador: Option<bool>,
    /// E-mail do pagador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_pagador: Option<String>,
    /// Celular do pagador.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub celular: Option<String>,
    /// Descrição da cobrança.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub descricao: Option<String>,
    /// URL de webhook para notificações.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Identificador customizado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magic_id: Option<String>,
    /// Chave de API adicional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Lista de usuários para split de pagamento.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub split_users: Option<Vec<SplitUser>>,
}

/// Resposta da geração de cobrança PIX.
#[derive(Debug, Clone, Deserialize)]
pub struct PixGenerateResponse {
    pub status: String,
    pub message: String,
    pub payment_code: String,
    pub id_transaction: String,
    pub payment_code_base64: String,
}

// ---------------------------------------------------------------------------
// PIX Sacar (cash-out)
// ---------------------------------------------------------------------------

/// Tipo de chave PIX.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PixKeyType {
    Cpf,
    Cnpj,
    Phone,
    Email,
    Random,
}

impl std::fmt::Display for PixKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PixKeyType::Cpf => "Cpf",
            PixKeyType::Cnpj => "Cnpj",
            PixKeyType::Phone => "Phone",
            PixKeyType::Email => "Email",
            PixKeyType::Random => "Random",
        };
        write!(f, "{}", s)
    }
}

/// Requisição para saque PIX (cash-out).
#[derive(Debug, Clone, Serialize, Default)]
pub struct PixWithdrawRequest {
    /// Valor do saque em reais.
    pub valor: f64,
    /// Chave PIX do destinatário.
    pub chave_pix: String,
    /// Tipo da chave PIX.
    pub tipo_chave: PixKeyType,
    /// CPF/CNPJ do destinatário (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documento: Option<String>,
    /// URL de webhook para notificações.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    /// Identificador customizado.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magic_id: Option<String>,
    /// Chave de API adicional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

impl Default for PixKeyType {
    fn default() -> Self {
        PixKeyType::Cpf
    }
}

/// Resposta do saque PIX.
#[derive(Debug, Clone, Deserialize)]
pub struct PixWithdrawResponse {
    pub status: String,
    pub message: String,
    pub internalreference: String,
}

// ---------------------------------------------------------------------------
// Balance
// ---------------------------------------------------------------------------

/// Resposta de consulta de saldo.
#[derive(Debug, Clone, Deserialize)]
pub struct BalanceResponse {
    pub balance: f64,
    pub blocked: f64,
    pub available: f64,
}

// ---------------------------------------------------------------------------
// Transactions
// ---------------------------------------------------------------------------

/// Tipo de transação para consulta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionType {
    CashIn,
    CashOut,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::CashIn => write!(f, "cash-in"),
            TransactionType::CashOut => write!(f, "cash-out"),
        }
    }
}

/// Resposta de consulta de transação.
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionResponse {
    pub id_transaction: String,
    pub status: String,
    pub amount: f64,
    #[serde(default)]
    pub paid_at: Option<String>,
    #[serde(default)]
    pub end_to_end: Option<String>,
}
