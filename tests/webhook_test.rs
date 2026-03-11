use nxgate::{parse_webhook, WebhookEvent};

#[test]
fn test_parse_cashin_paid() {
    let payload = br#"{
        "type": "QR_CODE_COPY_AND_PASTE_PAID",
        "data": {
            "amount": 150.75,
            "status": "PAID",
            "worked": true,
            "tag": "pedido_123",
            "tx_id": "TX001",
            "end_to_end": "E2E001"
        }
    }"#;

    let event = parse_webhook(payload).unwrap();

    match event {
        WebhookEvent::CashInPaid(wh) => {
            assert_eq!(wh.event_type, "QR_CODE_COPY_AND_PASTE_PAID");
            assert_eq!(wh.data.amount, 150.75);
            assert_eq!(wh.data.status, "PAID");
            assert!(wh.data.worked);
            assert_eq!(wh.data.tag.as_deref(), Some("pedido_123"));
            assert_eq!(wh.data.tx_id.as_deref(), Some("TX001"));
            assert_eq!(wh.data.end_to_end.as_deref(), Some("E2E001"));
        }
        _ => panic!("Esperava CashInPaid"),
    }
}

#[test]
fn test_parse_cashin_refunded() {
    let payload = br#"{
        "type": "QR_CODE_COPY_AND_PASTE_REFUNDED",
        "data": {
            "amount": 50.00,
            "status": "REFUNDED",
            "worked": true
        }
    }"#;

    let event = parse_webhook(payload).unwrap();

    match event {
        WebhookEvent::CashInRefunded(wh) => {
            assert_eq!(wh.event_type, "QR_CODE_COPY_AND_PASTE_REFUNDED");
            assert_eq!(wh.data.amount, 50.0);
            assert_eq!(wh.data.status, "REFUNDED");
        }
        _ => panic!("Esperava CashInRefunded"),
    }
}

#[test]
fn test_parse_cashout_success() {
    let payload = br#"{
        "type": "PIX_CASHOUT_SUCCESS",
        "worked": true,
        "status": "COMPLETED",
        "id_transaction": "TXN_999",
        "amount": 200.00,
        "key": "joao@email.com"
    }"#;

    let event = parse_webhook(payload).unwrap();

    match event {
        WebhookEvent::CashOutSuccess(wh) => {
            assert_eq!(wh.event_type, "PIX_CASHOUT_SUCCESS");
            assert!(wh.worked);
            assert_eq!(wh.status, "COMPLETED");
            assert_eq!(wh.id_transaction, "TXN_999");
            assert_eq!(wh.amount, 200.0);
            assert_eq!(wh.key, "joao@email.com");
        }
        _ => panic!("Esperava CashOutSuccess"),
    }
}

#[test]
fn test_parse_cashout_error() {
    let payload = br#"{
        "type": "PIX_CASHOUT_ERROR",
        "worked": false,
        "status": "ERROR",
        "id_transaction": "TXN_500",
        "amount": 75.00,
        "key": "12345678901"
    }"#;

    let event = parse_webhook(payload).unwrap();

    match event {
        WebhookEvent::CashOutError(wh) => {
            assert_eq!(wh.event_type, "PIX_CASHOUT_ERROR");
            assert!(!wh.worked);
            assert_eq!(wh.status, "ERROR");
        }
        _ => panic!("Esperava CashOutError"),
    }
}

#[test]
fn test_parse_cashout_refunded() {
    let payload = br#"{
        "type": "PIX_CASHOUT_REFUNDED",
        "worked": true,
        "status": "REFUNDED",
        "id_transaction": "TXN_777",
        "amount": 300.00,
        "key": "+5511999999999"
    }"#;

    let event = parse_webhook(payload).unwrap();

    match event {
        WebhookEvent::CashOutRefunded(wh) => {
            assert_eq!(wh.event_type, "PIX_CASHOUT_REFUNDED");
            assert_eq!(wh.amount, 300.0);
        }
        _ => panic!("Esperava CashOutRefunded"),
    }
}

#[test]
fn test_parse_unknown_event_type() {
    let payload = br#"{"type": "UNKNOWN_EVENT"}"#;
    let result = parse_webhook(payload);
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_json() {
    let payload = b"not json at all";
    let result = parse_webhook(payload);
    assert!(result.is_err());
}

#[test]
fn test_parse_missing_type_field() {
    let payload = br#"{"data": {"amount": 100}}"#;
    let result = parse_webhook(payload);
    assert!(result.is_err());
}

#[test]
fn test_parse_cashin_minimal_data() {
    let payload = br#"{
        "type": "QR_CODE_COPY_AND_PASTE_PAID",
        "data": {
            "amount": 10.0,
            "status": "PAID",
            "worked": false
        }
    }"#;

    let event = parse_webhook(payload).unwrap();

    match event {
        WebhookEvent::CashInPaid(wh) => {
            assert!(wh.data.tag.is_none());
            assert!(wh.data.tx_id.is_none());
            assert!(wh.data.end_to_end.is_none());
        }
        _ => panic!("Esperava CashInPaid"),
    }
}
