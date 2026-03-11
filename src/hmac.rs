use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;

use crate::error::NXGateError;

type HmacSha256 = Hmac<Sha256>;

/// Responsável por assinar requisições com HMAC-SHA256.
#[derive(Debug, Clone)]
pub struct HmacSigner {
    secret: String,
    client_id: String,
}

/// Headers HMAC gerados para uma requisição.
#[derive(Debug, Clone)]
pub struct HmacHeaders {
    pub x_client_id: String,
    pub x_hmac_signature: String,
    pub x_hmac_timestamp: String,
    pub x_hmac_nonce: String,
}

impl HmacSigner {
    /// Cria um novo signer HMAC.
    pub fn new(secret: String, client_id: String) -> Self {
        Self { secret, client_id }
    }

    /// Gera os headers HMAC para uma requisição.
    ///
    /// A mensagem assinada segue o formato:
    /// `METHOD\nPATH\nTIMESTAMP\nNONCE\nBODY`
    pub fn sign(
        &self,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<HmacHeaders, NXGateError> {
        let timestamp = Utc::now().to_rfc3339();
        let nonce = generate_nonce();

        let message = format!(
            "{}\n{}\n{}\n{}\n{}",
            method, path, timestamp, nonce, body
        );

        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .map_err(|e| NXGateError::Hmac(format!("Falha ao criar HMAC: {}", e)))?;

        mac.update(message.as_bytes());
        let result = mac.finalize();
        let signature = base64::engine::general_purpose::STANDARD.encode(result.into_bytes());

        Ok(HmacHeaders {
            x_client_id: self.client_id.clone(),
            x_hmac_signature: signature,
            x_hmac_timestamp: timestamp,
            x_hmac_nonce: nonce,
        })
    }

    /// Verifica uma assinatura HMAC recebida (útil para validar webhooks).
    pub fn verify(
        &self,
        method: &str,
        path: &str,
        timestamp: &str,
        nonce: &str,
        body: &str,
        signature: &str,
    ) -> Result<bool, NXGateError> {
        let message = format!(
            "{}\n{}\n{}\n{}\n{}",
            method, path, timestamp, nonce, body
        );

        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .map_err(|e| NXGateError::Hmac(format!("Falha ao criar HMAC: {}", e)))?;

        mac.update(message.as_bytes());

        let expected = base64::engine::general_purpose::STANDARD
            .decode(signature)
            .map_err(|e| NXGateError::Hmac(format!("Falha ao decodificar assinatura: {}", e)))?;

        Ok(mac.verify_slice(&expected).is_ok())
    }
}

/// Gera um nonce aleatório (32 caracteres alfanuméricos).
fn generate_nonce() -> String {
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_produces_valid_headers() {
        let signer = HmacSigner::new("test_secret".into(), "test_client".into());
        let headers = signer.sign("POST", "/pix/gerar", r#"{"valor":100}"#).unwrap();

        assert_eq!(headers.x_client_id, "test_client");
        assert!(!headers.x_hmac_signature.is_empty());
        assert!(!headers.x_hmac_timestamp.is_empty());
        assert_eq!(headers.x_hmac_nonce.len(), 32);
    }

    #[test]
    fn test_verify_valid_signature() {
        let signer = HmacSigner::new("my_secret".into(), "my_client".into());
        let headers = signer.sign("GET", "/v1/balance", "").unwrap();

        let valid = signer
            .verify(
                "GET",
                "/v1/balance",
                &headers.x_hmac_timestamp,
                &headers.x_hmac_nonce,
                "",
                &headers.x_hmac_signature,
            )
            .unwrap();

        assert!(valid);
    }

    #[test]
    fn test_verify_invalid_signature() {
        let signer = HmacSigner::new("my_secret".into(), "my_client".into());

        let valid = signer
            .verify("GET", "/v1/balance", "2026-01-01T00:00:00Z", "nonce123", "", "aW52YWxpZA==")
            .unwrap();

        assert!(!valid);
    }

    #[test]
    fn test_nonce_uniqueness() {
        let n1 = generate_nonce();
        let n2 = generate_nonce();
        assert_ne!(n1, n2);
        assert_eq!(n1.len(), 32);
    }
}
