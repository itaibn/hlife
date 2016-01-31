use std::collections::HashMap;
use std::hash::{Hash, Hasher, SipHasher};
use std::mem;

mod cache;

pub struct CacheData (HashMap<u64, UnsafeBlock>);

type UnsafeBlock = HeapBlock<'static>;

struct HeapBlock<'a> {
    content: BlockDesc<'a>,
    hash: u64,
}

#[derive(Hash)]
enum BlockDesc<'a> {
    Node([[Block<'a>; 2]; 2]),
    Leaf(Leaf),
}

pub type Block<'a> = &'a HeapBlock<'a>;
type Leaf = u8;

impl CacheData {
    pub fn lookup<'a>(&'a self, hash: u64) -> Option<Block<'a>> {
        //unsafe {mem::transmute(self.0.get(&hash))}
        self.0.get(&hash)
    }

    unsafe fn add_block<'a>(&'a mut self, hash: u64, block: HeapBlock<'a>) {
        let unsafe_block = mem::transmute(block);
        self.0.insert(hash, unsafe_block);
    }

    unsafe fn add_block_from_desc<'a>(&'a mut self, desc: BlockDesc<'a>) {
        let block = HeapBlock::from_desc(desc);
        self.add_block(block.hash, block);
    }

    unsafe fn get_block<'a>(&'a mut self, desc: BlockDesc<'a>) -> Block<'a> {
        let hash = hash(&desc);
        self.lookup(hash).map(|block| {return block;});
        // Note: Code branch calculates hash twice.
        self.add_block_from_desc(desc);
        self.lookup(hash).unwrap()
    }
}

impl<'a> HeapBlock<'a> {
    fn from_desc(desc: BlockDesc) -> HeapBlock {
        let hash = hash(&desc);
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
