/// Sanitizes bearer tokens from raw HTTP Authorization strings.
pub fn sanitize_auth_header(header: &str) -> String {
    let trimmed = header.trim();
    if let Some(token) = trimmed.strip_prefix("Bearer ") {
        if token.len() <= 8 {
            "Bearer [REDACTED]".to_string()
        } else {
            let prefix = &token[..3];
            let suffix = &token[token.len() - 4..];
            format!("Bearer {}...{}", prefix, suffix)
        }
    } else {
        "[REDACTED]".to_string()
    }
}
