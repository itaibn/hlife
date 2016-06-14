use std::cell::{RefCell, RefMut};
use std::fmt;

pub use block::{Leaf, LEAF_SIZE};
use block::{Block as RawBlock, Node as RawNode, CABlockCache};

pub struct Hashlife<'a> {
    table: RefCell<CABlockCache<'a>>,
    small_evolve_cache: [u8; 1<<16],
    blank_cache: RefCell<Vec<RawBlock<'a>>>,
    //placeholder_node: Node<'a>,
}

struct Block<'a> {
    raw: RawBlock<'a>,
    hl: &'a Hashlife<'a>,
    depth: usize,
}

struct Node<'a> {
    raw: RawNode<'a>,
    hl: &'a Hashlife<'a>,
    depth: usize,
}

// TODO: Incorporate into rest of this code
pub fn make_2x2<A,F>(func: F) -> [[A; 2]; 2]
    where F : Fn(usize, usize) -> A {
    
    [[func(0, 0), func(0, 1)], [func(1, 0), func(1, 1)]]
}

pub fn make_3x3<A,F>(func: F) -> [[A; 3]; 3]
    where F : Fn(usize, usize) -> A {

    [[func(0, 0), func(0, 1), func(0, 2)],
     [func(1, 0), func(1, 1), func(1, 2)],
     [func(2, 0), func(2, 1), func(2, 2)]]
}

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

    pub fn node(&self, elems: [[RawBlock<'a>; 2]; 2]) -> RawNode<'a> {
        self.block_cache().node(elems)
    }

    pub fn node_block(&self, elems: [[RawBlock<'a>; 2]; 2]) -> RawBlock<'a> {
        RawBlock::Node(self.node(elems))
    }

    pub fn block_cache(&self) -> RefMut<CABlockCache<'a>> {
        self.table.borrow_mut()
    }

    pub fn evolve(&self, node: RawNode<'a>) -> RawBlock<'a> {
        let elem = node.corners();

        node.evolve_cache().eval(move ||
            match elem[0][0] {
                RawBlock::Leaf(a00) => {
                    let a01 = elem[0][1].unwrap_leaf();
                    let a10 = elem[1][0].unwrap_leaf();
                    let a11 = elem[1][1].unwrap_leaf();
                    let res_leaf = self.evolve_leaf(
                        [[a00, a01], [a10, a11]]);
                    RawBlock::Leaf(res_leaf)
                },
                RawBlock::Node(_) => {
                    let mut intermediates = [[RawBlock::Node(node); 3]; 3];
                    for i in 0..3 {
                        for j in 0..3 {
                            // I don't know we need two separate `let`
                            // statements, but the borrow checker complains if I
                            // combine them.
                            let subblock = self.subblock(node, i as u8,
                                j as u8);
                            let subnode = subblock.unwrap_node();
                            intermediates[i][j] = self.evolve(subnode);
                        }
                    }
                    self.evolve_finish(intermediates)
                }
            }
        )
    }

    fn evolve_finish(&self, parts: [[RawBlock<'a>; 3]; 3]) -> RawBlock<'a>
    {
        let mut res_components = [[parts[0][0]; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                let mut section = [[parts[0][0]; 2]; 2];
                for x in 0..2 {
                    for y in 0..2 {
                        section[x][y] = parts[i+x][j+y];
                    }
                }
                let subpart = self.node(section);
                res_components[i][j] = self.evolve(subpart);
            }
        }
        self.node_block(res_components)
    }

    /// Public for use in other modules in this crate; don't rely on it.
    pub fn subblock(&self, node: RawNode<'a>, x: u8, y: u8) -> RawBlock<'a>
    {
        debug_assert!(x < 3 && y < 3);
        let (x, y) = (x as usize, y as usize);

        if (x|y)&1 == 0 {
            node.corners()[y/2][x/2]
        } else {
            match node.corners()[0][0] {
                RawBlock::Leaf(_) => self.subblock_leaf(node, x, y),
                RawBlock::Node(_) => self.subblock_node(node, x, y),
            }
        }
    }

    fn subblock_node(&self, node: RawNode<'a>, x: usize, y: usize) ->
        RawBlock<'a> {

        //let (x, y) = (x as usize, y as usize);
        let mut components = [[RawBlock::Node(node); 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                let xx = i+x;
                let yy = j+y;
                components[i][j] = node.corners()[xx/2][yy/2]
                    .unwrap_node().corners()[xx&1][yy&1];
            }
        }
        self.node_block(components)
    }

    fn subblock_leaf(&self, node: RawNode<'a>, x: usize, y: usize) ->
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
                println!("i {} j {} output_leaf {:x}", i, j, output_leaf);
            }
        }
        let res = RawBlock::Leaf(output_leaf);
        println!("ol {:x}\n{:?}", output_leaf, res);
        res
    }

    #[inline]
    fn evolve_leaf(&self, leafs: [[Leaf; 2]; 2]) -> u8 {
        let entry = leafs[0][0] as usize
            + ((leafs[0][1] as usize) << 2)
            + ((leafs[1][0] as usize) << 8)
            + ((leafs[1][1] as usize) << 10);
        self.small_evolve_cache[entry]
    }

    pub fn blank(&self, depth: usize) -> RawBlock<'a> {
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
            hl: &self,
            depth: raw.depth(),
        }
    }

    fn node_from_raw(&'a self, raw: RawNode<'a>) -> Node<'a> {
        Node {
            raw: raw,
            hl: &self,
            depth: raw.depth(),
        }
    }

    pub fn step_pow2(&self, node: RawNode<'a>, lognsteps: usize) -> RawBlock<'a>
    {
        assert!(lognsteps < node.depth());

        if lognsteps == node.depth() - 1 {
            self.evolve(node)
        } else {
            let parts = make_3x3(|i, j| {
                self.subblock(self.subblock(node, i as u8, j as
                    u8).unwrap_node(), 1, 1)
            });

            self.node_block(make_2x2(|x, y| {
                let around = self.node(make_2x2(|i, j| parts[x+i][y+j]));
                self.step_pow2(around, lognsteps)
            }))
        }
    }
}

impl<'a> fmt::Debug for Hashlife<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Hashlife instance>")
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
        let input_rles: [&'static [u8]; 2] = [
            b"bbo$boo$bbo!",
            b"x = 8, y = 8, rule = B3/S23\n\
            b3ob2o$bo2bobo$2obobo$bobob2o$obobob2o$2bo2b2o$ob2ob2o$bo2b3o!"
        ];
        let output_rles: [&'static [u8]; 2] = [
            b"oo$oo!",
            //b"x = 4, y = 4, rule = B3/S23
            b"o$b2o$o$o!"
        ];

        Hashlife::with_new(|hl| {
            for (input_rle, output_rle) in input_rles.iter()
                                                     .zip(output_rles.iter()) {
                print!("Testing:\n{}\n->\n{}\n",
                    String::from_utf8_lossy(input_rle),
                    String::from_utf8_lossy(output_rle));
                let input = hl.block_from_bytes(*input_rle)
                              .expect(&format!("Error parsing {:?}",
                                    String::from_utf8_lossy(input_rle)));
                let output = hl.block_from_bytes(*output_rle)
                               .expect(&format!("Error parsing {:?}",
                                    String::from_utf8_lossy(input_rle)));

                assert_eq!(hl.evolve(input.unwrap_node()), output)
            }
        });
    }

    #[test]
    fn test_blank0() {
        Hashlife::with_new(|hl| {
            let blank2 = hl.blank(2);
            assert_eq!(blank2.depth(), 2);
            let blank0 = hl.blank(0);
            assert_eq!(blank0, Block::Leaf(0));
            let blank1 = hl.blank(1);
            assert_eq!(blank2.unwrap_node().corners(), &[[blank1; 2]; 2]);
            assert_eq!(blank1.unwrap_node().corners(), &[[blank0; 2]; 2]);
        });
    }

    #[test]
    fn test_blank1() {
        Hashlife::with_new(|hl| {
            assert_eq!(hl.blank(0), Block::Leaf(0));
            assert_eq!(hl.blank(1).depth(), 1);
            assert_eq!(hl.blank(2).depth(), 2);
        });
    }
 
    #[test]
    fn test_step_pow2() {
        Hashlife::with_new(|hl| {
            let b = hl.block_from_bytes(b"2$6o!").unwrap();
            let n = b.unwrap_node();
            assert_eq!(hl.step_pow2(n, 0), hl.block_from_bytes(b"3o$3o!")
                .unwrap());
            assert_eq!(hl.step_pow2(n, 1), hl.block_from_bytes(b"3bo$2bo$2o!")
                .unwrap());
        });
    }

    #[test]
    fn test_subblock() {
        Hashlife::with_new(|hl| {
            let b = hl.block_from_bytes(b"bo$bo$3o$o!").unwrap();
            let n = b.unwrap_node();

            assert_eq!(hl.subblock(n, 0, 0),
                hl.block_from_bytes(b"bo$bo!").unwrap());
            assert_eq!(hl.subblock(n, 0, 1),
                hl.block_from_bytes(b"bo$oo!").unwrap());
            assert_eq!(hl.subblock(n, 0, 2),
                hl.block_from_bytes(b"oo$o!").unwrap());
            assert_eq!(hl.subblock(n, 1, 0),
                hl.block_from_bytes(b"o$o!").unwrap());
            assert_eq!(hl.subblock(n, 1, 1),
                hl.block_from_bytes(b"o$oo!").unwrap());
            assert_eq!(hl.subblock(n, 1, 2),
                hl.block_from_bytes(b"2o!").unwrap());
            assert_eq!(hl.subblock(n, 2, 0),
                hl.block_from_bytes(b"!").unwrap());
            assert_eq!(hl.subblock(n, 2, 1),
                hl.block_from_bytes(b"$o!").unwrap());
            assert_eq!(hl.subblock(n, 2, 2),
                hl.block_from_bytes(b"o!").unwrap());
        });
    }
}
