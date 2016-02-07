use cache::Cache;

use std::collections::HashMap;
use std::hash::{Hash, Hasher, SipHasher};

// [Currently these notes are out of date.]
// NOTE ON OWNERSHIP AND SAFETY:
//
// The data in a Hashlife computation consists of a collection of blocks which
// are all listed in a hash table. In this implementation, the hash table,
// CABlockCache, owns all the blocks that it references. However, these blocks
// also reference one another. Ideally, these references would be immutable
// references with the same lifetime as the CABlockCache that owns those blocks.
// However, Rust has no way of specifying such a lifetime, so we need to use
// unsafe Rust to simulate such a feature, and I'm not convinced my
// implementation is safe.
//
// The type Block<'a> corresponds to a block with all references having
// lifetime 'a (including all references in blocks that it references,
// recursively). Since Block<'a> includes cache data with references to
// other blocks and interior mutability, it is invariant in 'a.

// A hashtable with all the blocks used for a Hashlife computation.
pub struct CABlockCache<'a> (HashMap<u64, Box<HeapNode<'a>>>);

impl<'a> CABlockCache<'a> {
    pub fn with_block_cache<F, T>(f: F) -> T
        where F: for<'b> FnOnce(CABlockCache<'b>) -> T {

        let ca_block_cache = CABlockCache(HashMap::new());
        f(ca_block_cache)
    }

    pub fn new_block(&mut self, elems: [[Block<'a>; 2]; 2]) -> Node<'a> {
        let hash = hash(&elems);
        let blockref: &HeapNode<'a> = &**self.0.entry(hash).or_insert_with(||
            Box::new(HeapNode::from_elems_and_hash(elems, hash)));
        unsafe {&*(blockref as *const _)}
    }
}

pub struct HeapNode<'a> {
    pub content: [[Block<'a>; 2]; 2],
    hash: u64,
    pub evolve: Cache<Block<'a>>,
}

#[derive(Clone, Copy, Hash)]
pub enum Block<'a> {
    Node(Node<'a>),
    Leaf(Leaf),
}

pub type Node<'a> = &'a HeapNode<'a>;
pub type Leaf = u8;

impl<'a> HeapNode<'a> {
    fn from_elems_and_hash(elems: [[Block; 2]; 2], hash: u64) -> HeapNode {
        HeapNode {
            content: elems,
            hash: hash,
            evolve: Cache::new(),
        }
    }
}

impl<'a> Hash for HeapNode<'a> {
    fn hash<H:Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

// Copied from std::hash documentation.
// TODO: Find something quicker then SipHash
fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = SipHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl<'a> Block<'a> {
    pub fn unwrap_leaf(&self) -> Leaf {
        match *self {
            Block::Leaf(l) => l,
            _ => panic!("unwrap_leaf: Not a leaf"),
        }
    }

    pub fn unwrap_node(&self) -> &Node<'a> {
        match *self {
            Block::Node(ref n) => &n,
            _ => panic!("unwrap_node: Not a node"),
        }
    }
}
