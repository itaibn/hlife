use std::collections::HashMap;
use std::hash::{Hash, Hasher, SipHasher};
use std::mem;

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
// The type HeapBlock<'a> corresponds to a block with all references having
// lifetime 'a (including all references in blocks that it references,
// recursively). Since HeapBlock<'a> includes cache data with references to
// other blocks and interior mutability, it is invariant in 'a [currently this
// is not implemented so it is covariant in 'a].

// A hashtable with all the blocks used for a Hashlife computation.
pub struct CABlockCache (HashMap<u64, UnsafeBlock>);

struct UnsafeBlock(HeapBlock<'static>);

impl CABlockCache {
    pub fn get_block<'a>(&'a mut self, desc: BlockDesc<'a>) -> Block<'a> {
        let hash = hash(&desc);
        let unsafe_block = self.0.entry(hash).or_insert_with(||
            UnsafeBlock::from_heap_block(
                HeapBlock::from_desc_and_hash(desc, hash)
            )
        );
        unsafe {&unsafe_block.to_heap_block()}
    }
}

impl UnsafeBlock {
    fn from_heap_block(heap_block: HeapBlock) -> UnsafeBlock {
        unsafe {UnsafeBlock(mem::transmute(heap_block))}
    }

    unsafe fn to_heap_block<'a>(&self) -> &HeapBlock<'a> {
        // This will fail when HeapBlock<'a> becomes invariant
        &self.0
    }
}

pub struct HeapBlock<'a> {
    content: BlockDesc<'a>,
    hash: u64,
}

#[derive(Hash)]
pub enum BlockDesc<'a> {
    Node([[Block<'a>; 2]; 2]),
    Leaf(Leaf),
}

pub type Block<'a> = &'a HeapBlock<'a>;
pub type Leaf = u8;


impl<'a> HeapBlock<'a> {
    fn from_desc_and_hash(desc: BlockDesc, hash: u64) -> HeapBlock {
        HeapBlock {
            content: desc,
            hash: hash,
        }
    }    
}

impl<'a> Hash for HeapBlock<'a> {
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
