use std::cell::{RefCell, RefMut};
use std::fmt;

use rand;

pub use block::{Leaf, LG_LEAF_SIZE, LEAF_SIZE};
use block::{Block as RawBlock, Node as RawNode, CABlockCache};
use util::{make_2x2, make_3x3};

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

/// A table containing the 2x2 center block after one generation for all
/// possible 4x4 blocks.
fn mk_small_evolve_cache() -> [u8; 1<<16] {
    let mut res = [0; 1<<16];
    let bitcount = [0, 1, 1, 2, 1, 2, 2, 3];
    for a in 0..8 {
        for b in 0..8 {
            for c in 0..8 {
                let entry = a | b << 4 | c << 8;
                let count = bitcount[a] + bitcount[b] + bitcount[c];
                let living =
                    count == 3 || (count == 4 && (b & 2 != 0));
                res[entry] = if living {1} else {0};
            }
        }
    }
    for x in 0..1<<16 {
        let mut evolve = 0;
        for i in 0..2 {
            for j in 0..2 {
                let subblock = (x >> (i+4*j)) & 0x777;
                evolve |= (res[subblock] & 1) << (i + 4*j);
            }
        }
        res[x] = evolve;
    }
    res
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
                small_evolve_cache: mk_small_evolve_cache(),
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
        let elem = node.corners();

        node.evolve_cache().eval(move ||
            if node.node_of_leafs() {
                let elem_leafs = make_2x2(|i, j| elem[i][j].unwrap_leaf());
                RawBlock::Leaf(self.evolve_leaf(elem_leafs))
            } else {
                let intermediates = make_3x3(|i, j| {
                    // I don't know we need two separate `let`
                    // statements, but the borrow checker complains if I
                    // combine them.
                    let subblock = self.subblock(node, i as u8,
                        j as u8);
                    let subnode = subblock.unwrap_node();
                    self.evolve(subnode)
                });
                self.evolve_finish(intermediates)
            }
        )
    }

    /// Evolve (3*2^n)x(3*2^n) block (encoded as a 3x3 array of 2^nx2^n blocks)
    /// 2^(n-1) steps and return the 2^nx2^n block in the middle
    #[cfg_attr(features = "inline", inline)]
    fn evolve_finish(&self, parts: [[RawBlock<'a>; 3]; 3]) -> RawBlock<'a>
    {
        let res_components = make_2x2(|i, j| {
            self.evolve(self.node(make_2x2(|y, x| parts[i+y][j+x])))
        });
        self.node_block(res_components)
    }

    /// Given 2^(n+1)x2^(n+1) block, return 2^nx2^n subblock that's y*2^(n-1)
    /// south and x*2^(n-1) east of the north-west corner.
    ///
    /// Public for use in other modules in this crate; don't rely on it.
    #[cfg_attr(features = "inline", inline)]
    pub fn subblock(&self, node: RawNode<'a>, y: u8, x: u8) -> RawBlock<'a>
    {
        debug_assert!(x < 3 && y < 3);
        let (x, y) = (x as usize, y as usize);

        if (x|y)&1 == 0 {
            node.corners()[y/2][x/2]
        } else if node.node_of_leafs() {
            self.subblock_leaf(node, y, x)
        } else {
            self.subblock_node(node, y, x)
        }
    }

    #[cfg_attr(features = "inline", inline)]
    fn subblock_node(&self, node: RawNode<'a>, y: usize, x: usize) ->
        RawBlock<'a> {

        //let (x, y) = (x as usize, y as usize);
        let components = make_2x2(|j, i| {
            let xx = i+x;
            let yy = j+y;
            node.corners()[yy/2][xx/2].unwrap_node().corners()[yy&1][xx&1]
        });
        self.node_block(components)
    }

    #[cfg_attr(features = "inline", inline)]
    fn subblock_leaf(&self, node: RawNode<'a>, y: usize, x: usize) ->
        RawBlock<'a> {

        let mut output_leaf = 0;
        for i in 0..2 {
            for j in 0..2 {
                let xx = i+x;
                let yy = j+y;
                debug_assert!(xx < 4 && yy < 4);
                let cell = 1 & (node.corners()[yy/2][xx/2].unwrap_leaf()
                    >> ((xx&1) + 4*(yy&1)));
                output_leaf |= cell << (i + 4*j);
                //println!("i {} j {} output_leaf {:x}", i, j, output_leaf);
            }
        }
        RawBlock::Leaf(output_leaf)
    }

    /// `evolve` specialized to when the corners are all leafs.
    #[inline]
    fn evolve_leaf(&self, leafs: [[Leaf; 2]; 2]) -> Leaf {
        debug_assert!(LEAF_SIZE == 2);
        let entry = leafs[0][0] as usize
            + ((leafs[0][1] as usize) << 2)
            + ((leafs[1][0] as usize) << 8)
            + ((leafs[1][1] as usize) << 10);
        self.small_evolve_cache[entry]
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

    pub fn step_pow2(&self, node: RawNode<'a>, lognsteps: usize) -> RawBlock<'a>
    {
        assert!(lognsteps <= node.lg_size() - 2);

        if lognsteps == node.lg_size() - 2 {
            self.evolve(node)
        } else {
            let parts = make_3x3(|i, j| {
                self.subblock(self.subblock(node, i as u8, j as
                    u8).unwrap_node(), 1, 1)
            });

            self.node_block(make_2x2(|y, x| {
                let around = self.node(make_2x2(|i, j| parts[y+i][x+j]));
                self.step_pow2(around, lognsteps)
            }))
        }
    }

    /// Return a block with all cells set randomly of a given depth.
    pub fn random_block<R:rand::Rng>(&self, rng: &mut R, depth: usize) ->
        RawBlock<'a> {

        if depth == 0 {
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
    use super::{mk_small_evolve_cache, Hashlife};
    use block::Block;

    #[test]
    fn test_small_evolve_cache() {
        let cache = mk_small_evolve_cache();
        macro_rules! test_cases {
            ( $($test:expr => $result:expr),* ) =>
                {{$(assert_eq!(cache[$test], $result);)*}};
        }
        test_cases! (
            0x0070 => 0x11,
            0x0e00 => 0x22,
            0x1630 => 0x23,
            0x0660 => 0x33,
            0xffff => 0x00
        )
    }

    #[test]
    fn test_evolve() {
        let input_rles: [&'static str; 2] = [
            "bbo$boo$bbo!",
            "x = 8, y = 8, rule = B3/S23\n\
            3ob2o$bo2bobo$2obobo$bobob2o$obobob2o$2bo2b2o$ob2ob2o$bo2b3o!"
        ];
        let output_rles: [&'static str; 2] = [
            "oo$oo!",
            //b"x = 4, y = 4, rule = B3/S23
            "o$b2o$o$o!"
        ];

        Hashlife::with_new(|hl| {
            for (input_rle, output_rle) in input_rles.iter()
                                                     .zip(output_rles.iter()) {
                print!("Testing:\n{}\n->\n{}\n", input_rle, output_rle);
                let input = hl.rle(input_rle);
                let output = hl.rle(output_rle);

                assert_eq!(hl.evolve(input.unwrap_node()), output)
            }
        });
    }

    #[test]
    fn test_blank0() {
        Hashlife::with_new(|hl| {
            let blank3 = hl.blank(3);
            assert_eq!(blank3.lg_size(), 3);
            let blank1 = hl.blank(1);
            assert_eq!(blank1, Block::Leaf(0));
            let blank2 = hl.blank(2);
            assert_eq!(blank3.unwrap_node().corners(), &[[blank2; 2]; 2]);
            assert_eq!(blank2.unwrap_node().corners(), &[[blank1; 2]; 2]);
        });
    }

    #[test]
    fn test_blank1() {
        Hashlife::with_new(|hl| {
            assert_eq!(hl.blank(1), Block::Leaf(0));
            assert_eq!(hl.blank(2).lg_size(), 2);
            assert_eq!(hl.blank(3).lg_size(), 3);
        });
    }
 
    #[test]
    fn test_step_pow2() {
        Hashlife::with_new(|hl| {
            let b = hl.rle("2$6o!");
            let n = b.unwrap_node();
            assert_eq!(hl.step_pow2(n, 1), hl.evolve(n));
            assert_eq!(hl.step_pow2(n, 0), hl.rle("3o$3o!"));
            assert_eq!(hl.step_pow2(n, 1), hl.rle("3bo$2bo$2o!"));
        });
    }

    #[test]
    fn test_subblock_0() {
        Hashlife::with_new(|hl| {
            let b = hl.rle("bo$bo$3o$o!");
            let n = b.unwrap_node();

            assert_eq!(hl.subblock(n, 0, 0), hl.rle("bo$bo!"));
            assert_eq!(hl.subblock(n, 1, 0), hl.rle("bo$oo!"));
            assert_eq!(hl.subblock(n, 2, 0), hl.rle("oo$o!"));
            assert_eq!(hl.subblock(n, 0, 1), hl.rle("o$o!"));
            assert_eq!(hl.subblock(n, 1, 1), hl.rle("o$oo!"));
            assert_eq!(hl.subblock(n, 2, 1), hl.rle("2o!"));
            assert_eq!(hl.subblock(n, 0, 2), hl.rle("!"));
            assert_eq!(hl.subblock(n, 1, 2), hl.rle("$o!"));
            assert_eq!(hl.subblock(n, 2, 2), hl.rle("o!"));
        });
    }

    #[test]
    fn test_subblock_1() {
        Hashlife::with_new(|hl| {
            let b = hl.rle("2$7o!");
            let n = b.unwrap_node();
            assert_eq!(hl.subblock(n, 0, 1), hl.rle("2$4o!"));
            assert_eq!(hl.subblock(n, 1, 0), hl.rle("4o!"));
            assert_eq!(hl.subblock(n, 0, 2), hl.rle("2$3o!"));
        });
    }
}
