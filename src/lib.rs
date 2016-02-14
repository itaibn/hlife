#[macro_use]
extern crate nom;

mod block;
mod cache;
pub mod evolve;
mod format;

pub use evolve::Hashlife;
