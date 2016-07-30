
macro_rules! debug {
    ($($args:tt)*) => {
        if cfg!(test) {println!($($args)*);}
    }
}

pub fn log2_upper(n: u64) -> u32 {
    n.next_power_of_two().trailing_zeros()
}

// TODO: Incorporate into rest of this code
pub fn make_2x2<A,F>(mut func: F) -> [[A; 2]; 2]
    where F : FnMut(usize, usize) -> A {
    
    [[func(0, 0), func(0, 1)], [func(1, 0), func(1, 1)]]
}

pub fn make_3x3<A,F>(mut func: F) -> [[A; 3]; 3]
    where F : FnMut(usize, usize) -> A {

    [[func(0, 0), func(0, 1), func(0, 2)],
     [func(1, 0), func(1, 1), func(1, 2)],
     [func(2, 0), func(2, 1), func(2, 2)]]
}

#[test]
fn test_log2_upper() {
    assert_eq!(log2_upper(6), 3);
}
