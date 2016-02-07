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
pub struct CABlockCache<'a> (HashMap<u64, Box<Block<'a>>>);

impl<'a> CABlockCache<'a> {
    pub fn with_block_cache<F, T>(f: F) -> T
        where F: for<'b> FnOnce(CABlockCache<'b>) -> T {

        let ca_block_cache = CABlockCache(HashMap::new());
        f(ca_block_cache)
    }

    pub fn new_block(&mut self, desc: BlockDesc<'a>) -> BlockLink<'a> {
        let hash = hash(&desc);
        let blockref: &Block<'a> = &**self.0.entry(hash).or_insert_with(||
            Box::new(Block::from_desc_and_hash(desc, hash)));
        //unsafe {mem::transmute::<&Block<'a>, &'a Block<'a>>(blockref)}
        unsafe {&*(blockref as *const _)}
    }
}

pub struct Block<'a> {
    pub content: BlockDesc<'a>,
    hash: u64,
    pub evolve: Cache<Option<BlockLink<'a>>>,
}

#[derive(Hash)]
pub enum BlockDesc<'a> {
    Node(Node<'a>),
    Leaf(Leaf),
}

pub type BlockLink<'a> = &'a Block<'a>;
pub type Leaf = u8;
pub type Node<'a> = [[BlockLink<'a>; 2]; 2];


impl<'a> Block<'a> {
    fn from_desc_and_hash(desc: BlockDesc, hash: u64) -> Block {
        Block {
            content: desc,
            hash: hash,
            evolve: Cache::new(),
        }
    }    
}

impl<'a> Hash for Block<'a> {
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

impl<'a> BlockDesc<'a> {
    pub fn unwrap_leaf(&self) -> Leaf {
        match *self {
            BlockDesc::Leaf(l) => l,
            _ => panic!("unwrap_leaf: Not a leaf"),
        }
    }

    pub fn unwrap_node(&self) -> &Node<'a> {
        match *self {
            BlockDesc::Node(ref n) => &n,
            _ => panic!("unwrap_node: Not a node"),
        }
    }
}
