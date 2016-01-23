use std::collections::HashMap;
use std::mem;

mod cache;

pub struct CacheData (HashMap<u64, UnsafeBlock>);

enum GeneralBlock<Node> {
    Node(Node),
    Leaf(Leaf),
}

type UnsafeBlock = GeneralBlock<UnsafeNode>;

#[derive(Hash)]
struct UnsafeNode {
    subblocks: [[*const UnsafeBlock; 2]; 2],
    //evolve: 
}

pub type Block<'a> = GeneralBlock<Node<'a>>;

pub struct Node<'a> {
    subblocks: [[&'a Block<'a>; 2]; 2],
    //evolve:
}

type Leaf = u8;

impl CacheData {
    pub fn lookup<'a>(&'a self, hash: u64) -> Option<&'a Block<'a>> {
        unsafe {mem::transmute(self.0.get(&hash))}
    }
}
