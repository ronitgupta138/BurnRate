use burnrate::secret_guard::mask_credential_token;

#[test]
fn test_mask_credential_token_standard() {
    let key = "sk-ant-api03-abcdef1234567890-xyz9";
    let masked = mask_credential_token(key);
    assert_eq!(masked, "sk-...xyz9");
}

#[test]
fn test_mask_credential_token_short() {
    let short_key = "secret";
    assert_eq!(mask_credential_token(short_key), "[REDACTED]");
}
