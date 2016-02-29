#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

// Clippy doesn't like this pattern, but I do. I may consider changing my mind
// on this in the future, just to make clippy happy.
#![cfg_attr(feature="clippy", allow(needless_range_loop))]

#[macro_use]
extern crate nom;

mod block;
mod cache;
pub mod evolve;
pub mod format;

pub use evolve::Hashlife;
