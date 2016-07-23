use std::str::{self, FromStr};

use nom::*;

macro_rules! assert_parse {
    ($str:expr => $parser:expr, $res:expr) => {
        match $parser($str) {
            IResult::Done(_, parsed) => assert_eq!(parsed, $res),
            err => panic!("Failed parse: {:?}", err),
        }
    }
}

named!(pub parse_file<&[u8], ParseOut>,
    map!(many0!(parse_line), process_lines)
);

named!(parse_line<&[u8], LineParse>,
    chain!(
        out: alt!(
              map!(mc_header, LineParse::MCHeader)
            | map!(rle_meta, LineParse::RLEMeta)
            | map!(rle_line, LineParse::RLELine)
            | map!(mc_line, LineParse::MCLine)
            | map!(comment, LineParse::Comment)
        )
        ~ line_ending,
        || out
    )
);

// Unstable type before I figure out the output of the parser
#[derive(Debug, PartialEq)]
pub enum ParseOut {
    RLE(RLEBuf),
    MC(Vec<MCLine>),
    Fail,
}
pub type RLEOut = RLEBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
enum LineParse {
    Comment(Comment),
    RLEMeta(RLEMeta),
    RLELine(RLEBuf),
    MCHeader(MCHeader),
    MCLine(MCLine),
}

fn process_lines(lines: Vec<LineParse>) -> ParseOut {
    use self::LineParse as LP;

    // Parse State. Short name to make pattern matching more convenient.
    enum PS {
        Start,
        RLE(Option<RLEMeta>, RLEBuf),
        MC(MCHeader, Vec<MCLine>),
    }
    
    let mut parse_state = PS::Start;

    for line in lines {
        parse_state = match (parse_state, line) {
            (ps, LP::Comment(_)) => {ps},
            (PS::Start, LP::RLEMeta(meta)) => {
                PS::RLE(Some(meta), Vec::new())
            }
            (PS::Start, LP::RLELine(tokens)) => {    
                PS::RLE(None, tokens)
            }
            (PS::Start, LP::MCHeader(header)) => {
                PS::MC(header, Vec::new())
            }
            (PS::RLE(m, mut cur_tokens), LP::RLELine(tokens)) => {
                cur_tokens.extend_from_slice(&tokens);
                PS::RLE(m, cur_tokens)
            }
            (PS::MC(h, mut lines), LP::MCLine(line)) => {
                lines.push(line);
                PS::MC(h, lines)
            }
            _ => {
                // Inappropiate line
                return ParseOut::Fail;
            }
        }
    }

    match parse_state {
        PS::RLE(_, tokens) => ParseOut::RLE(tokens),
        PS::MC(_, lines) => ParseOut::MC(lines),
        _ => ParseOut::Fail,
    }
}

named!(uint<&[u8], u64>,
    map_res!(
        digit,
        // `unwrap` should never panic since `digit` only accepts ASCII
        // characters.
        |x| u64::from_str(str::from_utf8(x).unwrap())
    )
);

#[derive(Clone, Debug, PartialEq, Eq)]
struct Comment;

named!(comment<&[u8], Comment>,
    map!(tuple!(opt!(space), opt!(tuple!(tag!("#"), not_line_ending))),
        |_| Comment
    )
);

pub type RLEBuf = RLEEncodeBuf<RLEToken>;
pub type RLE = RLEEncode<RLEToken>;
pub type RLEEncodeBuf<A> = Vec<(usize, A)>;
pub type RLEEncode<A> = [(usize, A)];

// TODO: Replace u64 by bignums
#[derive(Clone, Debug, PartialEq, Eq)]
struct RLEMeta {
    x: u64,
    y: u64,
    //TODO
    //rule: ...
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RLEToken {
    State(State),
    EndLine,
    EndBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {Dead, Alive}

named!(rle_meta<&[u8], RLEMeta>,
    chain!(
        space? ~
        tag!("x") ~
        space? ~
        tag!("=") ~
        space? ~
        x: uint ~
        space? ~
        tag!(",") ~
        space? ~
        tag!("y") ~
        space? ~
        tag!("=") ~
        space? ~
        y: uint ~
        space? ~
        tag!(",") ~
        space? ~
        tag!("rule") ~
        space? ~
        tag!("=") ~
        space? ~
        // Replace with rule grammar
        not_line_ending
        ,
        || {RLEMeta {x: x, y: y}}
    )
);

named!(rle_cell_state<&[u8], State>,
    alt!(
        map!(tag!("b"), |_| State::Dead) |
        map!(tag!("o"), |_| State::Alive)
    )
);

named!(rle_token<&[u8], RLEToken>,
    alt!(
          map!(rle_cell_state, RLEToken::State)
        | map!(tag!("$"), |_| RLEToken::EndLine)
        | map!(tag!("!"), |_| RLEToken::EndBlock)
    )
);

// FIXME: potential truncation with "as" 
named!(opt_num<&[u8], usize>,
    map!(opt!(uint), |x: Option<u64>| x.unwrap_or(1) as usize)
);

named!(rle_line<&[u8], RLEBuf>, many1!(tuple!(opt_num, rle_token)));

#[derive(Clone, Debug, PartialEq, Eq)]
struct MCHeader;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MCLine {
    Leaf(MCLeaf),
    Node(MCNode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MCLeaf(pub Vec<Vec<State>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MCNode(pub usize, pub usize, pub usize, pub usize, pub usize);

named!(mc_header<&[u8], MCHeader>,
    map!(
        tuple!(tag!("[M2]"), opt!(not_line_ending)),
        |_| MCHeader
    )
);

named!(mc_line<&[u8], MCLine>,
    alt!(
        map!(mc_leaf, MCLine::Leaf) |
        map!(mc_node, MCLine::Node)
    )
);

named!(mc_leaf<&[u8], MCLeaf>,
    map!(many_m_n!(8, 8, map!(
        tuple!(
            many0!(alt!(
                map!(tag!("."), |_| State::Dead) |
                map!(tag!("*"), |_| State::Alive)
            )),
            // The way this is written every row must end with "$" to be parsed
            // correctly, which I'm pretty sure is correct.
            tag!("$")
        ), |(row, _)| row
    )), MCLeaf)
);

// FIXME: potential truncation with "as" 
named!(mc_node<&[u8], MCNode>,
    chain!(
        d: uint ~ space ~
        b0: uint ~ space ~
        b1: uint ~ space ~
        b2: uint ~ space ~
        b3: uint ~ space,
        || MCNode(d as usize, b0 as usize, b1 as usize, b2 as usize, b3 as
            usize)
    )
);

#[test]
fn test_rle_line() {
    use self::RLEToken::*;
    use self::State::*;

    assert_parse!(b"bo$bbo$3o!" => rle_line,
//        vec![Run(1, Dead), Run(1, Alive), EndLine, Run(1, Dead), Run(1, Dead),
//            Run(1, Alive), EndLine, Run(3, Alive), EndBlock]
        vec![(1, State(Dead)), (1, State(Alive)), (1, EndLine), (1,
            State(Dead)), (1, State(Dead)), (1, State(Alive)), (1, EndLine), (3,
            State(Alive)), (1, EndBlock)]
    );
    assert_parse!(b"o2$o" => rle_line, vec![(1, State(Alive)), (2, EndLine), (1,
        State(Alive))]);
    //assert_parse!(b" 12b " => rle_line, vec![Run(12, Dead)]);
}

#[test]
fn test_parse_rle_meta() {
    assert_parse!(b" x = 3 , y = 8 , rule = ?" => rle_meta,
        RLEMeta {x: 3, y: 8});
    let res = rle_meta(b"x=3,y=8,rule=B3/23");
    match res {
        IResult::Done(_, _) => {},
        _ => {
            println!("{:?}", res);
            panic!();
        }
    }

    let res = rle_meta(b" x = 3 , y = 8 , rule = ?");
    match res {
        IResult::Done(_, _) => {},
        _ => {
            println!("{:?}", res);
            panic!();
        }
    }
    assert_parse!(b"x=3,y=8,rule=B3/S23" => rle_meta,
        RLEMeta {x: 3, y: 8});
    assert_parse!(b"x=33,y=27421,rule=B3/S23" => rle_meta,
        RLEMeta {x:33, y:27421});
}

#[test]
fn test_parse_line() {
    use self::State::*;

    assert_parse!(b"x=1,y=1,rule=B3/S23\n" => parse_line,
        LineParse::RLEMeta(RLEMeta {x: 1, y: 1}));
    assert_parse!(b"3bo\n" => parse_line, 
//        LineParse::RLELine(vec![RLEToken::Run(3, State::Dead), RLEToken::Run(1,
//                                    State::Alive)]));
        LineParse::RLELine(vec![(3, RLEToken::State(State::Dead)), (1,
            RLEToken::State(State::Alive))])
    );
    assert_parse!(b" #  Comment!\n" => parse_line, LineParse::Comment(Comment));
    assert_parse!(b"[M2]\n" => parse_line, LineParse::MCHeader(MCHeader));
    assert_parse!(b".*$..*$***$$$$$$\n" => parse_line,
        LineParse::MCLine(MCLine::Leaf(MCLeaf(
        vec![vec![Dead, Alive], vec![Dead, Dead, Alive], vec![Alive, Alive,
        Alive], vec![], vec![], vec![], vec![], vec![]]))));
}

#[test]
fn test_process_lines() {
    use self::ParseOut::*;
    use self::RLEToken::*;
    //use RLEToken::{Run, EndBlock, EndLine};
    use self::State::*;
    //use self::LineParse::*;
    let alive = State(Alive);
    let dead = State(Dead);

    let meta = LineParse::RLEMeta(RLEMeta {x: 5, y: 5});
    let line0 = LineParse::RLELine(vec![(1, alive), (1, EndLine), (1, alive)]);
    let line1 = LineParse::RLELine(vec![(3, dead), (1, alive), (1, EndBlock)]);

    assert_eq!(process_lines(vec![line0.clone()]), RLE(vec![(1, alive), (1,
        EndLine), (1, alive)]));
    assert_eq!(process_lines(vec![meta.clone(), line0.clone()]),
        RLE(vec![(1, alive), (1, EndLine), (1, alive)]));
    assert_eq!(process_lines(vec![line0.clone(), line1.clone()]),
        RLE(vec![(1, alive), (1, EndLine), (1, alive), (3, dead), (1, alive),
            (1, EndBlock)]));
    assert_eq!(process_lines(vec![line0, meta]), Fail);
}

#[test]
fn test_parse_file() {
    use self::ParseOut::*;
    use self::RLEToken::*;
    use self::State::*;

    assert_parse!(b"x = 5, y = 5, rule = B3/S23\nobo$3bo!\n" => parse_file,
//        vec![Run(1, Alive), Run(1, Dead), Run(1, Alive), EndLine, Run(3, Dead),
//             Run(1, Alive), EndBlock]);
        RLE(vec![(1, State(Alive)), (1, State(Dead)), (1, State(Alive)), (1,
             EndLine), (3, State(Dead)), (1, State(Alive)), (1, EndBlock)]));
    assert_parse!(b"x = 2, y = 2, rule = B3/S23\nbb$bb$!\n" => parse_file,
        RLE(vec![(1, State(Dead)), (1, State(Dead)), (1, EndLine), (1,
            State(Dead)), (1, State(Dead)), (1, EndLine), (1, EndBlock)]));
}

#[test]
fn test_comment() {
    assert_parse!(b"" => comment, Comment);
    assert_parse!(b"\n" => parse_line, LineParse::Comment(Comment));
}

#[test]
fn debug() {
    assert_parse!(b"[M2]" => mc_header, MCHeader);
    assert_parse!(b"[M2]\n" => parse_line, LineParse::MCHeader(MCHeader));

    named!(rle_or_mc<u8>, alt!(map!(rle_line, |_| 0) | map!(mc_leaf, |_| 1)));
    assert_parse!(b"3b" => rle_or_mc, 0);
    assert_parse!(b".*$..*$***$$$$$$" => rle_or_mc, 1);

    let expected = vec![(1, RLEToken::EndBlock)];
    println!("{:?}", parse_line(b"\n"));
    assert_parse!(b"!" => rle_line, expected);
    assert_parse!(b"!\n" => parse_line, LineParse::RLELine(expected.clone()));
    assert_parse!(b"!\n" => parse_file, ParseOut::RLE(expected.clone()));
    assert_parse!(b"!\n\n" => parse_file, ParseOut::RLE(expected.clone()));

    named!(tuple_opt<(Option<&[u8]>, Option<&[u8]>)>,
        tuple!(opt!(tag!("A")), opt!(tag!("B"))));
    println!("{:?}", tuple_opt(b""));
    assert_parse!(b"" => tuple_opt, (None, None));
}
