//! Make a block out of parsed RLE code.

use std::ops::Range;

use ::Hashlife;
use block::Block;
use leaf::{Leaf, LEAF_SIZE, LEAF_Y_SHIFT, LEAF_X_SHIFT};
use super::parse::{RLE, RLEEncode, RLEToken, State};

fn expand_rle<A:Clone>(rle: &RLEEncode<A>) -> Vec<A> {
    use std::iter;
    rle.iter().flat_map(|&(n, ref t)| iter::repeat(t.clone()).take(n)).collect()
}

// matrix[y][x]
fn tokens_to_matrix(tokens: &[RLEToken]) -> Result<Vec<Vec<State>>, ()> {
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
                return Ok(matrix);
            }
        }
    }

    Err(())
}

pub fn block_from_rle<'a>(hl: &Hashlife<'a>, rle: &RLE) -> Result<Block<'a>,
    ()> {

    use std::cmp::max;

    let mut matrix = try!(tokens_to_matrix(&expand_rle(rle)));
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
    Ok(block_from_matrix(hl, res_depth, matrix))
}

pub fn block_from_matrix<'a>(hl: &Hashlife<'a>, depth: u32, matrix:
    Vec<&[State]>) -> Block<'a> {

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
                subblocks[i][j] = block_from_matrix(hl, depth-1,
                    submatrix);
            }
        }
        Block::Node(hl.node(subblocks))
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

/*
    state_to_bit(states[0][0])
    | state_to_bit(states[0][1]) << 1
    | state_to_bit(states[1][0]) << 4
    | state_to_bit(states[1][1]) << 5
*/
    let mut res: Leaf = 0;
    for y in 0..LEAF_SIZE {
        for x in 0..LEAF_SIZE {
            res |= (state_to_bit(states[y][x]) as Leaf) << (y * LEAF_Y_SHIFT
                + x * LEAF_X_SHIFT);
        }
    }
    res
}

#[test]
fn test_expand_rle() {
    // Test with the look-and-say sequence
    assert_eq!(expand_rle(&[(1, 1)]), [1]);
    assert_eq!(expand_rle(&[(2, 1)]), [1, 1]);
    assert_eq!(expand_rle(&[(1, 2), (1, 1)]), [2, 1]);
    assert_eq!(expand_rle(&[(1, 1), (1, 2), (2, 1)]), [1, 2, 1, 1]);
    assert_eq!(expand_rle(&[(3, 1), (2, 2), (1, 1)]), [1, 1, 1, 2, 2, 1]);
}

#[cfg(test)]
mod test {
    use super::block_from_rle;

    #[test]
    #[cfg(not(feature = "4x4_leaf"))]
    fn test_build_examples() {
        use format::parse::RLEToken::*;
        use format::parse::State::*;
        use block::Block;
        use ::Hashlife;

        //let tokens0 = vec![Run(1, Dead), Run(1, Alive), EndLine, Run(1, Alive),
        //    EndBlock];
        let tokens0 = [(1, State(Dead)), (1, State(Alive)), (1, EndLine),
            (1, State(Alive)), (1, EndBlock)];
        //let tokens1 = vec![Run(3, Alive), EndLine, EndLine, Run(1, Alive),
        //    EndBlock];
        let tokens1 = [(3, State(Alive)), (1, EndLine), (1, EndLine), (1,
            State(Alive)), (1, EndBlock)];
        let tokens2 = [(1, EndBlock)];
        // From failed format::write test
        let tokens3 = [(1, State(Dead)), (1, State(Dead)), (1, EndLine),
            (1, State(Dead)), (1, State(Dead)), (1, EndBlock)];

        Hashlife::with_new(|hl| {
            assert_eq!(block_from_rle(&hl, &tokens0), Ok(Block::Leaf(0x12)));
            let node = hl.node([[Block::Leaf(0x03), Block::Leaf(0x01)],
                [Block::Leaf(0x01), Block::Leaf(0x00)]]);
            assert_eq!(block_from_rle(&hl, &tokens1), Ok(Block::Node(node)));
            assert_eq!(block_from_rle(&hl, &tokens2), Ok(Block::Leaf(0x00)));
            assert_eq!(block_from_rle(&hl, &[(1, EndLine), (1, EndBlock)]),
                Ok(Block::Leaf(0x00)));
            assert_eq!(block_from_rle(&hl, &tokens3), Ok(Block::Leaf(0x00)));
        });
    }
}
