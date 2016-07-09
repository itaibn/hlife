use block::Block;
use evolve::Hashlife;
use util::make_2x2;

use super::parse::{MCLine, MCLeaf, MCNode};

pub fn build_mc<'a>(hl: &Hashlife<'a>, mclines: &[MCLine]) -> Result<Block<'a>,
    ()> {

    let mut table = Vec::new();

    for line in mclines {
        let new_block = match *line {
            MCLine::Leaf(ref leaf) => build_mc_leaf(hl, leaf),
            MCLine::Node(MCNode(d, b0, b1, b2, b3)) => {
                hl.node_block(make_2x2::<Block, _>(|i, j| {
                    let index: usize = match (i, j)
                        {(0,0) => b0, (0,1) => b1, (1,0) => b2, (1,1) => b3,
                         _ => unreachable!()};
                    if index == 0 {
                        hl.blank(d)
                    } else {
                        //*try!(table.get(index-1).ok_or(()))
                        table[index-1]
                    }
                }))
            }
        };
        table.push(new_block)
    }
    table.last().cloned().ok_or(())
}

fn build_mc_leaf<'a>(_: &Hashlife<'a>, _: &MCLeaf) -> Block<'a> {
    unimplemented!()
}