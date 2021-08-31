
use num::BigUint;

macro_rules! debug {
    ($($args:tt)*) => {
        if cfg!(test) {println!($($args)*);}
    }
}

/*
/// Return ceiling(log_2 (n))
pub fn log2_upper(n: u64) -> u32 {
    n.next_power_of_two().trailing_zeros()
}
*/

/// Return ceiling(log_2 (n))
pub fn log2_upper_bigu(n: &BigUint) -> u64 {
    (n-1u32).bits()
}

pub fn make_2x2<A,F>(mut func: F) -> [[A; 2]; 2]
    where F : FnMut(usize, usize) -> A {
    
    [[func(0, 0), func(0, 1)], [func(1, 0), func(1, 1)]]
}

pub fn try_make_2x2<A,E,F>(mut func: F) -> Result<[[A; 2]; 2], E>
    where F : FnMut(usize, usize) -> Result<A,E> {
    
    Ok([[func(0, 0)?, func(0, 1)?],
        [func(1, 0)?, func(1, 1)?]])
}

pub fn make_3x3<A,F>(mut func: F) -> [[A; 3]; 3]
    where F : FnMut(usize, usize) -> A {

    [[func(0, 0), func(0, 1), func(0, 2)],
     [func(1, 0), func(1, 1), func(1, 2)],
     [func(2, 0), func(2, 1), func(2, 2)]]
}

#[test]
fn test_log2_upper() {
    use num::FromPrimitive;
    assert_eq!(log2_upper_bigu(&BigUint::from_u32(6).unwrap()), 3);
    assert_eq!(log2_upper_bigu(&BigUint::from_u32(8).unwrap()), 3);
}
