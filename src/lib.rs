//! # NXGATE PIX SDK
//!
//! SDK oficial para integração com a API NXGATE PIX.
//!
//! Suporta operações de cash-in (geração de QR Code), cash-out (saque PIX),
//! consulta de saldo, consulta de transações e processamento de webhooks.
//!
//! ## Exemplo rápido
//!
//! ```rust,no_run
//! use nxgate::{NXGateClient, PixGenerateRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), nxgate::NXGateError> {
//!     let client = NXGateClient::builder()
//!         .client_id("nxgate_xxx")
//!         .client_secret("secret")
//!         .build()?;
//!
//!     let charge = client.pix_generate(PixGenerateRequest {
//!         valor: 100.0,
//!         nome_pagador: "João da Silva".into(),
//!         documento_pagador: "12345678901".into(),
//!         ..Default::default()
//!     }).await?;
//!
//!     println!("PIX code: {}", charge.payment_code);
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod error;
pub mod hmac;
pub mod types;
pub mod webhook;

// Re-exports para uso direto
pub use client::{NXGateClient, NXGateClientBuilder};
pub use error::NXGateError;
pub use types::*;
pub use webhook::{
    parse_webhook, CashInWebhook, CashInWebhookData, CashOutWebhook, WebhookEvent,
};
