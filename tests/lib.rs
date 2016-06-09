extern crate hlife;

use std::io::{self, Read};
use std::fs::{self, File};

use hlife::Hashlife;
use hlife::global::Pattern;

fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut file = try!(File::open(path));
    try!(file.read_to_end(&mut buf));
    Ok(buf)
}

#[test]
fn test_global_instances() {
    const TEST_INSTANCES: usize = 1;
    const TEST_TIMES: [usize; TEST_INSTANCES] = [1];

//    println!("Current dir: {}",
//        fs::canonicalize(".").unwrap().to_str().unwrap());

    Hashlife::with_new(|hl| {
        for n in 0..TEST_INSTANCES {
            let in_bytes = read_file(&format!("tests/in{:03}.rle", n)).unwrap();
            let out_bytes = read_file(&format!("tests/out{:03}.rle",
                n)).unwrap();
            let in_block = hl.block_from_bytes(&in_bytes).unwrap();
            let out_block = hl.block_from_bytes(&out_bytes).unwrap();
            let mut in_pattern = Pattern::new(&hl, in_block);
            let out_pattern = Pattern::new(&hl, out_block);
            in_pattern.step(TEST_TIMES[n]);
            assert_eq!(in_pattern, out_pattern);
        }
    });
}
