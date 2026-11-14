use burnrate::pricing::PricingRegistry;

#[test]
fn test_frontier_models_pricing_lookup() {
    let o3 = PricingRegistry::lookup("o3-mini");
    assert_eq!(o3.model_id, "o3-mini");
    assert!((o3.input_per_million - 1.10).abs() < 1e-4);
    assert!((o3.output_per_million - 4.40).abs() < 1e-4);

    let r1 = PricingRegistry::lookup("deepseek-r1");
    assert_eq!(r1.model_id, "deepseek-r1");
    assert!((r1.input_per_million - 0.55).abs() < 1e-4);

    let local = PricingRegistry::lookup("qwen-2.5-coder");
    assert_eq!(local.model_id, "local-model");
    assert_eq!(local.input_per_million, 0.0);
    assert_eq!(local.output_per_million, 0.0);
}

#[test]
fn test_is_local_classification() {
    assert!(PricingRegistry::is_local("ollama/qwen2.5-coder"));
    assert!(PricingRegistry::is_local("local-model"));
    assert!(!PricingRegistry::is_local("claude-3-7-sonnet"));
    assert!(!PricingRegistry::is_local("gpt-4o"));
}

#[test]
fn test_list_supported_models() {
    let models = PricingRegistry::list_supported_models();
    assert!(models.contains(&"o3-mini"));
    assert!(models.contains(&"deepseek-r1"));
    assert!(models.contains(&"claude-sonnet"));
}
