#![feature(test)]

extern crate hlife;
extern crate test;

use std::io::{self, Read};
use std::fs::File;

use test::Bencher;

use hlife::Hashlife;
#[cfg(not(features = "4x4_leaf"))]
use hlife::global::Pattern;

fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(not(features = "4x4_leaf"))]
#[ignore]
#[bench]
fn bench_global_instances(b: &mut Bencher) {
    const TEST_INSTANCES: usize = 2;
    const TEST_TIMES: [u64; TEST_INSTANCES] = [1, 175];

//    println!("Current dir: {}",
//        fs::canonicalize(".").unwrap().to_str().unwrap());

    for n in 0..TEST_INSTANCES {
        let in_bytes = read_file(&format!("instances/in{:03}.rle", n)).unwrap();
        let out_bytes = read_file(&format!("instances/out{:03}.rle",
            n)).unwrap();
        b.iter(|| test_in_out_pair(&in_bytes, &out_bytes, TEST_TIMES[n]));
    }
}

#[cfg(not(features = "4x4_leaf"))]
fn test_in_out_pair(in_rle: &[u8], out_rle: &[u8], steps: u64) {
    Hashlife::with_new(|hl| {
        let in_block = hl.block_from_bytes(&in_rle).unwrap();
        let out_block = hl.block_from_bytes(&out_rle).unwrap();
        let mut in_pattern = Pattern::new(in_block);
        let out_pattern = Pattern::new(out_block);
        in_pattern.step(steps);
        assert_eq!(in_pattern, out_pattern);
    });
}
