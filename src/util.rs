
pub fn log2_upper(n: u64) -> u32 {
    n.next_power_of_two().trailing_zeros()
}
