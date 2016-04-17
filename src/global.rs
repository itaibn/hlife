
use evolve::*;

struct Pattern<'a, 'b:'a> {
    hl: &'a Hashlife<'b>,
    block: Block<'b>,
}

impl<'a, 'b> Pattern<'a, 'b> {
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

    fn step_pow2(&mut self, lognsteps: usize) {}
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
