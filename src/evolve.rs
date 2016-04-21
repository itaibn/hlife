use std::cell::{RefCell, RefMut};

pub use block::{Block, Node, Leaf, LEAF_SIZE};
use block::CABlockCache;

pub struct Hashlife<'a> {
    table: RefCell<CABlockCache<'a>>,
    small_evolve_cache: [u8; 1<<16],
    blank_cache: RefCell<Vec<Block<'a>>>,
    //placeholder_node: Node<'a>,
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
                blank_cache: RefCell::new(vec![Block::Leaf(0)]),
                //placeholder_node: placeholder_node,
            };
            f(hashlife)
        })
    }

    pub fn node(&self, elems: [[Block<'a>; 2]; 2]) -> Node<'a> {
        self.block_cache().node(elems)
    }

    pub fn block_cache(&self) -> RefMut<CABlockCache<'a>> {
        self.table.borrow_mut()
    }

    pub fn evolve(&self, node: Node<'a>) -> Block<'a> {
        let elem = node.corners();

        node.evolve_cache().eval(move ||
            match elem[0][0] {
                Block::Leaf(a00) => {
                    let a01 = elem[0][1].unwrap_leaf();
                    let a10 = elem[1][0].unwrap_leaf();
                    let a11 = elem[1][1].unwrap_leaf();
                    let res_leaf = self.evolve_leaf(
                        [[a00, a01], [a10, a11]]);
                    Block::Leaf(res_leaf)
                },
                Block::Node(_) => {
                    let mut intermediates = [[Block::Node(node); 3]; 3];
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

    fn evolve_finish(&self, parts: [[Block<'a>; 3]; 3]) -> Block<'a>
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
        Block::Node(self.node(res_components))
    }

    fn subblock(&self, node: Node<'a>, x: u8, y: u8) -> Block<'a>
    {
        let (x, y) = (x as usize, y as usize);

        if (x|y)&1 == 0 {
            node.corners()[x/2][y/2]
        } else {
            match node.corners()[0][0] {
                Block::Leaf(_) => self.subblock_leaf(node, x, y),
                Block::Node(_) => self.subblock_node(node, x, y),
            }
        }
    }

    fn subblock_node(&self, node: Node<'a>, x: usize, y: usize) -> Block<'a>
    {
        //let (x, y) = (x as usize, y as usize);
        let mut components = [[Block::Node(node); 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                let xx = i+x;
                let yy = j+y;
                components[i][j] = node.corners()[xx/2][yy/2]
                    .unwrap_node().corners()[xx&1][yy&1];
            }
        }
        Block::Node(self.node(components))
    }

    fn subblock_leaf(&self, node: Node<'a>, x: usize, y: usize) -> Block<'a>
    {
        let mut output_leaf = 0;
        for i in 0..2 {
            for j in 0..2 {
                let xx = i+x;
                let yy = j+y;
                let cell = 1 & (node.corners()[xx/2][yy/2].unwrap_leaf()
                    >> ((xx&2) + 4*(yy&2)));
                output_leaf |= cell << (i + 4*j);
            }
        }
        Block::Leaf(output_leaf)
    }

    #[inline]
    fn evolve_leaf(&self, leafs: [[Leaf; 2]; 2]) -> u8 {
        let entry = leafs[0][0] as usize
            + ((leafs[0][1] as usize) << 2)
            + ((leafs[1][0] as usize) << 8)
            + ((leafs[1][1] as usize) << 10);
        self.small_evolve_cache[entry]
    }

    pub fn blank(&self, depth: usize) -> Block<'a> {
        let mut blank_cache = self.blank_cache.borrow_mut();

        if depth < blank_cache.len() {
            blank_cache[depth]
        } else {
            let mut big_blank = *blank_cache.last().unwrap();
            let repeats = depth + 1 - blank_cache.len();
            for _ in 0..repeats {
                big_blank = Block::Node(self.node([[big_blank; 2]; 2]));
                blank_cache.push(big_blank);
            }
            big_blank
        }
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
}
