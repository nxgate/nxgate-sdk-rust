use serde::Deserialize;

use crate::error::NXGateError;

// ---------------------------------------------------------------------------
// Cash-in webhook
// ---------------------------------------------------------------------------

/// Dados do webhook de cash-in (QR Code pago/estornado).
#[derive(Debug, Clone, Deserialize)]
pub struct CashInWebhookData {
    pub amount: f64,
    pub status: String,
    pub worked: bool,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub tx_id: Option<String>,
    #[serde(default)]
    pub end_to_end: Option<String>,
}

/// Evento de webhook de cash-in.
#[derive(Debug, Clone, Deserialize)]
pub struct CashInWebhook {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: CashInWebhookData,
}

// ---------------------------------------------------------------------------
// Cash-out webhook
// ---------------------------------------------------------------------------

/// Evento de webhook de cash-out.
#[derive(Debug, Clone, Deserialize)]
pub struct CashOutWebhook {
    #[serde(rename = "type")]
    pub event_type: String,
    pub worked: bool,
    pub status: String,
    pub id_transaction: String,
    pub amount: f64,
    pub key: String,
}

// ---------------------------------------------------------------------------
// Unified webhook event
// ---------------------------------------------------------------------------

/// Evento de webhook unificado (cash-in ou cash-out).
#[derive(Debug, Clone)]
pub enum WebhookEvent {
    /// QR Code pago.
    CashInPaid(CashInWebhook),
    /// QR Code estornado.
    CashInRefunded(CashInWebhook),
    /// Cash-out com sucesso.
    CashOutSuccess(CashOutWebhook),
    /// Erro no cash-out.
    CashOutError(CashOutWebhook),
    /// Cash-out estornado.
    CashOutRefunded(CashOutWebhook),
}

/// Detecta o tipo de evento a partir do campo "type".
#[derive(Debug, Deserialize)]
struct RawEvent {
    #[serde(rename = "type")]
    event_type: String,
}

/// Faz o parse de um payload de webhook e retorna o evento tipado.
///
/// # Exemplos
///
/// ```rust,no_run
/// use nxgate::parse_webhook;
///
/// let json = br#"{"type":"QR_CODE_COPY_AND_PASTE_PAID","data":{"amount":100.0,"status":"PAID","worked":true}}"#;
/// let event = parse_webhook(json).unwrap();
/// ```
pub fn parse_webhook(payload: &[u8]) -> Result<WebhookEvent, NXGateError> {
    let raw: RawEvent = serde_json::from_slice(payload)
        .map_err(|e| NXGateError::Webhook(format!("JSON inválido: {}", e)))?;

    match raw.event_type.as_str() {
        "QR_CODE_COPY_AND_PASTE_PAID" => {
            let wh: CashInWebhook = serde_json::from_slice(payload)
                .map_err(|e| NXGateError::Webhook(format!("Erro ao parsear cash-in: {}", e)))?;
            Ok(WebhookEvent::CashInPaid(wh))
        }
        "QR_CODE_COPY_AND_PASTE_REFUNDED" => {
            let wh: CashInWebhook = serde_json::from_slice(payload)
                .map_err(|e| NXGateError::Webhook(format!("Erro ao parsear cash-in: {}", e)))?;
            Ok(WebhookEvent::CashInRefunded(wh))
        }
        "PIX_CASHOUT_SUCCESS" => {
            let wh: CashOutWebhook = serde_json::from_slice(payload)
                .map_err(|e| NXGateError::Webhook(format!("Erro ao parsear cash-out: {}", e)))?;
            Ok(WebhookEvent::CashOutSuccess(wh))
        }
        "PIX_CASHOUT_ERROR" => {
            let wh: CashOutWebhook = serde_json::from_slice(payload)
                .map_err(|e| NXGateError::Webhook(format!("Erro ao parsear cash-out: {}", e)))?;
            Ok(WebhookEvent::CashOutError(wh))
        }
        "PIX_CASHOUT_REFUNDED" => {
            let wh: CashOutWebhook = serde_json::from_slice(payload)
                .map_err(|e| NXGateError::Webhook(format!("Erro ao parsear cash-out: {}", e)))?;
            Ok(WebhookEvent::CashOutRefunded(wh))
        }
        other => Err(NXGateError::Webhook(format!(
            "Tipo de evento desconhecido: {}",
            other
        ))),
    }
}
