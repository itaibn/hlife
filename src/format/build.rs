//! Make a block out of parsed RLE code.

use std::ops::Range;

use block::{CABlockCache, Block, Leaf, LEAF_SIZE};
use super::parse::{RLE, RLEToken, State};

fn expand_rle<A:Clone>(rle: &[(usize, A)]) -> Vec<A> {
    use std::iter;
    rle.iter().flat_map(|&(n, ref t)| iter::repeat(t.clone()).take(n)).collect()
}

fn tokens_to_matrix(tokens: &[RLEToken]) -> Vec<Vec<State>> {
    let mut matrix = Vec::new();
    let mut cur_line = Vec::new();

    for token in tokens {
        match *token {
            RLEToken::State(state) => {
                cur_line.push(state);
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

    panic!("RLE with no end.")
}

impl<'a> CABlockCache<'a> {
    pub fn block_from_rle(&mut self, rle: &RLE) -> Block<'a> {
        use std::cmp::max;

        let mut matrix = tokens_to_matrix(&expand_rle(rle));
        let max_row_len = matrix.iter().map(|row| row.len()).max().unwrap_or(0);
        let max_side = max(max_row_len, matrix.len());
        let res_side: usize = max(max_side, LEAF_SIZE).next_power_of_two();
        let res_depth = (res_side / LEAF_SIZE).trailing_zeros();

        for row in &mut matrix {
            row.resize(res_side, State::Dead);
        }

        let empty_row = vec![State::Dead; res_side];
        matrix.resize(res_side, empty_row);

        let matrix = matrix.iter().map(|row| &**row).collect();
        //println!("depth {}", res_depth);
        self.block_from_matrix(res_depth, matrix)
    }

    fn block_from_matrix(&mut self, depth: u32, matrix: Vec<&[State]>) ->
        Block<'a> {

        assert_eq!(matrix.len(), LEAF_SIZE << depth);
        for row in &matrix {assert_eq!(row.len(), LEAF_SIZE << depth);}

        if depth == 0 {
            Block::Leaf(states_to_leaf(&matrix))
        } else {
            let mut subblocks = [[Block::Leaf(0); 2]; 2];
            // Side-length of subblock.
            let slen = LEAF_SIZE << (depth - 1);
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

#[test]
fn test_expand_rle() {
    // Test with the look-and-say sequence
    assert_eq!(expand_rle(&[(1, 1)]), vec![1]);
    assert_eq!(expand_rle(&[(2, 1)]), vec![1, 1]);
    assert_eq!(expand_rle(&[(1, 2), (1, 1)]), vec![2, 1]);
    assert_eq!(expand_rle(&[(1, 1), (1, 2), (2, 1)]), vec![1, 2, 1, 1]);
    assert_eq!(expand_rle(&[(3, 1), (2, 2), (1, 1)]), vec![1, 1, 1, 2, 2, 1]);
}

#[cfg(test)]
mod test {
    #[test]
    fn build_leaf() {
        use format::parse::RLEToken::*;
        use format::parse::State::*;
        use block::{CABlockCache, Block};

        //let tokens0 = vec![Run(1, Dead), Run(1, Alive), EndLine, Run(1, Alive),
        //    EndBlock];
        let tokens0 = vec![(1, State(Dead)), (1, State(Alive)), (1, EndLine),
            (1, State(Alive)), (1, EndBlock)];
        //let tokens1 = vec![Run(3, Alive), EndLine, EndLine, Run(1, Alive),
        //    EndBlock];
        let tokens1 = vec![(3, State(Alive)), (1, EndLine), (1, EndLine), (1,
            State(Alive)), (1, EndBlock)];
        let tokens2 = vec![(1, EndBlock)];

        CABlockCache::with_new(|mut cache| {
            assert_eq!(cache.block_from_rle(&tokens0), Block::Leaf(0x12));
            let node = cache.new_block([[Block::Leaf(0x03), Block::Leaf(0x01)],
                [Block::Leaf(0x01), Block::Leaf(0x00)]]);
            assert_eq!(cache.block_from_rle(&tokens1), Block::Node(node));
            assert_eq!(cache.block_from_rle(&tokens2), Block::Leaf(0x00));
        });
    }
}
