use block::{Block, Leaf, LEAF_SIZE};
use super::parse::{RLEToken, State};

pub fn format_rle(block: &Block) -> String {
    let len = LEAF_SIZE << block.depth();
    tokens_to_string(len, len, matrix_to_tokens(block_to_matrix(block)))
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

fn matrix_to_tokens(matrix: Vec<Vec<State>>) -> Vec<RLEToken> {
    let mut res = Vec::new();
    let len = matrix.len();
    for (i, row) in matrix.into_iter().enumerate() {
        res.extend(row.into_iter().map(RLEToken::State));
        res.push(if i+1 < len {RLEToken::EndLine} else {RLEToken::EndBlock});
    }
    res
}

fn tokens_to_string(x: usize, y: usize, tokens: Vec<RLEToken>) -> String {
    let rle_compressed = rle_compress(tokens);

    let mut res = format!("x = {}, y = {}, rule = B3/S23\n", x, y);
    let mut line_len = 0;

    for (_, token) in rle_compressed {
        match token {
            RLEToken::State(State::Alive) => {
                res.push('o');
                line_len += 1;
            }
            RLEToken::State(State::Dead) => {
                res.push('b');
                line_len += 1;
            }
            RLEToken::EndLine => {
                res.push('$');
                line_len += 1;
            }
            RLEToken::EndBlock => {
                res.push('!');
                line_len += 1;
            }
        }
        if line_len >= 79 {
            res.push('\n');
            line_len = 0;
        }
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
        let cond = prev_.as_ref().map(|prev| *prev == token).unwrap_or(false);
        if cond {
            count += 1;
            // Temp
            res.push((0, token));
        } else {
            prev_.map(|prev| res.push((count, prev)));
            prev_ = Some(token);
        }
    }
    prev_.map(|prev| res.push((count, prev)));
    res
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
                "x = 2, y = 2, rule = B3/S23\noo$bb!\n");
            let b1 = Block::Node(bc.node([[b0, b0], [b0, b0]]));
            assert_eq!(format_rle(&b1),
                "x = 4, y = 4, rule = B3/S23\noooo$bbbb$oooo$bbbb!\n");
        });
    }
}
