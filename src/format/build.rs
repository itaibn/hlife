//! Make a block out of parsed RLE code.

#![allow(dead_code)]

use std::ops::Range;

use block::{CABlockCache, Block, Leaf, LEAF_SIZE};
use super::parse::{RLEToken, State};

fn tokens_to_matrix(tokens: &[RLEToken]) -> Vec<Vec<State>> {
    let mut matrix = Vec::new();
    let mut cur_line = Vec::new();

    for token in tokens {
        match *token {
            RLEToken::Run(length, state) => {
                // Needs to be a separate statement to satisfy the borrow
                // checker
                let len = cur_line.len();
                cur_line.resize(len + length, state);
            }
            RLEToken::EndLine => {
                matrix.push(cur_line);
                cur_line = Vec::new();
            }
            RLEToken::EndBlock => {
                matrix.push(cur_line);
                return matrix;
            }
        }
    }

    panic!("RLE with not end.")
}

impl<'a> CABlockCache<'a> {
    pub fn block_from_rle(&mut self, rle: &[RLEToken]) -> Block<'a> {
        use std::cmp::max;

        let mut matrix = tokens_to_matrix(rle);
        let max_row_len = matrix.iter().map(|row| row.len()).max().unwrap();
        let max_side = max(max_row_len, matrix.len());
        let res_side: usize = max_side.next_power_of_two();
        let res_depth = (res_side / LEAF_SIZE).trailing_zeros();

        for row in &mut matrix {
            row.resize(res_side, State::Dead);
        }

        let empty_row = vec![State::Dead; res_side];
        matrix.resize(res_side, empty_row);

        let matrix = matrix.iter().map(|row| &**row).collect();
        self.block_from_matrix(res_depth, matrix)
    }

    fn block_from_matrix(&mut self, depth: u32, matrix: Vec<&[State]>) ->
        Block<'a> {

        if depth == 0 {
            Block::Leaf(states_to_leaf(&matrix))
        } else {
            let mut subblocks = [[Block::Leaf(0); 2]; 2];
            // Side-length of subblock.
            let slen = 1 << (depth - 1);
            for i in 0..2 {
                for j in 0..2 {
                    let submatrix = submatrix(&matrix,
                                              i*slen..(i+1)*slen,
                                              j*slen..(j+1)*slen);
                    subblocks[i][j] = self.block_from_matrix(depth-1,
                        submatrix);
                }
            }
            Block::Node(self.new_block(subblocks))
        }
    }
}

fn submatrix<'a, T>(matrix: &[&'a [T]], outer: Range<usize>, inner:
    Range<usize>) -> Vec<&'a [T]> {

    matrix[outer].iter().map(|row| &row[inner.clone()]).collect()
}

fn states_to_leaf(states: &[&[State]]) -> Leaf {
    fn state_to_bit(state: State) -> u8 {
        match state {
            State::Dead => 0,
            State::Alive => 1,
        }
    }

    assert!(states.len() == LEAF_SIZE && states.iter().all(|row| row.len() ==
        LEAF_SIZE));

    state_to_bit(states[0][0])
    | state_to_bit(states[0][1]) << 1
    | state_to_bit(states[1][0]) << 4
    | state_to_bit(states[1][1]) << 5
}

#[cfg(test)]
mod test {
    #[test]
    fn build_leaf() {
        use format::parse::RLEToken::*;
        use format::parse::State::*;
        use block::{CABlockCache, Leaf, Block};

        let tokens = vec![Run(1, Dead), Run(1, Alive), EndLine, Run(1, Alive),
            EndBlock];

        CABlockCache::with_block_cache(|mut cache| {
            let block = cache.block_from_rle(&tokens);
            match block {
                Block::Leaf(0x12) => {/* Good */},
                _ => panic!("{:?}", block),
            }
        });
    }
}
