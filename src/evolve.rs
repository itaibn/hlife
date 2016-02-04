use block::*;

struct Hashlife {
    table: CABlockCache,
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

impl Hashlife {
    pub fn new() -> Self {
        Hashlife {
            table: CABlockCache::new(),
            small_evolve_cache: mk_small_evolve_cache(),
        }
    }

    fn evolve<'a>(&'a mut self, block: BlockLink<'a>) -> Option<BlockLink<'a>> {
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
                        Node(_) => unimplemented!()
                    }
                }
            }
        )
        //None
    }

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
