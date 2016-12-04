use ::Hashlife;
use block::{Block as RawBlock, Node as RawNode};
use leaf::{
    Leaf,
    LEAF_SIZE,
    QUARTER_LEAF_MASK,
    LEAF_Y_SHIFT,
    LEAF_X_SHIFT,
};
use util::{make_2x2, make_3x3};

/// A table containing the 2x2 center block after one generation for all
/// possible 4x4 blocks.
pub fn mk_small_evolve_cache() -> [u8; 1<<16] {
    let mut res = [0; 1<<16];
    let bitcount = [0, 1, 1, 2, 1, 2, 2, 3];
    for a in 0..8 {
        for b in 0..8 {
            for c in 0..8 {
                let entry = a | b << 4 | c << 8;
                let count = bitcount[a] + bitcount[b] + bitcount[c];
                let living =
                    count == 3 || (count == 4 && (b & 2 != 0));
                res[entry] = if living {1} else {0};
            }
        }
    }
    for x in 0..1<<16 {
        let mut evolve = 0;
        for i in 0..2 {
            for j in 0..2 {
                let subblock = (x >> (i+4*j)) & 0x777;
                evolve |= (res[subblock] & 1) << (i + 4*j);
            }
        }
        res[x] = evolve;
    }
    res
}

/// Given 2^(n+1)x2^(n+1) node `node`, progress it 2^(n-1) generations and
/// return 2^nx2^n block in the center. This is the main component of the
/// Hashlife algorithm.
pub fn evolve<'a>(hl: &Hashlife<'a>, node: RawNode<'a>) -> RawBlock<'a> {
    let elem = node.corners();

    node.evolve_cache().eval(move ||
        if node.node_of_leafs() {
            let elem_leafs = make_2x2(|i, j| elem[i][j].unwrap_leaf());
            RawBlock::Leaf(evolve_leaf(hl, elem_leafs))
        } else {
            let intermediates = make_3x3(|i, j| {
                // I don't know we need two separate `let`
                // statements, but the borrow checker complains if I
                // combine them.
                let subblock = subblock(hl, node, i as u8, j as u8);
                let subnode = subblock.unwrap_node();
                evolve(hl, subnode)
            });
            evolve_finish(hl, intermediates)
        }
    )
}

/// Evolve (3*2^n)x(3*2^n) block (encoded as a 3x3 array of 2^nx2^n blocks)
/// 2^(n-1) steps and return the 2^nx2^n block in the middle
#[cfg_attr(features = "inline", inline)]
fn evolve_finish<'a>(hl: &Hashlife<'a>, parts: [[RawBlock<'a>; 3]; 3]) ->
    RawBlock<'a> {

    let res_components = make_2x2(|i, j| {
        evolve(hl, hl.raw_node(make_2x2(|y, x| parts[i+y][j+x])))
    });
    hl.raw_node_block(res_components)
}

/// Given 2^(n+1)x2^(n+1) block, return 2^nx2^n subblock that's y*2^(n-1)
/// south and x*2^(n-1) east of the north-west corner.
///
/// Public for use in other modules in this crate; don't rely on it.
#[cfg_attr(features = "inline", inline)]
pub fn subblock<'a>(hl: &Hashlife<'a>, node: RawNode<'a>, y: u8, x: u8) ->
    RawBlock<'a> {

    debug_assert!(x < 3 && y < 3);
    let (x, y) = (x as usize, y as usize);

    if (x|y)&1 == 0 {
        node.corners()[y/2][x/2]
    } else if node.node_of_leafs() {
        subblock_leaf(hl, node, y, x)
    } else {
        subblock_node(hl, node, y, x)
    }
}

#[cfg_attr(features = "inline", inline)]
fn subblock_node<'a>(hl: &Hashlife<'a>, node: RawNode<'a>, y: usize, x: usize)
    -> RawBlock<'a> {

    //let (x, y) = (x as usize, y as usize);
    let components = make_2x2(|j, i| {
        let xx = i+x;
        let yy = j+y;
        node.corners()[yy/2][xx/2].unwrap_node().corners()[yy&1][xx&1]
    });
    hl.raw_node_block(components)
}

#[cfg_attr(features = "inline", inline)]
fn subblock_leaf<'a>(_: &Hashlife<'a>, node: RawNode<'a>, y: usize, x: usize) ->
    RawBlock<'a> {

    const HALF_LEAF: usize = LEAF_SIZE / 2;

    let mut output_leaf = 0;
    for j in 0..2 {
        for i in 0..2 {
            let yy = j+y;
            let xx = i+x;
            debug_assert!(xx < 4 && yy < 4);
            let source_leaf = node.corners()[yy / 2][xx / 2].unwrap_leaf();
            let source_shift = (yy&1) * HALF_LEAF * LEAF_Y_SHIFT
                + (xx&1) * HALF_LEAF * LEAF_X_SHIFT;
            let output_shift = j * HALF_LEAF * LEAF_Y_SHIFT
                + i * HALF_LEAF * LEAF_X_SHIFT;
            let cell = QUARTER_LEAF_MASK & (source_leaf >> source_shift);
            output_leaf |= cell << output_shift;
            println!("i {} j {} output_leaf {:x}", i, j, output_leaf);
        }
    }
    RawBlock::Leaf(output_leaf)
}

/// `evolve` specialized to when the corners are all leafs.
#[cfg(not(feature = "4x4_leaf"))]
#[inline]
fn evolve_leaf(hl: &Hashlife, leafs: [[Leaf; 2]; 2]) -> Leaf {
    debug_assert!(LEAF_SIZE == 2);
    let entry = leafs[0][0] as usize
        + ((leafs[0][1] as usize) << 2)
        + ((leafs[1][0] as usize) << 8)
        + ((leafs[1][1] as usize) << 10);
    hl.small_evolve_cache()[entry]
}

#[cfg(feature = "4x4_leaf")]
fn evolve_leaf(hl: &Hashlife, leafs: [[Leaf; 2]; 2]) -> Leaf {
    let small_evolve_cache = hl.small_evolve_cache();
    let e4x4 = |l: Leaf| small_evolve_cache[l as usize] as Leaf;

    let nw = leafs[0][0];
    let ne = leafs[0][1];
    let sw = leafs[1][0];
    let se = leafs[1][1];
    let n = ((nw >> 2) & 0x3333) | ((ne << 2) & 0xcccc);
    let s = ((sw >> 2) & 0x3333) | ((se << 2) & 0xcccc);
    let w = (nw >> 8) | (sw << 8);
    let e = (ne >> 8) | (se << 8);
    let c = (n >> 8) | (s << 8);

    macro_rules! step_vars {
        ( $( ($eD:ident, $D:expr) ),* ) => {
            $( let $eD = e4x4($D); )*
        }
    }

    step_vars! ((enw, nw), (en, n), (ene, ne), (ew, w), (ec,c), (ee, e),
        (esw, sw), (es, s), (ese, se));

    let eenw = e4x4(enw | en << 2 | ew << 8 | ec << 10);
    let eene = e4x4(en | ene << 2 | ec << 8 | ee << 10);
    let eesw = e4x4(ew | ec << 2 | esw << 8 | es << 10);
    let eese = e4x4(ec | ee << 2 | es << 8 | ese << 10);

    eenw | eene << 2 | eesw << 8 | eese << 10
}

/*
    fn leaf_step(&self, leafs: [[Leaf; 2]; 2], nstep: usize) -> Leaf {
        assert!(nstep < LEAF_SIZE);
        self.leaf_step_impl(leafs, nstep)
    }
*/

//    #[cfg(not(feature = "4x4_leaf"))]
//    fn leaf_step(&self, leafs: [[Leaf; 2]; 2], nstep: usize) -> Leaf {
//        if nstep == 0 {
//            leaf_step
    
#[cfg(not(feature = "4x4_leaf"))]
pub fn step_pow2<'a>(hl: &Hashlife<'a>, node: RawNode<'a>, lognsteps: usize) ->
    RawBlock<'a> {

    assert!(lognsteps <= node.lg_size() - 2);

    if lognsteps == node.lg_size() - 2 {
        evolve(hl, node)
    } else {
        let parts = make_3x3(|i, j| {
            subblock(hl,
                subblock(hl, node, i as u8, j as u8).unwrap_node(), 1, 1)
        });

        hl.raw_node_block(make_2x2(|y, x| {
            let around = hl.raw_node(make_2x2(|i, j| parts[y+i][x+j]));
            step_pow2(hl, around, lognsteps)
        }))
    }
}

#[cfg(feature = "4x4_leaf")]
pub fn step_pow2<'a>(hl: &Hashlife<'a>, node: RawNode<'a>, lognsteps: usize) ->
    RawBlock<'a> {

    unimplemented!()
}

#[cfg(test)]
mod test {
    use ::Hashlife;

    use super::mk_small_evolve_cache;

    #[test]
    fn test_small_evolve_cache() {
        let cache = mk_small_evolve_cache();
        macro_rules! test_cases {
            ( $($test:expr => $result:expr),* ) =>
                {{$(assert_eq!(cache[$test], $result);)*}};
        }
        test_cases! (
            0x0070 => 0x11,
            0x0e00 => 0x22,
            0x1630 => 0x23,
            0x0660 => 0x33,
            0xffff => 0x00
        )
    }

    #[test]
    fn test_evolve() {
        let input_rles: [&'static str; 1] = [
            //"bbo$boo$bbo!",
            "x = 8, y = 8, rule = B3/S23\n\
            3ob2o$bo2bobo$2obobo$bobob2o$obobob2o$2bo2b2o$ob2ob2o$bo2b3o!"
        ];
        let output_rles: [&'static str; 1] = [
            //"oo$oo!",
            //b"x = 4, y = 4, rule = B3/S23
            "o$b2o$o$o!"
        ];

        Hashlife::with_new(|hl| {
            for (input_rle, output_rle) in input_rles.iter()
                                                     .zip(output_rles.iter()) {
                let input = hl.raw_rle(input_rle);
                let output = hl.raw_rle(output_rle);

                assert_eq!(hl.raw_evolve(input.unwrap_node()), output)
            }
        });
    }

    #[cfg(not(feature = "4x4_leaf"))]
    #[test]
    fn test_step_pow2() {
        Hashlife::with_new(|hl| {
            let b = hl.raw_rle("2$6o!");
            let n = b.unwrap_node();
            assert_eq!(hl.raw_step_pow2(n, 1), hl.raw_evolve(n));
            assert_eq!(hl.raw_step_pow2(n, 0), hl.raw_rle("3o$3o!"));
            assert_eq!(hl.raw_step_pow2(n, 1), hl.raw_rle("3bo$2bo$2o!"));
        });
    }

    #[cfg(not(feature = "4x4_leaf"))]
    #[test]
    fn test_subblock_0() {
        Hashlife::with_new(|hl| {
            let b = hl.raw_rle("bo$bo$3o$o!");
            let n = b.unwrap_node();

            assert_eq!(hl.raw_subblock(n, 0, 0), hl.raw_rle("bo$bo!"));
            assert_eq!(hl.raw_subblock(n, 1, 0), hl.raw_rle("bo$oo!"));
            assert_eq!(hl.raw_subblock(n, 2, 0), hl.raw_rle("oo$o!"));
            assert_eq!(hl.raw_subblock(n, 0, 1), hl.raw_rle("o$o!"));
            assert_eq!(hl.raw_subblock(n, 1, 1), hl.raw_rle("o$oo!"));
            assert_eq!(hl.raw_subblock(n, 2, 1), hl.raw_rle("2o!"));
            assert_eq!(hl.raw_subblock(n, 0, 2), hl.raw_rle("!"));
            assert_eq!(hl.raw_subblock(n, 1, 2), hl.raw_rle("$o!"));
            assert_eq!(hl.raw_subblock(n, 2, 2), hl.raw_rle("o!"));
        });
    }

    #[test]
    fn test_subblock_1() {
        Hashlife::with_new(|hl| {
            let b = hl.raw_rle("2$7o!");
            let n = b.unwrap_node();

            assert_eq!(hl.raw_subblock(n, 0, 1), hl.raw_rle("2$4o!"));
            assert_eq!(hl.raw_subblock(n, 1, 0), hl.raw_rle("4o!"));
            assert_eq!(hl.raw_subblock(n, 0, 2), hl.raw_rle("2$3o!"));
        });
    }
}
