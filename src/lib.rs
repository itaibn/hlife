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

pub use leaf::{Leaf, LG_LEAF_SIZE, LEAF_SIZE, LEAF_MASK};
use block::{
    Block as RawBlock,
    Node as RawNode,
    CABlockCache,
};
use util::make_2x2;

/// Global state for the Hashlife algorithm. For information on the lifetime
/// parameter see `block::CABlockHash`.
struct HashlifeCache<'a> {
    table: RefCell<CABlockCache<'a>>,
    small_evolve_cache: [u8; 1<<16],
    blank_cache: RefCell<Vec<RawBlock<'a>>>,
    //placeholder_node: Node<'a>,
}

#[derive(Clone, Copy, Debug)]
pub struct Hashlife<'a>(&'a HashlifeCache<'a>);

#[derive(Clone, Copy, Debug)]
pub struct Block<'a> {
    raw: RawBlock<'a>,
    hl: Hashlife<'a>,
    lg_size: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Node<'a> {
    raw: RawNode<'a>,
    hl: Hashlife<'a>,
    lg_size: usize,
}

impl<'a> Drop for HashlifeCache<'a> {
    fn drop(&mut self) {
        self.blank_cache.get_mut().clear();
    }
}

impl<'a> Hashlife<'a> {
    /// Create a new Hashlife and pass it to a function. For explanation on why
    /// this function calling convention is used see `CABlockCache::with_new`
    pub fn with_new<F,T>(f: F) -> T
        where F: for<'b> FnOnce(Hashlife<'b>) -> T {
        CABlockCache::with_new(|bcache| {
            //let placeholder_node = bcache.new_block([[Block::Leaf(0); 2]; 2]);
            let hashlife_cache = HashlifeCache {
                table: RefCell::new(bcache),
                small_evolve_cache: evolve::mk_small_evolve_cache(),
                blank_cache: RefCell::new(vec![RawBlock::Leaf(0)]),
                //placeholder_node: placeholder_node,
            };
            let hashlife = unsafe {&*(&hashlife_cache as *const _)};
            f(Hashlife(hashlife))
        })
    }

    /// Create a new raw node with `elems` as corners
    pub fn raw_node(&self, elems: [[RawBlock<'a>; 2]; 2]) -> RawNode<'a> {
        self.block_cache().node(elems)
    }

    /// Creates a node `elems` as corners. Panics with sizes don't match.
    pub fn node(&self, elems: [[Block<'a>; 2]; 2]) -> Node<'a> {
        let elem_lg_size = elems[0][0].lg_size();
        make_2x2(|i, j| assert!(elems[i][j].lg_size() == elem_lg_size,
            "Sizes don't match in new node"));
        let raw_elems = make_2x2(|i, j| elems[i][j].to_raw());

        Node {
            raw: self.raw_node(raw_elems),
            hl: *self,
            lg_size: elem_lg_size + 1,
        }
    }

    /// Create a new block with `elems` as corners
    pub fn raw_node_block(&self, elems: [[RawBlock<'a>; 2]; 2]) -> RawBlock<'a>
    {
        RawBlock::Node(self.raw_node(elems))
    }

    /// Creates a new block with `elems` as corners. Panics if sizes don't
    /// match.
    pub fn node_block(&self, elems: [[Block<'a>; 2]; 2]) -> Block<'a> {
        Block::from_node(self.node(elems))
    }

    /// Creates leaf block
    pub fn leaf(&self, leaf: Leaf) -> Block<'a> {
        Block {
            raw: RawBlock::Leaf(leaf),
            hl: *self,
            lg_size: LG_LEAF_SIZE,
        }
    }

    /// Reference to underlying block cache (I don't remember why I made it
    /// public)
    pub fn block_cache(&self) -> RefMut<CABlockCache<'a>> {
        self.0.table.borrow_mut()
    }

    /// Small block cache for `evolve`
    pub fn small_evolve_cache(&self) -> &[u8; 1<<16] {
        &self.0.small_evolve_cache
    }

    /// Given 2^(n+1)x2^(n+1) node `node`, progress it 2^(n-1) generations and
    /// return 2^nx2^n block in the center. This is the main component of the
    /// Hashlife algorithm.
    ///
    /// This is the raw version of big stepping.
    pub fn raw_evolve(&self, node: RawNode<'a>) -> RawBlock<'a> {
        evolve::evolve(self, node)
    }

    /// Given 2^(n+1)x2^(n+1) node `node`, progress it 2^(n-1) generations and
    /// return 2^nx2^n block in the center. This is the main component of the
    /// Hashlife algorithm.
    ///
    /// This is the normal version of big stepping.
    pub fn big_step(&self, node: Node<'a>) -> Block<'a> {
        Block {
            raw: evolve::evolve(self, node.raw),
            hl: *self,
            lg_size: node.lg_size - 1, 
        }
    }

    /// Given 2^(n+1)x2^(n+1) block, return 2^nx2^n subblock that's y*2^(n-1)
    /// south and x*2^(n-1) east of the north-west corner.
    ///
    /// Public for use in other modules in this crate; don't rely on it.
    pub fn raw_subblock(&self, node: RawNode<'a>, y: u8, x: u8) -> RawBlock<'a>
    {
       evolve::subblock(self, node, y, x)
    }
    
    /// Returns a raw blank block (all the cells are dead) with a given depth
    pub fn raw_blank(&self, lg_size: usize) -> RawBlock<'a> {
        let depth = lg_size - LG_LEAF_SIZE;
        let mut blank_cache = self.0.blank_cache.borrow_mut();

        if depth < blank_cache.len() {
            blank_cache[depth]
        } else {
            let mut big_blank = *blank_cache.last().unwrap();
            let repeats = depth + 1 - blank_cache.len();
            for _ in 0..repeats {
                big_blank = self.raw_node_block([[big_blank; 2]; 2]);
                blank_cache.push(big_blank);
            }
            big_blank
        }
    }

    /// Returns a blank block (all the cells are dead) with a given depth
    pub fn blank(&self, lg_size: usize) -> Block<'a> {
        Block {
            raw: self.raw_blank(lg_size),
            hl: *self,
            lg_size: lg_size,
        }
    }

    fn block_from_raw(&self, raw: RawBlock<'a>) -> Block<'a> {
        Block {
            raw: raw,
            hl: *self,
            lg_size: raw.lg_size_verified().unwrap(),
        }
    }

    fn node_from_raw(&self, raw: RawNode<'a>) -> Node<'a> {
        Node {
            raw: raw,
            hl: *self,
            lg_size: RawBlock::Node(raw).lg_size_verified().unwrap(),
        }
    }

    /// Return sidelength 2^(n-1) block at the center of node after it evolved
    /// for 2^lognsteps steps.
    pub fn raw_step_pow2(&self, node: RawNode<'a>, lognsteps: usize) ->
        RawBlock<'a> {

        evolve::step_pow2(self, node, lognsteps)
    }

    /// Return sidelength 2^(n-1) block at the center of node after it evolved
    /// for 2^lognsteps steps.
    pub fn step_pow2(&self, node: Node<'a>, lognsteps: usize) -> Block<'a> { 
        let raw_node = self.raw_step_pow2(node.to_raw(), lognsteps);
        Block {
            raw: raw_node,
            hl: *self,
            lg_size: node.lg_size() - 1
        }
    }

    // Temp interface
    /// Return a block with all cells set randomly of size 2^(depth+1)
    pub fn raw_random_block<R:rand::Rng>(&self, rng: &mut R, depth: usize) ->
        RawBlock<'a> {
        
        let lg_size = depth + 1;

        if lg_size == LG_LEAF_SIZE {
            let leaf = rng.gen::<Leaf>() & LEAF_MASK;
            RawBlock::Leaf(leaf)
        } else {
            self.raw_node_block(make_2x2(|_,_| self.raw_random_block(rng,
                depth-1)))
        }
    }
}

impl<'a> fmt::Debug for HashlifeCache<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Hashlife instance>")
    }
}

impl<'a> Node<'a> {
    pub fn to_raw(&self) -> RawNode<'a> {
        self.raw
    }

    pub fn evolve(&self) -> Block<'a> {
        self.hl.block_from_raw(self.hl.raw_evolve(self.raw))
    }

    pub fn corners(&self) -> [[Block<'a>; 2]; 2] {
        make_2x2(|i, j| self.hl.block_from_raw(self.raw.corners()[i][j]))
    }

    pub fn lg_size(&self) -> usize {
        self.lg_size
    }

    pub fn node_of_leafs(&self) -> bool {
        self.lg_size == 1
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<'a> Eq for Node<'a> {}

impl<'a> Block<'a> {
    pub fn to_raw(&self) -> RawBlock<'a> {
        self.raw
    }


    pub fn from_node(node: Node<'a>) -> Self {
        Block {
            raw: RawBlock::Node(node.raw),
            hl: node.hl,
            lg_size: node.lg_size,
        }
    }
    pub fn destruct(self) -> Result<Node<'a>, Leaf> {
        match self.raw {
            RawBlock::Node(n) => Ok(self.hl.node_from_raw(n)),
            RawBlock::Leaf(l) => Err(l),
        }
    }

    pub fn unwrap_leaf(self) -> Leaf {
        self.destruct().unwrap_err()
    }

    pub fn unwrap_node(self) -> Node<'a> {
        self.destruct().unwrap()
    }

    pub fn lg_size(&self) -> usize {
        self.lg_size
    }

    pub fn lg_size_verified(&self) -> Result<usize, ()> {
        Ok(self.lg_size())
    }

    pub fn is_blank(&self) -> bool {
        self.raw.is_blank()
    }
}

impl<'a> PartialEq for Block<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<'a> Eq for Block<'a> {}

#[cfg(test)]
mod test {
    use super::Hashlife;
    use leaf::LG_LEAF_SIZE;
    use block::Block;

    #[test]
    fn test_blank0() {
        Hashlife::with_new(|hl| {
            let blank3 = hl.raw_blank(5);
            assert_eq!(blank3.lg_size(), 5);
            let blank1 = hl.raw_blank(3);
            let blank2 = hl.raw_blank(4);
            assert_eq!(blank3.unwrap_node().corners(), &[[blank2; 2]; 2]);
            assert_eq!(blank2.unwrap_node().corners(), &[[blank1; 2]; 2]);


        });
    }

    #[test]
    fn test_blank1() {
        Hashlife::with_new(|hl| {
            assert_eq!(hl.raw_blank(LG_LEAF_SIZE), Block::Leaf(0));
            assert_eq!(hl.raw_blank(4).lg_size(), 4);
            assert_eq!(hl.raw_blank(5).lg_size(), 5);
        });
    }
}
