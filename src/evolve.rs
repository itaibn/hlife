use block::*;

struct Hashlife<'a> {
    table: CABlockCache<'a>,
    small_evolve_cache: [u8; 1<<16],
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
    pub fn with_hashlife<F,T>(f: F) -> T
        where F: for<'b> FnOnce(Hashlife<'b>) -> T {
        CABlockCache::with_block_cache(|bcache| {
            let hashlife = Hashlife {
                table: bcache,
                small_evolve_cache: mk_small_evolve_cache(),
            };
            f(hashlife)
        })
    }

    pub fn evolve(&mut self, block: BlockLink<'a>) -> Option<BlockLink<'a>> {
        use block::BlockDesc::*;

        block.evolve.eval(move ||
            match block.content {
                Leaf(_) => None,
                Node(ref x) => {
                    match x[0][0].content {
                        Leaf(a00) => {
                            let a01 = x[0][1].content.unwrap_leaf();
                            let a10 = x[1][0].content.unwrap_leaf();
                            let a11 = x[1][1].content.unwrap_leaf();
                            let res_leaf = self.evolve_leaf(
                                [[a00, a01], [a10, a11]]);
                            Some(self.table.new_block(Leaf(res_leaf)))
                        },
                        // Make sure there's a branch which calls evolve()
                        // recursively, to check if all the lifetime hackery is
                        // correct
                        Node(_) => {
                            let mut intermediates = [[block; 3]; 3];
                            for i in 0..3 {
                                for j in 0..3 {
                                    let subblock = self.subblock(block, i as u8,
                                        j as u8);
                                    intermediates[i][j] = self.evolve(subblock)
                                        .unwrap();
                                }
                            }
                            Some(self.evolve_finish(intermediates))
                        }
                    }
                }
            }
        )
    }

    fn evolve_finish(&mut self, parts: [[BlockLink<'a>; 3]; 3]) -> BlockLink<'a>
    {
        use block::BlockDesc::*;

        let mut res_components = [[parts[0][0]; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                let mut section = [[parts[0][0]; 2]; 2];
                for x in 0..2 {
                    for y in 0..2 {
                        section[x][y] = parts[i+x][j+y];
                    }
                }
                let subpart = self.table.new_block(Node(section));
                res_components[i][j] = self.evolve(subpart).unwrap();
            }
        }
        self.table.new_block(Node(res_components))
    }

    #[inline]
    fn subblock(&mut self, block: BlockLink<'a>, x: u8, y: u8) -> BlockLink<'a>
    {
        let (x, y) = (x as usize, y as usize);

        let node = block.content.unwrap_node();
        if (x|y)&1 == 0 {
            node[x/2][y/2]
        } else {
            let mut components = [[block; 2]; 2];
            for i in 0..2 {
                for j in 0..2 {
                    let xx = i+x;
                    let yy = j+y;
                    components[i][j] = node[xx/2][yy/2]
                        .content.unwrap_node()[xx&1][yy&1];
                }
            }
            self.table.new_block(BlockDesc::Node(components))
        }
    }

    #[inline]
    fn evolve_leaf(&self, leafs: [[u8; 2]; 2]) -> u8 {
        let entry = leafs[0][0] as usize
            + (leafs[0][1] as usize) << 2
            + (leafs[1][0] as usize) << 8
            + (leafs[1][1] as usize) << 10;
        self.small_evolve_cache[entry]
    }
}

#[cfg(test)]
mod test {
    use super::{mk_small_evolve_cache};

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
}
