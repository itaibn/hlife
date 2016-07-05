
use block::Block;
use evolve::*;
use util::{log2_upper, make_2x2};

#[derive(Debug)]
pub struct Pattern<'a, 'b:'a> {
    hl: &'a Hashlife<'b>,
    block: Block<'b>,
    dead_space: u64,
}

impl<'a, 'b> Pattern<'a, 'b> {
    // `block` must be a node
    pub fn new(hl: &'a Hashlife<'b>, block: Block<'b>) -> Self {
        Pattern {hl: hl, block: block, dead_space: 0}
    }

    pub fn block(&self) -> Block<'b> {
        self.block
    }

    pub fn step(&mut self, mut nsteps: usize) {
        let mut pow2 = 0;
        // Maybe better in opposite order
        while nsteps > 0 {
            if nsteps & 1 > 0 {
                self.step_pow2(pow2);
            }
            nsteps /= 2;
            pow2 += 1;
        }
    }

    fn step_pow2(&mut self, lognsteps: usize) {
        self.step_pow2_1(lognsteps);
    }

    fn step_pow2_1(&mut self, lognsteps: usize) {
        let new_length = self.length() + (1 << (1 + lognsteps));
        let depth_needed = log2_upper(new_length / (LEAF_SIZE as u64)) as usize;
        while self.block.depth() < depth_needed {
            self.encase();
        }
        let reencase = encase(self.hl, self.block);
        self.block = self.hl.step_pow2(reencase.unwrap_node(), lognsteps);
    }

    fn encase(&mut self) {
        let depth = self.block.depth();
        self.block = encase(self.hl, self.block);
        self.dead_space += 1 << (depth-1);
    }

    fn length(&self) -> u64 {
        ((LEAF_SIZE as u64) << self.block.depth()) - 2 * self.dead_space
    }
}

impl<'a, 'b> Eq for Pattern<'a, 'b> { }

impl<'a, 'b> PartialEq for Pattern<'a, 'b> {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::swap;

        let (mut a, mut b) = (self.block(), other.block());
        if a.depth() > b.depth() {
            swap(&mut a, &mut b);
        }
        while b.depth() > a.depth() {
            a = encase(self.hl, a);
        }
        a == b
    }
}

fn encase<'a>(hl: &Hashlife<'a>, b: Block<'a>) -> Block<'a> {
    // Assumes b is a node.
    let n = b.unwrap_node();
    hl.node_block(make_2x2(|y0, x0| {
        hl.node_block(make_2x2(|y1, x1| {
            let x = 2*x0 + x1;
            let y = 2*y0 + y1;
            if 0 < x && x < 3 && 0 < y && y < 3 {
                n.corners()[y-1][x-1]
            } else {
                hl.blank(b.depth() - 1)
            }
        }))
    }))
    // Leaf case (for later use):
    //let shift = (1-y)*LEAF_SIZE*(LEAF_SIZE/2) + (1-x)*(LEAF_SIZE/2);
    //let part = QUARTER_LEAF_MASK & (l >> shift);
    //Block::Leaf(part << (y*LEAF_SIZE*(LEAF_SIZE/2) + x*(LEAF_SIZE/2)))
}

#[cfg(test)]
mod test {
    use super::Pattern;
    use evolve::Hashlife;

    fn parse<'a, 'b>(hl: &'a Hashlife<'b>, bytes: &[u8]) -> Pattern<'a, 'b> {
        Pattern::new(hl, hl.block_from_bytes(bytes).unwrap())
    }

    #[test]
    fn test_blinker_1gen() {
        Hashlife::with_new(|hl| {
            let mut blinker_in = parse(&hl, b"$3o!");
            blinker_in.step(1);
            let blinker_out = parse(&hl, b"bo$bo$bo!");
            if blinker_in != blinker_out {
                use format::write::format_rle;
                panic!("{}\n{}", format_rle(&blinker_in.block()),
                    format_rle(&blinker_out.block()));
            }
        });
    }
}
