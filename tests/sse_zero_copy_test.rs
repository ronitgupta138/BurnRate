use burnrate::streaming::extract_sse_data_lines;

#[test]
fn test_extract_sse_data_lines_zero_copy() {
    let chunk = b"event: message\ndata: {\"usage\":{\"total_tokens\":42}}\n\nevent: ping\ndata: [DONE]\n";
    let lines = extract_sse_data_lines(chunk);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], b"{\"usage\":{\"total_tokens\":42}}");
    assert_eq!(lines[1], b"[DONE]");
}

#[test]
fn test_extract_sse_empty_and_crlf() {
    let chunk = b"data: hello\r\ndata: world\r\n";
    let lines = extract_sse_data_lines(chunk);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], b"hello");
    assert_eq!(lines[1], b"world");
}
