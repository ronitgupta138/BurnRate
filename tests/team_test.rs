use burnrate::team::TeamBudgetPool;

#[test]
fn test_team_budget_pool_multi_agent_partitioning() {
    let pool = TeamBudgetPool::new(10.00); // $10 total team pool
    pool.allocate_agent("agent-coder", 2.00); // $2.00 allocated
    pool.allocate_agent("agent-reviewer", 1.00); // $1.00 allocated

    // 1. Spend within quota
    let exhausted1 = pool.record_agent_spend("agent-coder", 0.80);
    assert!(!exhausted1);

    // 2. Breach quota
    let exhausted2 = pool.record_agent_spend("agent-coder", 1.50);
    assert!(exhausted2);

    let quota = pool.get_agent_quota("agent-coder").unwrap();
    assert!(quota.is_exhausted);
    assert_eq!(quota.turns_count, 2);
}
