extern crate clap;

extern crate hlife;

use std::fs::File;
use std::io::Read;
use std::process::exit;

use clap::{Arg, App};

use hlife::Hashlife;
use hlife::global::Pattern;
use hlife::format::write::format_rle;

fn main() {
    let matches = App::new("Itai's Hashlife")
            .arg(Arg::with_name("INPUT-FILE")
                    .required(true)
                    .index(1))
            .arg(Arg::with_name("GENERATIONS")
                    .required(true)
                    .index(2))
            .get_matches();

    let filename = matches.value_of("INPUT-FILE").expect("internal clap error");
    let gens_string = matches.value_of("GENERATIONS").expect("internal clap\
        error");
    let gens = u64::from_str_radix(&gens_string, 10).unwrap_or_else(|_| {
        println!(
            "Error: Second argument gens must be a nonnegative integer: {}",
            &gens_string);
        exit(1);
    });
    let mut in_file = File::open(&filename).unwrap_or_else(|_| {
        println!("Cannot open file {}", &filename);
        exit(1);
    });
    let mut rle_buf = Vec::new();
    in_file.read_to_end(&mut rle_buf).unwrap_or_else(|_| {
        println!("Error reading file {}", &filename);
        exit(1);
    });

    Hashlife::with_new(|hl| {
        let block = hl.block_from_bytes(&rle_buf).unwrap_or_else(|_| {
            println!("Badly formatted RLE in {}", &filename);
            exit(1);
        });
        let mut pattern = Pattern::new(block);
        pattern.step(gens);
        print!("{}", format_rle(&pattern.block()));
    });
}
