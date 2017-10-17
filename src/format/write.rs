use std::cmp;

use ::Block;
use block::Block as RawBlock;
use leaf::{Leaf, LEAF_SIZE, LEAF_Y_SHIFT, LEAF_X_SHIFT};
use super::parse::{RLEToken, RLEBuf, State};

/// Transforms a block into RLE format. Panics if the block is ill-formed.
pub fn format_rle(block: &Block) -> String {
    raw_format_rle(&block.to_raw())
}

/// Raw version of `format_rle`. Used in implementation of Debug of
/// `block::Block`.
pub fn raw_format_rle(block: &RawBlock) -> String {
    //let len = 1 << block.lg_size();
    let _ = 1 << block.lg_size_verified().expect("Ill-formatted block");
    rle_to_string(matrix_to_rle(block_to_matrix(block)))
}

fn block_to_matrix(block: &RawBlock) -> Vec<Vec<State>> {
    match *block {
        RawBlock::Leaf(leaf) => leaf_to_matrix(leaf).iter().map(|row|
            row.to_vec()).collect(),
        RawBlock::Node(node) => {
            let corners = node.corners();
            merge_rows(
                merge_columns(block_to_matrix(&corners[0][0]),
                              block_to_matrix(&corners[0][1])),
                merge_columns(block_to_matrix(&corners[1][0]),
                              block_to_matrix(&corners[1][1]))
            )
        }
    }
}

#[inline]
fn leaf_to_matrix(leaf: Leaf) -> [[State; LEAF_SIZE]; LEAF_SIZE] {
    #[inline]
    fn bit(n: Leaf, bit: usize) -> State {
        match (n >> bit) & 1 {
            0 => State::Dead,
            1 => State::Alive,
            _ => unreachable!(),
        }
    }

    //[[bit(leaf, 0), bit(leaf, 1)], [bit(leaf, 4), bit(leaf, 5)]]
    let mut res = [[State::Dead; LEAF_SIZE]; LEAF_SIZE];
    for y in 0..LEAF_SIZE {
        for x in 0..LEAF_SIZE {
            res[y][x] = bit(leaf, y * LEAF_Y_SHIFT + x * LEAF_X_SHIFT);
        }
    }
    res
}

fn merge_rows<A>(mut top: Vec<Vec<A>>, mut bottom: Vec<Vec<A>>) -> Vec<Vec<A>> {
    top.append(&mut bottom);
    top
}

fn merge_columns<A>(left: Vec<Vec<A>>, right: Vec<Vec<A>>) -> Vec<Vec<A>> {
    debug_assert_eq!(left.len(), right.len());

    left.into_iter()
        .zip(right)
        .map(|(mut left_row, mut right_row)| {
            left_row.append(&mut right_row);
            left_row
        }).collect()
}

struct RLEData {
    rle: RLEBuf,
    xsize: usize,
    ysize: usize
}

fn matrix_to_rle(matrix: Vec<Vec<State>>) -> RLEData {
    let mut res: RLEBuf = Vec::new();
    let mut blank_lines = 0;
    let mut xmax = 0;
    let mut ymax = 1;

    for line in matrix {
        let mut xlen = 0;
        let mut run_val = State::Dead;
        let mut run_len = 0;
        let mut line_blank = true;

        for state in line {
            if state == run_val {
                run_len += 1;
            } else {
                if line_blank && blank_lines > 0 {
                    res.push((blank_lines, RLEToken::EndLine));
                    ymax += blank_lines;
                    blank_lines = 1;
                    line_blank = false;
                }
                if run_len > 0 {
                    res.push((run_len, RLEToken::State(run_val)));
                    xlen += run_len;
                }
                run_val = state;
                run_len = 1;
            }
        }

        if run_val != State::Dead {
            res.push((run_len, RLEToken::State(run_val)));
            xlen += run_len;
        }

        if line_blank {
            blank_lines += 1;
        }

        xmax = cmp::max(xmax, xlen);
    }
    res.push((1, RLEToken::EndBlock));
    RLEData {rle: res, xsize: xmax, ysize: ymax}
}

fn rle_to_string(rle_data: RLEData) -> String {
    fn token_len_to_string(len: usize, token: RLEToken) -> String {
        let mut res = if len == 1 {String::new()} else {len.to_string()};
        res.push(match token {
            RLEToken::State(State::Alive) => 'o',
            RLEToken::State(State::Dead) => 'b',
            RLEToken::EndLine => '$',
            RLEToken::EndBlock => '!',
        });
        res
    }

    let RLEData {rle, ysize: y, xsize: x} = rle_data;

    let mut res = format!("x = {}, y = {}, rule = B3/S23\n", x, y);
    let mut line_len = 0;

    for (len, token) in rle {
        let token_string = token_len_to_string(len, token);
        if line_len + token_string.len() > 79 {
            res.push('\n');
            line_len = 0;
        }
        line_len += token_string.len();
        res.push_str(&token_string);
    }
    if line_len > 0 {
        res.push('\n');
    }

    res
}

#[cfg(test)]
mod test {
    use super::format_rle;
    use ::Hashlife;

    #[test]
    fn test_round_trip() {
        let tests: [&str; 3] = ["!\n", "5bo!\n", "2$o!\n"];

        Hashlife::with_new(|hl| {
            for &test in &tests {
                let block = hl.rle(test);
                let reformatted = format_rle(&block);
                println!("{} -> {}", test, reformatted);
                assert_eq!(Ok(block), hl.block_from_bytes(
                    reformatted.as_bytes()));
            }
        });
    }

    // Test specific input-output pairs. Since I expect exact output will change
    // in later versions of this module this is not stable.
    #[cfg(not(feature = "4x4_leaf"))]
    #[test]
    fn test_instances() {
        //if cfg!(features = "4x4_leaf")

        Hashlife::with_new(|hl| {
            let b0 = hl.leaf(0x03);
            assert_eq!(format_rle(&b0),
                "x = 2, y = 1, rule = B3/S23\n2o!\n");
            let b1 = hl.node_block([[b0, b0], [b0, b0]]);
            assert_eq!(format_rle(&b1),
                "x = 4, y = 3, rule = B3/S23\n4o2$4o!\n");
        });
    }

    #[cfg(feature = "4x4_leaf")]
    #[test]
    fn test_instances() {
        Hashlife::with_new(|hl| {
            let b0 = hl.leaf(0x000f);
            assert_eq!(format_rle(&b0),
                "x = 4, y = 1, rule = B3/S23\n4o!\n");
            let b1 = hl.node_block([[b0, b0], [b0, b0]]);
            assert_eq!(format_rle(&b1),
                "x = 8, y = 5, rule = B3/S23\n8o4$8o!\n");
        });
    }
}
