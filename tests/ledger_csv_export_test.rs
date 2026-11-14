use burnrate::ledger::format_record_csv;

#[test]
fn test_format_record_csv_output() {
    let row = format_record_csv("sess_01", "claude-sonnet", 1500, 0.004500);
    assert_eq!(row, "sess_01,claude-sonnet,1500,0.004500\n");
}
