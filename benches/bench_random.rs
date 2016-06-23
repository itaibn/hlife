#![feature(test)]

extern crate hlife;
extern crate rand;
extern crate test;

use hlife::Hashlife;
use rand::thread_rng;
use test::Bencher;

fn bench_random_at_depth(n: usize, b: &mut Bencher) {
    assert!(n > 0);
    let mut rng = thread_rng();
    Hashlife::with_new(|hl| {
        b.iter(|| hl.evolve(hl.random_block(&mut rng, n).unwrap_node()));
    });
}

#[bench]
fn bench_random_depth_2(b: &mut Bencher) {
    bench_random_at_depth(2, b);
}

#[bench]
fn bench_random_depth_4(b: &mut Bencher) {
    bench_random_at_depth(4, b);
}

#[bench]
fn bench_random_depth_6(b: &mut Bencher) {
    bench_random_at_depth(6, b);
}

#[ignore]
#[bench]
fn bench_random_depth_8(b: &mut Bencher) {
    bench_random_at_depth(8, b);
}
