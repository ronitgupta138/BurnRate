use burnrate::token_sanitizer::sanitize_auth_header;

#[test]
fn test_sanitize_bearer_token() {
    let raw = "Bearer sk-ant-1234567890abcdef";
    let sanitized = sanitize_auth_header(raw);
    assert_eq!(sanitized, "Bearer sk-...cdef");

    let short = "Bearer secret";
    assert_eq!(sanitize_auth_header(short), "Bearer [REDACTED]");
}
