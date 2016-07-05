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
              map!(comment, LineParse::Comment)
            | map!(rle_meta, LineParse::RLEMeta)
            | map!(rle_line, LineParse::RLELine)
            | map!(mc_header, |_| LineParse::MCHeader)
        )
        ~ line_ending,
        || out
    )
);

// Temp type before I figure out the output of the parser
pub type ParseOut = Result<RLEOut, ()>;
pub type RLEOut = RLEBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
enum LineParse {
    Comment(Comment),
    RLEMeta(RLEMeta),
    RLELine(RLEBuf),
    MCHeader,
}

// TODO: Return error instead of panicking.
fn process_lines(lines: Vec<LineParse>) -> ParseOut {
    // For now, assume the format is RLE
    let mut cur_meta: Option<Option<RLEMeta>> = None;
    let mut cur_tokens: RLEBuf = Vec::new();

    for line in lines {
        match line {
            LineParse::Comment(_) => {},
            LineParse::RLEMeta(ref meta) => {
                if cur_meta.is_some() {
                    // "RLE metainformation in inappropiate location"
                    return Err(());
                } else {
                    cur_meta = Some(Some(meta.clone()));
                }
            }
            LineParse::RLELine(ref tokens) => {    
                // Make clippy happy, and use or_else instead of or
                cur_meta = cur_meta.or_else(|| Some(None));
                cur_tokens.extend_from_slice(tokens);
            }
            LineParse::MCHeader => {
                // ".mc format not implemented"
                return Err(())
            }
        }
    }

    // Turn tokens to output.
    Ok(cur_tokens)
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
    map!(tuple!(space, opt!(tuple!(tag!("#"), not_line_ending))),
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

named!(opt_num<&[u8], usize>,
    map!(opt!(uint), |x: Option<u64>| x.unwrap_or(1) as usize)
);

named!(rle_line<&[u8], RLEBuf>, many0!(tuple!(opt_num, rle_token)));

named!(mc_header<&[u8], ()>,
    map!(
        tuple!(tag!("[M2]"), not_line_ending),
        |_| ()
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
    assert_parse!(b"x=1,y=1,rule=B3/S23\n" => parse_line,
        LineParse::RLEMeta(RLEMeta {x: 1, y: 1}));
    assert_parse!(b"3bo\n" => parse_line, 
//        LineParse::RLELine(vec![RLEToken::Run(3, State::Dead), RLEToken::Run(1,
//                                    State::Alive)]));
        LineParse::RLELine(vec![(3, RLEToken::State(State::Dead)), (1,
            RLEToken::State(State::Alive))])
    );
    assert_parse!(b" #  Comment!\n" => parse_line, LineParse::Comment(Comment));
}

#[test]
fn test_process_lines() {
    use self::RLEToken::*;
    //use RLEToken::{Run, EndBlock, EndLine};
    use self::State::*;
    //use self::LineParse::*;
    let alive = State(Alive);
    let dead = State(Dead);

    let meta = LineParse::RLEMeta(RLEMeta {x: 5, y: 5});
    let line0 = LineParse::RLELine(vec![(1, alive), (1, EndLine), (1, alive)]);
    let line1 = LineParse::RLELine(vec![(3, dead), (1, alive), (1, EndBlock)]);

    assert_eq!(process_lines(vec![line0.clone()]), Ok(vec![(1, alive), (1,
        EndLine), (1, alive)]));
    assert_eq!(process_lines(vec![meta.clone(), line0.clone()]),
        Ok(vec![(1, alive), (1, EndLine), (1, alive)]));
    assert_eq!(process_lines(vec![line0.clone(), line1.clone()]),
        Ok(vec![(1, alive), (1, EndLine), (1, alive), (3, dead), (1, alive), (1,
            EndBlock)]));
    // Current implementation panics
    //assert_eq!(process_lines(vec![line0, meta], /* Failure */))
}

#[test]
fn test_parse_file() {
    use self::RLEToken::*;
    use self::State::*;

    assert_parse!(b"x = 5, y = 5, rule = B3/S23\nobo$3bo!\n" => parse_file,
//        vec![Run(1, Alive), Run(1, Dead), Run(1, Alive), EndLine, Run(3, Dead),
//             Run(1, Alive), EndBlock]);
        Ok(vec![(1, State(Alive)), (1, State(Dead)), (1, State(Alive)), (1,
             EndLine), (3, State(Dead)), (1, State(Alive)), (1, EndBlock)]));
    assert_parse!(b"x = 2, y = 2, rule = B3/S23\nbb$bb$!\n" => parse_file,
        Ok(vec![(1, State(Dead)), (1, State(Dead)), (1, EndLine), (1,
            State(Dead)), (1, State(Dead)), (1, EndLine), (1, EndBlock)]));
}
