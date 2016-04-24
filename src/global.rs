
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

        let blank = self.hl.blank(lognsteps);
        let matrix = matrix(blank, self.block, lognsteps);
        let mut evolve_matrix = Vec::new();
        for i in 0 .. matrix.len()-1 {
            let mut evolve_row = Vec::new();
            for j in 0 .. matrix.len()-1 {
                let node_desc = [[matrix[i][j], matrix[i][j+1]],
                                 [matrix[i+1][j], matrix[i+1][j+1]]];
                evolve_row.push(self.hl.evolve(self.hl.node(node_desc)));
            }
            evolve_matrix.push(evolve_row);
        }
        print!("lognsteps: {}\nblank: {:?}\n", lognsteps, blank);
        for row in &matrix {
            println!("{:?}", &row);
        }
        println!("");
        for row in &evolve_matrix {
            println!("{:?}", &row);
        }
        self.block = matrix_to_block(self.hl, usize::MAX, lognsteps,
            evolve_matrix);
    }
}

impl<'a, 'b> Eq for Pattern<'a, 'b> { }

impl<'a, 'b> PartialEq for Pattern<'a, 'b> {
    fn eq(&self, other: &Self) -> bool {
        use std::mem::swap;

        let (mut a, mut b) = (self.block(), other.block());
        if a.depth() > b.depth() {
            swap(&mut a, &mut b);
        }
        while b.depth() > a.depth() {
            let corners = b.unwrap_node().corners();
            if !corners[0][1].is_blank()
                || !corners[1][0].is_blank()
                || !corners[1][1].is_blank() {
                return false;
            }
            b = corners[0][0];
        }
        a == b
    }
}

// Rethink name
fn matrix<'a>(blank: Block<'a>, block: Block<'a>, depth: usize)
    -> Vec<Vec<Block<'a>>> {

    use std::iter;

    debug_assert!(blank.depth() <= block.depth());

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
        Hashlife::with_new(|hl| {
            let mut blinker_in = parse(&hl, b"3o!");
            blinker_in.step(1);
            let blinker_out = parse(&hl, b"2bo$2bo$2bo2$!");
            if blinker_in != blinker_out {
                use format::write::format_rle;
                panic!("{}\n{}", format_rle(&blinker_in.block()),
                    format_rle(&blinker_out.block()));
            }
        });
    }
}
