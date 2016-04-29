extern crate hlife;

use hlife::Hashlife;
use hlife::global::Pattern;
use hlife::format::write::format_rle;

use std::env::args;
use std::fs::File;
use std::io::Read;
use std::process::exit;

fn main() {
    let args: Vec<_> = args().collect();
    if args.len() != 3 {
        println!("{} input.rle gens", &args[0]);
        exit(1);
    }
    let gens = usize::from_str_radix(&args[2], 10).unwrap_or_else(|_| {
        println!(
            "Error: Second argument gens must be a nonnegative integer: {}",
            &args[2]);
        exit(1);
    });
    let mut in_file = File::open(&args[1]).unwrap_or_else(|_| {
        println!("Cannot open file {}", &args[1]);
        exit(1);
    });
    let mut rle_buf = Vec::new();
    in_file.read_to_end(&mut rle_buf).unwrap_or_else(|_| {
        println!("Error reading file {}", &args[1]);
        exit(1);
    });

    Hashlife::with_new(|hl| {
        let block = hl.block_from_bytes(&rle_buf).unwrap_or_else(|_| {
            println!("Badly formatted RLE in {}", &args[1]);
            exit(1);
        });
        let mut pattern = Pattern::new(&hl, block);
        pattern.step(gens);
        print!("{}", format_rle(&pattern.block()));
    });
}
