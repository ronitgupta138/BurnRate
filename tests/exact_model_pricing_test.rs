use burnrate::pricing::PricingRegistry;

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn opus_4_5_is_not_priced_as_opus_4() {
    // Opus 4.5 lists at $5 / $25, not the $15 / $75 of Opus 4 and 4.1.
    let p = PricingRegistry::lookup("claude-opus-4-5-20251101");
    assert!(close(p.input_per_million, 5.0), "{p:?}");
    assert!(close(p.output_per_million, 25.0), "{p:?}");
    assert!(close(p.cache_read_per_million, 0.5), "{p:?}");
    assert!(close(p.cache_write_per_million, 6.25), "{p:?}");
    // A 1M in / 100k out turn: $7.50, where the family table charged $22.50.
    assert!(close(p.calculate_cost(1_000_000, 100_000, 0, 0), 7.5));
}

#[test]
fn haiku_4_5_is_not_priced_as_haiku_3_5() {
    let p = PricingRegistry::lookup("claude-haiku-4-5");
    assert!(close(p.input_per_million, 1.0), "{p:?}");
    assert!(close(p.output_per_million, 5.0), "{p:?}");
}

#[test]
fn gpt_4_1_and_gpt_5_get_their_own_prices() {
    let g41 = PricingRegistry::lookup("gpt-4.1");
    assert!(
        close(g41.input_per_million, 2.0) && close(g41.output_per_million, 8.0),
        "{g41:?}"
    );

    let g5 = PricingRegistry::lookup("openai/gpt-5");
    assert!(
        close(g5.input_per_million, 1.25) && close(g5.output_per_million, 10.0),
        "{g5:?}"
    );
}

#[test]
fn gemini_is_not_priced_as_sonnet() {
    let p = PricingRegistry::lookup("models/gemini-2.5-pro");
    assert_eq!(p.model_id, "gemini-2.5-pro");
    assert!(close(p.input_per_million, 1.25), "{p:?}");
}

#[test]
fn hosted_llama_is_not_free() {
    // Llama on Bedrock is billed; only a local endpoint is free.
    let p = PricingRegistry::lookup("us.meta.llama3-3-70b-instruct-v1:0");
    assert!(close(p.input_per_million, 0.72), "{p:?}");
    assert_eq!(
        PricingRegistry::lookup("ollama/llama3.3").input_per_million,
        0.0
    );
    assert_eq!(
        PricingRegistry::lookup("qwen-2.5-coder").model_id,
        "local-model"
    );
}

#[test]
fn family_names_still_use_the_family_table() {
    assert_eq!(
        PricingRegistry::lookup("claude-sonnet").model_id,
        "claude-sonnet"
    );
    assert_eq!(
        PricingRegistry::lookup("claude-haiku").model_id,
        "claude-haiku"
    );
}
