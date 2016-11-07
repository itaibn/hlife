#![feature(test)]

extern crate hlife;
extern crate test;

use hlife::Hashlife;
use test::Bencher;

#[bench]
fn bench_startup(b: &mut Bencher) {
    b.iter(|| Hashlife::with_new(|_| {}));
}
