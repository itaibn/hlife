use ::{Block, Hashlife};
use leaf::LG_LEAF_SIZE;
use util::try_make_2x2;

use super::build_rle::block_from_matrix;
use super::parse::{State, MCLine, MCLeaf, MCNode};

pub fn build_mc<'a>(hl: &Hashlife<'a>, mclines: &[MCLine]) -> Result<Block<'a>,
    ()> {

    let mut table = Vec::new();

    for line in mclines {
        let new_block = match *line {
            MCLine::Leaf(ref leaf) => build_mc_leaf(hl, leaf),
            MCLine::Node(MCNode(d, b0, b1, b2, b3)) => {
                let elems = try_make_2x2(|i, j| {
                    let index: usize = match (i, j)
                        {(0,0) => b0, (0,1) => b1, (1,0) => b2, (1,1) => b3,
                         _ => unreachable!()};
                    if index == 0 {
                        Ok(hl.blank(d-1))
                    } else {
                        Ok(*table.get(index-1).ok_or(())?)
                    }
                })?;
                hl.node_block(elems)
            }
        };
        debug_assert!(new_block.lg_size_verified().is_ok());
        table.push(new_block);
    }
    assert_eq!(table.len(), mclines.len());
    debug!("Table: {:?}", table);
    //println!("{:?}", table);
    table.last().cloned().ok_or(())
}

fn build_mc_leaf<'a>(hl: &Hashlife<'a>, leaf: &MCLeaf) -> Block<'a> {

    let full_leaf = leaf.0.iter().map(|row| {
        let mut new_row = row.clone();
        new_row.resize(8, State::Dead);
        new_row
    }).collect::<Vec<_>>();
    let matrix = full_leaf.iter().map(|row| &**row).collect();
    assert!(LG_LEAF_SIZE <= 3);
    block_from_matrix(hl, 3 - LG_LEAF_SIZE as u32, matrix)
}
