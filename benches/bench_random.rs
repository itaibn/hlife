#![feature(test)]

extern crate hlife;
extern crate rand;
extern crate test;

use hlife::Hashlife;
use rand::thread_rng;
use test::Bencher;

fn bench_random_at_depth(n: usize, b: &mut Bencher) {
    //assert!(n > 0);
    let mut rng = thread_rng();
    Hashlife::with_new(|hl| {
        b.iter(|| hl.big_step(hl.random_block(&mut rng, n).unwrap_node()));
    });
}

#[bench]
fn bench_random_2_pow_3(b: &mut Bencher) {
    bench_random_at_depth(3, b);
}

#[bench]
fn bench_random_2_pow_5(b: &mut Bencher) {
    bench_random_at_depth(5, b);
}

#[bench]
fn bench_random_2_pow_7(b: &mut Bencher) {
    bench_random_at_depth(7, b);
}

#[ignore]
#[bench]
fn bench_random_2_pow_9(b: &mut Bencher) {
    bench_random_at_depth(9, b);
}
