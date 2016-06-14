
pub fn log2_upper(n: u64) -> u32 {
    n.next_power_of_two().trailing_zeros()
}

#[test]
fn test_log2_upper() {
    assert_eq!(log2_upper(6), 3);
}
