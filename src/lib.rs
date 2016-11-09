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

//pub use evolve::Hashlife;

mod block;
mod leaf;
mod cache;

use std::cell::{RefCell, RefMut};
use std::fmt;

//use rand;

pub use block::{Leaf, LG_LEAF_SIZE, LEAF_SIZE};
use block::{
    Block as RawBlock,
    Node as RawNode,
    CABlockCache,
};
use util::{make_2x2};

/// Global state for the Hashlife algorithm. For information on the lifetime
/// parameter see `block::CABlockHash`.
pub struct Hashlife<'a> {
    table: RefCell<CABlockCache<'a>>,
    small_evolve_cache: [u8; 1<<16],
    blank_cache: RefCell<Vec<RawBlock<'a>>>,
    //placeholder_node: Node<'a>,
}

#[derive(Clone, Copy, Debug)]
pub struct Block<'a> {
    raw: RawBlock<'a>,
    hl: &'a Hashlife<'a>,
    lg_size: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Node<'a> {
    raw: RawNode<'a>,
    hl: &'a Hashlife<'a>,
    lg_size: usize,
}

impl<'a> Hashlife<'a> {
    /// Create a new Hashlife and pass it to a function. For explanation on why
    /// this function calling convention is used see `CABlockCache::with_new`
    pub fn with_new<F,T>(f: F) -> T
        where F: for<'b> FnOnce(Hashlife<'b>) -> T {
        CABlockCache::with_new(|bcache| {
            //let placeholder_node = bcache.new_block([[Block::Leaf(0); 2]; 2]);
            let hashlife = Hashlife {
                table: RefCell::new(bcache),
                small_evolve_cache: evolve::mk_small_evolve_cache(),
                blank_cache: RefCell::new(vec![RawBlock::Leaf(0)]),
                //placeholder_node: placeholder_node,
            };
            f(hashlife)
        })
    }

    /// Create a new node with `elems` as corners
    #[cfg_attr(features = "inline", inline)]
    pub fn node(&self, elems: [[RawBlock<'a>; 2]; 2]) -> RawNode<'a> {
        self.block_cache().node(elems)
    }

    /// Create a new block with `elems` as corners
    #[cfg_attr(features = "inline", inline)]
    pub fn node_block(&self, elems: [[RawBlock<'a>; 2]; 2]) -> RawBlock<'a> {
        RawBlock::Node(self.node(elems))
    }

    /// Reference to underlying block cache (I don't remember why I made it
    /// public)
    #[cfg_attr(features = "inline", inline)]
    pub fn block_cache(&self) -> RefMut<CABlockCache<'a>> {
        self.table.borrow_mut()
    }

    /// Given 2^(n+1)x2^(n+1) node `node`, progress it 2^(n-1) generations and
    /// return 2^nx2^n block in the center. This is the main component of the
    /// Hashlife algorithm.
    pub fn evolve(&self, node: RawNode<'a>) -> RawBlock<'a> {
        evolve::evolve(self, node)
    }

    /// Given 2^(n+1)x2^(n+1) block, return 2^nx2^n subblock that's y*2^(n-1)
    /// south and x*2^(n-1) east of the north-west corner.
    ///
    /// Public for use in other modules in this crate; don't rely on it.
    #[cfg_attr(features = "inline", inline)]
    pub fn subblock(&self, node: RawNode<'a>, y: u8, x: u8) -> RawBlock<'a>
    {
       evolve::subblock(self, node, y, x)
    }
    
    /// Return blank block (all the cells are dead) with a given depth
    pub fn blank(&self, lg_size: usize) -> RawBlock<'a> {
        let depth = lg_size - LG_LEAF_SIZE;
        let mut blank_cache = self.blank_cache.borrow_mut();

        if depth < blank_cache.len() {
            blank_cache[depth]
        } else {
            let mut big_blank = *blank_cache.last().unwrap();
            let repeats = depth + 1 - blank_cache.len();
            for _ in 0..repeats {
                big_blank = self.node_block([[big_blank; 2]; 2]);
                blank_cache.push(big_blank);
            }
            big_blank
        }
    }

    fn block_from_raw(&'a self, raw: RawBlock<'a>) -> Block<'a> {
        Block {
            raw: raw,
            hl: self,
            lg_size: raw.lg_size(),
        }
    }

    fn node_from_raw(&'a self, raw: RawNode<'a>) -> Node<'a> {
        Node {
            raw: raw,
            hl: self,
            lg_size: raw.lg_size(),
        }
    }

    /// Return sidelength 2^(n-1) block at the center of node after it evolved
    /// for 2^lognsteps steps.
    pub fn step_pow2(&self, node: RawNode<'a>, lognsteps: usize) -> RawBlock<'a>
    {
        evolve::step_pow2(self, node, lognsteps)
    }

    // Temp interface
    /// Return a block with all cells set randomly of size 2^(depth+1)
    pub fn random_block<R:rand::Rng>(&self, rng: &mut R, depth: usize) ->
        RawBlock<'a> {
        
        let lg_size = depth + 1;

        if lg_size == LG_LEAF_SIZE {
            use block::LEAF_MASK;
            let leaf = rng.gen::<Leaf>() & LEAF_MASK;
            RawBlock::Leaf(leaf)
        } else {
            self.node_block(make_2x2(|_,_| self.random_block(rng, depth-1)))
        }
    }
}

impl<'a> fmt::Debug for Hashlife<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Hashlife instance>")
    }
}

impl<'a> Node<'a> {
    pub fn evolve(&self) -> Block<'a> {
        self.hl.block_from_raw(self.hl.evolve(self.raw))
    }
}

impl<'a> Block<'a> {
    pub fn unwrap_node(self) -> Node<'a> {
        self.hl.node_from_raw(self.raw.unwrap_node())
    }
}

#[cfg(test)]
mod test {
    use super::Hashlife;
    use block::Block;

    #[test]
    fn test_blank0() {
        Hashlife::with_new(|hl| {
            let blank3 = hl.blank(5);
            assert_eq!(blank3.lg_size(), 5);
            let blank1 = hl.blank(3);
            let blank2 = hl.blank(4);
            assert_eq!(blank3.unwrap_node().corners(), &[[blank2; 2]; 2]);
            assert_eq!(blank2.unwrap_node().corners(), &[[blank1; 2]; 2]);


        });
    }

    #[test]
    fn test_blank1() {
        use block::LG_LEAF_SIZE;

        Hashlife::with_new(|hl| {
            assert_eq!(hl.blank(LG_LEAF_SIZE), Block::Leaf(0));
            assert_eq!(hl.blank(4).lg_size(), 4);
            assert_eq!(hl.blank(5).lg_size(), 5);
        });
    }
}
