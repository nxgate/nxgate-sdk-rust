use nxgate::{NXGateClient, NXGateError, PixGenerateRequest, PixKeyType, PixWithdrawRequest};

#[test]
fn test_builder_missing_client_id() {
    let result = NXGateClient::builder()
        .client_secret("secret")
        .build();

    assert!(result.is_err());
    match result.unwrap_err() {
        NXGateError::Config(msg) => assert!(msg.contains("client_id")),
        other => panic!("Esperava Config error, recebeu: {:?}", other),
    }
}

#[test]
fn test_builder_missing_client_secret() {
    let result = NXGateClient::builder()
        .client_id("my_id")
        .build();

    assert!(result.is_err());
    match result.unwrap_err() {
        NXGateError::Config(msg) => assert!(msg.contains("client_secret")),
        other => panic!("Esperava Config error, recebeu: {:?}", other),
    }
}

#[test]
fn test_builder_success() {
    let result = NXGateClient::builder()
        .client_id("my_id")
        .client_secret("my_secret")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_builder_with_hmac() {
    let result = NXGateClient::builder()
        .client_id("my_id")
        .client_secret("my_secret")
        .hmac_secret("hmac_key")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_builder_with_custom_base_url() {
    let result = NXGateClient::builder()
        .client_id("my_id")
        .client_secret("my_secret")
        .base_url("https://sandbox.nxgate.com.br")
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_pix_generate_request_default() {
    let req = PixGenerateRequest {
        valor: 99.90,
        nome_pagador: "Maria".into(),
        documento_pagador: "12345678901".into(),
        ..Default::default()
    };

    assert_eq!(req.valor, 99.90);
    assert_eq!(req.nome_pagador, "Maria");
    assert!(req.webhook.is_none());
    assert!(req.split_users.is_none());
}

#[test]
fn test_pix_generate_request_serialization() {
    let req = PixGenerateRequest {
        valor: 100.0,
        nome_pagador: "João".into(),
        documento_pagador: "12345678901".into(),
        descricao: Some("Teste".into()),
        ..Default::default()
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["valor"], 100.0);
    assert_eq!(json["nome_pagador"], "João");
    assert_eq!(json["descricao"], "Teste");
    // Campos None não devem aparecer no JSON
    assert!(json.get("webhook").is_none());
    assert!(json.get("magic_id").is_none());
}

#[test]
fn test_pix_withdraw_request_serialization() {
    let req = PixWithdrawRequest {
        valor: 50.0,
        chave_pix: "joao@email.com".into(),
        tipo_chave: PixKeyType::Email,
        ..Default::default()
    };

    let json = serde_json::to_value(&req).unwrap();
    assert_eq!(json["valor"], 50.0);
    assert_eq!(json["chave_pix"], "joao@email.com");
    assert_eq!(json["tipo_chave"], "Email");
    assert!(json.get("documento").is_none());
}

#[test]
fn test_pix_key_type_display() {
    assert_eq!(PixKeyType::Cpf.to_string(), "Cpf");
    assert_eq!(PixKeyType::Cnpj.to_string(), "Cnpj");
    assert_eq!(PixKeyType::Phone.to_string(), "Phone");
    assert_eq!(PixKeyType::Email.to_string(), "Email");
    assert_eq!(PixKeyType::Random.to_string(), "Random");
}

#[test]
fn test_transaction_type_display() {
    use nxgate::TransactionType;
    assert_eq!(TransactionType::CashIn.to_string(), "cash-in");
    assert_eq!(TransactionType::CashOut.to_string(), "cash-out");
}

#[tokio::test]
async fn test_client_auth_failure_against_invalid_server() {
    let client = NXGateClient::builder()
        .client_id("invalid")
        .client_secret("invalid")
        .base_url("http://127.0.0.1:1") // porta inválida
        .build()
        .unwrap();

    let result = client.get_balance().await;
    assert!(result.is_err());
}
