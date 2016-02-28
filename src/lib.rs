#[macro_use]
extern crate nom;

mod block;
mod cache;
pub mod evolve;
pub mod format;

pub use evolve::Hashlife;
