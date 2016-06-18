use block::{Block, Leaf, LEAF_SIZE};
use super::parse::{RLEToken, RLEBuf, State};

pub fn format_rle(block: &Block) -> String {
    let len = LEAF_SIZE << block.depth();
    matrix_to_string(len, len, block_to_matrix(block))
}

fn block_to_matrix(block: &Block) -> Vec<Vec<State>> {
    match *block {
        Block::Leaf(l) => leaf_to_matrix(l).iter().map(|row|
            row.to_vec()).collect(),
        Block::Node(ref n) => {
            let corners = n.corners();
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
    fn bit(n: u8, bit: usize) -> State {
        match (n >> bit) & 1 {
            0 => State::Dead,
            1 => State::Alive,
            _ => unreachable!(),
        }
    }

    [[bit(leaf, 0), bit(leaf, 1)], [bit(leaf, 4), bit(leaf, 5)]]
}

fn merge_rows<A>(mut top: Vec<Vec<A>>, mut bottom: Vec<Vec<A>>) -> Vec<Vec<A>> {
    top.append(&mut bottom);
    top
}

fn merge_columns<A>(left: Vec<Vec<A>>, right: Vec<Vec<A>>) -> Vec<Vec<A>> {
    debug_assert!(left.len() == right.len());

    left.into_iter()
        .zip(right)
        .map(|(mut left_row, mut right_row)| {
            left_row.append(&mut right_row);
            left_row
        }).collect()
}

fn matrix_to_rle(matrix: Vec<Vec<State>>) -> RLEBuf {
    let mut res: RLEBuf = Vec::new();
    let mut blank_lines = 0;
    for line in matrix {
        let mut run_val = State::Dead;
        let mut run_len = 0;
        let mut line_blank = true;
        for state in line {
            if state == run_val {
                run_len += 1;
            } else {
                if line_blank && blank_lines > 0 {
                    res.push((blank_lines, RLEToken::EndLine));
                    blank_lines = 1;
                    line_blank = false;
                }
                if run_len > 0 {
                    res.push((run_len, RLEToken::State(run_val)));
                }
                run_val = state;
                run_len = 1;
            }
        }
        if run_val != State::Dead {
            res.push((run_len, RLEToken::State(run_val)));
        }
        if line_blank {
            blank_lines += 1;
        }
    }
    res.push((1, RLEToken::EndBlock));
    res
}

fn matrix_to_tokens(matrix: Vec<Vec<State>>) -> Vec<RLEToken> {
    let mut res = Vec::new();
    let len = matrix.len();
    for (i, row) in matrix.into_iter().enumerate() {
        res.extend(row.into_iter().map(RLEToken::State));
        res.push(if i+1 < len {RLEToken::EndLine} else {RLEToken::EndBlock});
    }
    res
}

fn matrix_to_string(x: usize, y: usize, tokens: Vec<Vec<State>>) -> String {
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

    let rle_compressed = matrix_to_rle(tokens);

    let mut res = format!("x = {}, y = {}, rule = B3/S23\n", x, y);
    let mut line_len = 0;

    for (len, token) in rle_compressed {
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

fn rle_compress<A:Eq>(tokens: Vec<A>) -> Vec<(usize, A)> {
    let mut res = Vec::new();
    let mut prev_: Option<A> = None;
    let mut count = 0;

    for token in tokens {
        if prev_.as_ref() == Some(&token) {
            count += 1;
        } else {
            prev_.map(|prev| res.push((count, prev)));
            prev_ = Some(token);
            count = 1;
        }
    }
    prev_.map(|prev| res.push((count, prev)));
    res
}

#[test]
fn test_rle_compress() {
    assert_eq!(rle_compress(vec![1; 5]), vec![(5, 1)]);
    assert_eq!(rle_compress(vec![1, 2, 1, 1]), vec![(1, 1), (1, 2), (2, 1)]);
}

#[cfg(test)]
mod test {
    use super::format_rle;
    use evolve::Hashlife;

    #[test]
    fn test_round_trip() {
        let tests: [&[u8]; 3] = [b"!\n", b"5bo!\n", b"2$o!\n"];

        Hashlife::with_new(|hl| {
            for &test in &tests {
                let block = hl.block_from_bytes(test).unwrap();
                let reformatted = format_rle(&block);
                println!("{} -> {}", String::from_utf8(test.to_vec()).unwrap(),
                    reformatted);
                assert_eq!(Ok(block),
                    hl.block_from_bytes(reformatted.as_bytes()));
            }
        });
    }

    // Test specific input-output pairs. Since I expect exact output will change
    // in later versions of this module this is not stable.
    #[test]
    fn test_instances() {
        use block::Block;

        Hashlife::with_new(|hl| {
            let mut bc = hl.block_cache();
            let b0 = Block::Leaf(0x03);
            assert_eq!(format_rle(&b0),
                "x = 2, y = 2, rule = B3/S23\n2o!\n");
            let b1 = Block::Node(bc.node([[b0, b0], [b0, b0]]));
            assert_eq!(format_rle(&b1),
                "x = 4, y = 4, rule = B3/S23\n4o2$4o!\n");
        });
    }
}
