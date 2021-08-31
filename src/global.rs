
use num::{BigUint, One, FromPrimitive, Zero};

use ::{Block, Hashlife};
use util::{log2_upper_bigu, make_2x2};

/// Infinite pattern which is dead in all but a finite area.
#[derive(Debug)]
pub struct Pattern<'a> {
    block: Block<'a>,
    dead_space: BigUint,
}

impl<'a> Pattern<'a> {
    // `block` must be a node
    pub fn new(block: Block<'a>) -> Self {
        assert!(block.destruct().is_ok(), "Pattern block must be a node");
        Pattern {block: block, dead_space: BigUint::zero()}
    }

    pub fn block(&self) -> Block<'a> {
        self.block
    }

    pub fn step(&mut self, nsteps: u64) {
        self.step_bigu(&BigUint::from_u64(nsteps).unwrap())
    }

    pub fn step_bigu(&mut self, nsteps: &BigUint) {
        let new_length = self.length_bigu() + (nsteps << 1);
        let lg_size_needed = log2_upper_bigu(&new_length) as usize + 1;
        let mut block = self.block;
        while block.lg_size() < lg_size_needed {
            block = encase(self.hl(), block);
        }
        self.block = self.hl().step_bigu(block.unwrap_node(), nsteps);
        self.dead_space = (BigUint::one() << self.block.lg_size()) - new_length;
    }

/*
    fn step_pow2(&mut self, lognsteps: usize) {
        self.step_pow2_1(lognsteps);
    }

    fn step_pow2_1(&mut self, lognsteps: usize) {
        let new_length = self.length() + (1 << (1 + lognsteps));
        let lgsize_needed = log2_upper(new_length) as usize;
        while self.block.lg_size() < lgsize_needed {
            self.encase();
        }
        let reencase = encase(self.hl(), self.block);
        self.block = self.hl().step_pow2(reencase.unwrap_node(), lognsteps);
    }

    fn encase(&mut self) {
        let lg_size = self.block.lg_size();
        self.block = encase(self.hl(), self.block);
        self.dead_space += 1 << lg_size;
    }
*/

    fn length_bigu(&self) -> BigUint {
        (BigUint::one() << self.block.lg_size()) - (&self.dead_space << 1)
    }

    fn hl(&self) -> Hashlife<'a> {
        self.block.hashlife_instance()
    }
}

impl<'a> Eq for Pattern<'a> { }

impl<'a> PartialEq for Pattern<'a> {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::swap;

        let (mut a, mut b) = (self.block(), other.block());
        if a.lg_size() > b.lg_size() {
            swap(&mut a, &mut b);
        }
        while b.lg_size() > a.lg_size() {
            a = encase(self.hl(), a);
        }
        a == b
    }
}

fn encase<'a>(hl: Hashlife<'a>, b: Block<'a>) -> Block<'a> {
    // Assumes b is a node.
    let n = b.unwrap_node();
    hl.node_block(make_2x2(|y0, x0| {
        hl.node_block(make_2x2(|y1, x1| {
            let x = 2*x0 + x1;
            let y = 2*y0 + y1;
            if 0 < x && x < 3 && 0 < y && y < 3 {
                n.corners()[y-1][x-1]
            } else {
                hl.blank(b.lg_size() - 1)
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
    use ::Hashlife;

    fn parse<'a>(hl: Hashlife<'a>, bytes: &'static str) -> Pattern<'a> {
        Pattern::new(hl.rle(bytes))
    }

    #[test]
    fn test_blinker_1gen() {
        Hashlife::with_new(|hl| {
            let mut blinker_in = parse(hl, "2$2b3o!");
            blinker_in.step(1);
            let blinker_out = parse(hl, "$3bo$3bo$3bob!");
            if blinker_in != blinker_out {
                use format::write::format_rle;
                panic!("{}\n{}", format_rle(&blinker_in.block()),
                    format_rle(&blinker_out.block()));
            }
        });
    }
}
