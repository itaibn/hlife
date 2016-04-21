
use evolve::*;

pub struct Pattern<'a, 'b:'a> {
    hl: &'a Hashlife<'b>,
    block: Block<'b>,
}

impl<'a, 'b> Pattern<'a, 'b> {
    pub fn new(hl: &'a Hashlife<'b>, block: Block<'b>) -> Self {
        Pattern {hl: hl, block: block}
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
        use std::usize;

        let subdivide_depth = lognsteps + 1;
        let blank = self.hl.blank(subdivide_depth);
        let matrix = matrix(blank, self.block, subdivide_depth);
        let evolve_matrix = matrix.iter()
            .map(|row| row.iter().map(|entry|
                self.hl.evolve(entry.unwrap_node())
            ).collect::<Vec<_>>()
            ).collect::<Vec<_>>();
        self.block = matrix_to_block(self.hl, usize::MAX, lognsteps,
            evolve_matrix);
    }
}

// Rethink name
fn matrix<'a>(blank: Block<'a>, block: Block<'a>, depth: usize)
    -> Vec<Vec<Block<'a>>> {

    use std::iter;

    debug_assert!(blank.depth() == block.depth());

    let len = 1 << (block.depth() - depth);
    let mut res = vec![vec![blank; len + 2]];
    res.extend(iter::repeat(vec![blank]).take(len));
    extend_with_matrix(block, &mut res[1..], depth);
    for row in &mut res[1..] {
        row.push(blank);
    }
    res.push(vec![blank; len + 2]);
    res
}

fn extend_with_matrix<'a>(block: Block<'a>, matrix: &mut [Vec<Block<'a>>],
    depth: usize) {
    
    debug_assert!(block.depth() >= depth);
    debug_assert!(matrix.len() == 1 << (block.depth() - depth));
    if block.depth() == depth {
        matrix[0].push(block);
    } else {
        let half = matrix.len() / 2;
        let (top, bottom) = matrix.split_at_mut(half);
        let node = block.unwrap_node();
        for (row, buffer) in node.corners().iter().zip([top,
            bottom].iter_mut()) {

            for &subblock in row {
                extend_with_matrix(subblock, &mut *buffer, depth);
            }
        }
    }
}

// Note: Very similar code in format::build
fn matrix_to_block<'a>(hl: &Hashlife<'a>, layers_: usize, in_depth: usize,
    mut matrix: Vec<Vec<Block<'a>>>) -> Block<'a> {

    use std::usize;
    use std::cmp;

    let layers;
    if layers_ == usize::MAX {
        let max_row = matrix.iter().map(|row| row.len()).max().unwrap_or(0);
        layers = cmp::max(max_row, matrix.len());
    } else {
        layers = layers_;
    }

    matrix.resize(1 << layers, vec![]);
    for row in &mut matrix {
        row.resize(1 << layers, hl.blank(in_depth));
    }

    if layers == 0 {
        return matrix[0][0];
    }

    let mut top_square = [[Block::Leaf(0); 2]; 2];
    let half = 1 << (layers - 1);
    for i in 0..2 {
        for j in 0..2 {
            let submatrix = matrix[i*half .. (i+1)*half].iter()
                .map(|row| row[j*half .. (j+1)*half].to_vec())
                .collect::<Vec<_>>();
            top_square[i][j] = matrix_to_block(hl, layers-1, in_depth,
                submatrix);
        }
    }
    Block::Node(hl.node(top_square))
}

#[cfg(test)]
mod test {
    use super::Pattern;
    use evolve::Hashlife;

    fn parse<'a, 'b>(hl: &'a Hashlife<'b>, bytes: &[u8]) -> Pattern<'a, 'b> {
        Pattern::new(&hl, hl.block_from_bytes(bytes).unwrap())
    }

    #[test]
    fn test_blinker_1gen() {
        Hashlife::with_new(|mut hl| {
            let blinker_in = parse(&hl, b"3b!");
            let blinker_out = parse(&hl, b"2ob$2ob$2ob2$!");
            blinker_in.step(1);
            assert_eq!(blinker_in.block(), blinker_out.block());
        });
    }
}
