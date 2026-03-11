use nxgate::hmac::HmacSigner;

#[test]
fn test_sign_and_verify_roundtrip() {
    let signer = HmacSigner::new("super_secret_key".into(), "client_abc".into());

    let headers = signer.sign("POST", "/pix/gerar", r#"{"valor":250.00}"#).unwrap();

    assert_eq!(headers.x_client_id, "client_abc");
    assert!(!headers.x_hmac_signature.is_empty());
    assert!(!headers.x_hmac_timestamp.is_empty());
    assert_eq!(headers.x_hmac_nonce.len(), 32);

    // Verificar que a assinatura é válida
    let valid = signer
        .verify(
            "POST",
            "/pix/gerar",
            &headers.x_hmac_timestamp,
            &headers.x_hmac_nonce,
            r#"{"valor":250.00}"#,
            &headers.x_hmac_signature,
        )
        .unwrap();

    assert!(valid);
}

#[test]
fn test_tampered_body_fails_verification() {
    let signer = HmacSigner::new("key123".into(), "client123".into());

    let headers = signer.sign("POST", "/pix/sacar", r#"{"valor":100}"#).unwrap();

    // Alterar o body deve invalidar a assinatura
    let valid = signer
        .verify(
            "POST",
            "/pix/sacar",
            &headers.x_hmac_timestamp,
            &headers.x_hmac_nonce,
            r#"{"valor":999}"#, // body diferente
            &headers.x_hmac_signature,
        )
        .unwrap();

    assert!(!valid);
}

#[test]
fn test_different_method_fails_verification() {
    let signer = HmacSigner::new("key123".into(), "client123".into());

    let headers = signer.sign("POST", "/pix/gerar", "{}").unwrap();

    let valid = signer
        .verify(
            "GET", // método diferente
            "/pix/gerar",
            &headers.x_hmac_timestamp,
            &headers.x_hmac_nonce,
            "{}",
            &headers.x_hmac_signature,
        )
        .unwrap();

    assert!(!valid);
}

#[test]
fn test_different_path_fails_verification() {
    let signer = HmacSigner::new("key123".into(), "client123".into());

    let headers = signer.sign("GET", "/v1/balance", "").unwrap();

    let valid = signer
        .verify(
            "GET",
            "/v1/transactions", // path diferente
            &headers.x_hmac_timestamp,
            &headers.x_hmac_nonce,
            "",
            &headers.x_hmac_signature,
        )
        .unwrap();

    assert!(!valid);
}

#[test]
fn test_wrong_secret_fails_verification() {
    let signer1 = HmacSigner::new("secret_a".into(), "client".into());
    let signer2 = HmacSigner::new("secret_b".into(), "client".into());

    let headers = signer1.sign("GET", "/v1/balance", "").unwrap();

    let valid = signer2
        .verify(
            "GET",
            "/v1/balance",
            &headers.x_hmac_timestamp,
            &headers.x_hmac_nonce,
            "",
            &headers.x_hmac_signature,
        )
        .unwrap();

    assert!(!valid);
}

#[test]
fn test_empty_body_sign() {
    let signer = HmacSigner::new("key".into(), "client".into());
    let headers = signer.sign("GET", "/v1/balance", "").unwrap();
    assert!(!headers.x_hmac_signature.is_empty());
}

#[test]
fn test_nonce_is_unique_across_calls() {
    let signer = HmacSigner::new("key".into(), "client".into());

    let h1 = signer.sign("GET", "/v1/balance", "").unwrap();
    let h2 = signer.sign("GET", "/v1/balance", "").unwrap();

    assert_ne!(h1.x_hmac_nonce, h2.x_hmac_nonce);
    assert_ne!(h1.x_hmac_signature, h2.x_hmac_signature);
}
