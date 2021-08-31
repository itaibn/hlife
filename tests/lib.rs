extern crate hlife;

use std::io::{self, Read};
use std::fs::File;

use hlife::Hashlife;
#[cfg(not(feature = "4x4_leaf"))]
use hlife::global::Pattern;

#[cfg(not(feature = "4x4_leaf"))]
fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(not(feature = "4x4_leaf"))]
#[ignore]
#[test]
fn test_global_instances() {
    const TEST_INSTANCES: usize = 2;
    const TEST_TIMES: [u64; TEST_INSTANCES] = [1, 175];

//    println!("Current dir: {}",
//        fs::canonicalize(".").unwrap().to_str().unwrap());

    Hashlife::with_new(|hl| {
        for n in 0..TEST_INSTANCES {
            let in_bytes = read_file(&format!("instances/in{:03}.rle",
                n)).unwrap();
            let out_bytes = read_file(&format!("instances/out{:03}.rle",
                n)).unwrap();
            let in_block = hl.block_from_bytes(&in_bytes).unwrap();
            let out_block = hl.block_from_bytes(&out_bytes).unwrap();
            let mut in_pattern = Pattern::new(in_block);
            let out_pattern = Pattern::new(out_block);
            in_pattern.step(TEST_TIMES[n]);
            assert_eq!(in_pattern, out_pattern);
        }
    });
}
