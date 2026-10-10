use burnrate::streaming::StreamingTokenCounter;

#[test]
fn test_streaming_token_counter_ingestion() {
    let counter = StreamingTokenCounter::new();
    counter.ingest_chunk(b"data: {\"choices\": [{\"delta\": {\"content\": \"hello\"}}]}\n\n");
    counter.ingest_chunk(b"data: {\"choices\": [{\"delta\": {\"content\": \" world\"}}]}\n\n");

    assert_eq!(counter.total_output_chunks.load(std::sync::atomic::Ordering::Relaxed), 2);
    assert!(counter.approximate_tokens() > 0);
}
