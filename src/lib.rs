#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![cfg_attr(feature="clippy_pedantic", warn(clippy_pedantic))]

// Clippy doesn't like this pattern, but I do. I may consider changing my mind
// on this in the future, just to make clippy happy.
#![cfg_attr(all(feature="clippy", not(feature="clippy_pedantic")),
    allow(needless_range_loop))]

#[macro_use]
extern crate nom;

mod block;
mod cache;
pub mod evolve;
pub mod format;

pub use evolve::Hashlife;
