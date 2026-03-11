# NXGATE PIX SDK para Rust

SDK oficial em Rust para integração com a API **NXGATE PIX**. Suporta operações de cash-in (geração de QR Code), cash-out (saque PIX), consulta de saldo, consulta de transações e processamento de webhooks.

## Funcionalidades

- **Autenticação OAuth2** com renovação automática de token
- **Assinatura HMAC-SHA256** opcional para segurança adicional
- **Cash-in**: Geração de cobrança PIX com QR Code
- **Cash-out**: Saque PIX para qualquer chave
- **Saldo**: Consulta de saldo disponível
- **Transações**: Consulta de transações por ID
- **Webhooks**: Parser tipado para eventos de webhook
- **Retry automático**: Tentativas com backoff em caso de HTTP 503
- **Async/await**: Totalmente assíncrono com Tokio

## Instalação

Adicione ao seu `Cargo.toml`:

```toml
[dependencies]
nxgate = "1.0"
tokio = { version = "1", features = ["full"] }
```

## Início Rápido

```rust
use nxgate::{NXGateClient, PixGenerateRequest, PixWithdrawRequest, PixKeyType};

#[tokio::main]
async fn main() -> Result<(), nxgate::NXGateError> {
    // Criar o cliente
    let client = NXGateClient::builder()
        .client_id("nxgate_xxx")
        .client_secret("secret")
        .hmac_secret("optional") // opcional
        .build()?;

    // Cash-in: Gerar cobrança PIX
    let charge = client.pix_generate(PixGenerateRequest {
        valor: 100.0,
        nome_pagador: "João da Silva".into(),
        documento_pagador: "12345678901".into(),
        webhook: Some("https://meusite.com/webhook".into()),
        ..Default::default()
    }).await?;

    println!("QR Code: {}", charge.payment_code);
    println!("Transaction ID: {}", charge.id_transaction);

    // Cash-out: Saque PIX
    let withdrawal = client.pix_withdraw(PixWithdrawRequest {
        valor: 50.0,
        chave_pix: "joao@email.com".into(),
        tipo_chave: PixKeyType::Email,
        ..Default::default()
    }).await?;

    println!("Saque: {} - {}", withdrawal.status, withdrawal.message);

    // Consultar saldo
    let balance = client.get_balance().await?;
    println!("Saldo disponível: R$ {:.2}", balance.available);

    // Consultar transação
    use nxgate::TransactionType;
    let tx = client.get_transaction(TransactionType::CashIn, "TX001").await?;
    println!("Transação: {} - {}", tx.id_transaction, tx.status);

    Ok(())
}
```

## Processamento de Webhooks

O SDK inclui um parser tipado para eventos de webhook recebidos da NXGATE:

```rust
use nxgate::{parse_webhook, WebhookEvent};

fn handle_webhook(body: &[u8]) -> Result<(), nxgate::NXGateError> {
    match parse_webhook(body)? {
        WebhookEvent::CashInPaid(event) => {
            println!("PIX recebido! Valor: R$ {:.2}", event.data.amount);
            println!("Status: {}", event.data.status);
            if let Some(tx_id) = &event.data.tx_id {
                println!("TX ID: {}", tx_id);
            }
        }
        WebhookEvent::CashInRefunded(event) => {
            println!("PIX estornado! Valor: R$ {:.2}", event.data.amount);
        }
        WebhookEvent::CashOutSuccess(event) => {
            println!("Saque realizado! ID: {}", event.id_transaction);
        }
        WebhookEvent::CashOutError(event) => {
            println!("Erro no saque! ID: {}", event.id_transaction);
        }
        WebhookEvent::CashOutRefunded(event) => {
            println!("Saque estornado! ID: {}", event.id_transaction);
        }
    }
    Ok(())
}
```

### Tipos de Eventos

| Evento | Tipo | Descrição |
|--------|------|-----------|
| `QR_CODE_COPY_AND_PASTE_PAID` | Cash-in | QR Code PIX foi pago |
| `QR_CODE_COPY_AND_PASTE_REFUNDED` | Cash-in | PIX recebido foi estornado |
| `PIX_CASHOUT_SUCCESS` | Cash-out | Saque PIX realizado com sucesso |
| `PIX_CASHOUT_ERROR` | Cash-out | Erro ao processar saque PIX |
| `PIX_CASHOUT_REFUNDED` | Cash-out | Saque PIX foi estornado |

## Assinatura HMAC

Quando o `hmac_secret` é fornecido no builder, todas as requisições são automaticamente assinadas com HMAC-SHA256. Os seguintes headers são adicionados:

| Header | Descrição |
|--------|-----------|
| `X-Client-ID` | Identificador do cliente |
| `X-HMAC-Signature` | Assinatura HMAC-SHA256 em Base64 |
| `X-HMAC-Timestamp` | Timestamp ISO 8601 da requisição |
| `X-HMAC-Nonce` | String única por requisição |

A mensagem assinada segue o formato:

```
METHOD\nPATH\nTIMESTAMP\nNONCE\nBODY
```

Você também pode usar o `HmacSigner` diretamente para verificar assinaturas de webhook:

```rust
use nxgate::hmac::HmacSigner;

let signer = HmacSigner::new("meu_secret".into(), "meu_client_id".into());

let valido = signer.verify(
    "POST",
    "/webhook",
    "2026-01-01T00:00:00Z",
    "nonce_recebido",
    r#"{"type":"PIX_CASHOUT_SUCCESS",...}"#,
    "assinatura_recebida_base64",
)?;
```

## Split de Pagamento

O SDK suporta split de pagamento na geração de cobranças PIX:

```rust
use nxgate::{PixGenerateRequest, SplitUser};

let request = PixGenerateRequest {
    valor: 100.0,
    nome_pagador: "João".into(),
    documento_pagador: "12345678901".into(),
    split_users: Some(vec![
        SplitUser { username: "loja_a".into(), percentage: 70.0 },
        SplitUser { username: "loja_b".into(), percentage: 30.0 },
    ]),
    ..Default::default()
};
```

## Tipos de Chave PIX

```rust
use nxgate::PixKeyType;

let cpf = PixKeyType::Cpf;        // CPF
let cnpj = PixKeyType::Cnpj;      // CNPJ
let phone = PixKeyType::Phone;    // Telefone
let email = PixKeyType::Email;    // E-mail
let random = PixKeyType::Random;  // Chave aleatória
```

## Tratamento de Erros

O SDK utiliza o enum `NXGateError` para todos os erros:

```rust
use nxgate::NXGateError;

match resultado {
    Err(NXGateError::Auth(msg)) => println!("Erro de autenticação: {}", msg),
    Err(NXGateError::Api { status, body }) => println!("Erro HTTP {}: {}", status, body),
    Err(NXGateError::Http(e)) => println!("Erro de rede: {}", e),
    Err(NXGateError::MaxRetries { attempts, .. }) => println!("Falhou após {} tentativas", attempts),
    Err(e) => println!("Outro erro: {}", e),
    Ok(_) => println!("Sucesso!"),
}
```

## Retry Automático

O SDK automaticamente retenta requisições nos seguintes casos:

- **HTTP 503** (Service Unavailable): até 2 tentativas adicionais com backoff exponencial
- **HTTP 401** (Unauthorized): invalida o token e tenta nova autenticação

## Configuração Avançada

```rust
let client = NXGateClient::builder()
    .client_id("nxgate_xxx")
    .client_secret("secret")
    .hmac_secret("hmac_key")                      // Assinatura HMAC (opcional)
    .base_url("https://sandbox.nxgate.com.br")    // URL customizada (opcional)
    .build()?;
```

## Licença

Este projeto é distribuído sob a licença MIT. Veja o arquivo [LICENSE](LICENSE) para mais detalhes.
