#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![cfg_attr(feature="clippy_pedantic", warn(clippy_pedantic))]

// Clippy doesn't like this pattern, but I do. I may consider changing my mind
// on this in the future, just to make clippy happy.
#![cfg_attr(all(feature="clippy", not(feature="clippy_pedantic")),
    allow(needless_range_loop))]

extern crate fnv;
#[macro_use]
extern crate nom;
extern crate rand;

#[macro_use]
mod util;

pub mod evolve;
pub mod format;
pub mod global;

pub use evolve::Hashlife;

mod block;
mod cache;
