//! Low-level code for the creation and handling of blocks

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

/// A hashtable with all the block nodes used for a Hashlife computation.
/// Lifetime parameter indicates the lifetime of all the blocks stored therein;
/// Note that all the blocks are owned are owned by `CABlockCache`; the nodes
/// themselves only contain references to one another (with lifetime 'a).
pub struct CABlockCache<'a> (HashMap<u64, Box<HeapNode<'a>>>);

impl<'a> CABlockCache<'a> {
    /// Create a new `CABlockCache` and pass it to `f`.
    /// This indirect initialization approach is necessary since the
    /// CABlockCache needs to outlive its own lifetime parameter, and a simple
    /// `new` method cannot specify this lifetime constraint. This API winds up
    /// very similar to using "generativity" -- indeed, a side effect of this
    /// implementation is that each node is uniquely associated at the
    /// type-level to the block cache that owns it, and it is impossible for a
    /// node owned by one cache to link to nodes owned by another cache.
    /// However, this is not a fundamental nor necessary feature of the design
    /// -- for instance, there is nothing unsafe about implementing a
    /// `with_two_new` method that generates two `CABlockCache`s with the same
    /// lifetime parameter, and with such a method it is possible (though
    /// probably a bad idea) to mix nodes owned by by block caches.
    pub fn with_new<F, T>(f: F) -> T
        where F: for<'b> FnOnce(CABlockCache<'b>) -> T {

        let ca_block_cache = CABlockCache(HashMap::new());
        f(ca_block_cache)
    }

    /// Return a reference to a node with `elems` as corners, creating this node
    /// if it did not already exist.
    pub fn new_block(&mut self, elems: [[Block<'a>; 2]; 2]) -> Node<'a> {
        let hash = hash(&elems);
        let blockref: &HeapNode<'a> = &**self.0.entry(hash).or_insert_with(||
            Box::new(HeapNode::from_elems_and_hash(elems, hash)));
        assert!(blockref.content == elems, "Hash collision");
        // The only unsafe line in this crate! Extend the lifetime of blockref
        // to 'a. This is why the hashmap needs to store all the nodes in boxes:
        // If it stored the nodes directly, the reference to them would be
        // invalidated whenever the hashtable is resized. When storing boxes,
        // the references to them live as long as the boxes themselves. Notice
        // that the interface of `CABlockCache` does not allow removing entries
        // from the underlying hashmap, this box will live as long as the
        // underlying hashmap, so extending the lifetime to 'a is safe.
        unsafe {&*(blockref as *const _)}
    }
}

// Just in case, clear CABlockCache before dropping it.
impl<'a> Drop for CABlockCache<'a> {
    fn drop(&mut self) {
        for (_, block) in &mut self.0 {
            block.content = [[Block::Leaf(0); 2]; 2];
        }
    }
}

// Note: uncertain if default implementation of Debug is right
#[derive(Debug)]
pub struct HeapNode<'a> {
    pub content: [[Block<'a>; 2]; 2],
    hash: u64,
    pub evolve: Cache<Block<'a>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Block<'a> {
    Node(Node<'a>),
    Leaf(Leaf),
}

pub type Node<'a> = &'a HeapNode<'a>;
pub type Leaf = u8;

pub const LEAF_SIZE: usize = 2;

impl<'a> HeapNode<'a> {
    fn from_elems_and_hash(elems: [[Block; 2]; 2], hash: u64) -> HeapNode {
        HeapNode {
            content: elems,
            hash: hash,
            evolve: Cache::new(),
        }
    }

    pub fn corners(&self) -> &[[Block<'a>; 2]; 2] {
        &self.content
    }
}

impl<'a> PartialEq for HeapNode<'a> {
    fn eq(&self, other: &HeapNode<'a>) -> bool {
        self.hash == other.hash
    }
}

impl<'a> Eq for HeapNode<'a> { }

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
        if let Block::Leaf(l) = *self {
            l
        } else {
            panic!("unwrap_leaf: Not a leaf");
        }
    }

    pub fn unwrap_node(&self) -> &Node<'a> {
        if let Block::Node(ref n) = *self {
            &n
        } else {
            panic!("unwrap_node: Not a node");
        }
    }

    // Will probably be moved
    pub fn depth(&self) -> usize {
        let mut count = 0;
        let mut block: &Block = &self;
        while let Block::Node(ref n) = *block {
            block = &n.content[0][0];
            count += 1;
        }
        count
    }
}
