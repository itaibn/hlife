#![allow(dead_code)]

use std::result;
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
        )
        ~ line_ending,
        || out
    )
);

// Temp type before I figure out the output of the parser
pub type ParseOut = RLEOut;
pub type RLEOut = Vec<RLEToken>;

struct ParseError;
type Result<T> = result::Result<T, ParseError>;

#[derive(Clone, Debug, PartialEq, Eq)]
enum LineParse {
    Comment(Comment),
    RLEMeta(RLEMeta),
    RLELine(Vec<RLEToken>),
}

// TODO: Return error instead of panicking.
fn process_lines(lines: Vec<LineParse>) -> ParseOut {
    // For now, assume the format is RLE
    let mut cur_meta: Option<Option<RLEMeta>> = None;
    let mut cur_tokens = Vec::new();

    for line in lines {
        match line {
            LineParse::Comment(_) => {},
            LineParse::RLEMeta(ref meta) => {
                if cur_meta.is_some() {
                    panic!("RLE metainformation in inappropiate location");
                } else {
                    cur_meta = Some(Some(meta.clone()));
                }
            }
            LineParse::RLELine(ref tokens) => {    
                // Make clippy happy, and use or_else instead of or
                cur_meta = cur_meta.or_else(|| Some(None));
                cur_tokens.extend_from_slice(tokens);
            }
        }
    }

    // Turn tokens to output.
    cur_tokens
}

type Comment = ();

// TODO: Replace u64 by bignums
#[derive(Clone, Debug, PartialEq, Eq)]
struct RLEMeta {
    x: u64,
    y: u64,
    //TODO
    //rule: ...
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RLEToken {
    Run(usize, State),
    EndLine,
    EndBlock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {Dead, Alive}

named!(uint<&[u8], u64>,
    map_res!(
        digit,
        // `unwrap` should never panic since `digit` only accepts ASCII
        // characters.
        |x| u64::from_str(str::from_utf8(x).unwrap())
    )
);

named!(comment<&[u8], ()>,
    map!(tuple!(space, opt!(tuple!(tag!("#"), not_line_ending))),
        |_| ()
    )
);

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

fn rle_cell_state(input: &[u8]) -> IResult<&[u8], State> {
    if input.len() == 0 {
        IResult::Incomplete(Needed::Size(1))
    } else {
        match input[0] {
            b'b' => IResult::Done(&input[1..], State::Dead),
            b'o' => IResult::Done(&input[1..], State::Alive),
            _ => IResult::Error(Err::Position(ErrorKind::Tag, input)),
        }
    }
}

named!(rle_token<&[u8], RLEToken>,
    alt!(
        chain!(count: uint? ~ state: rle_cell_state,
            || RLEToken::Run(count.unwrap_or(1) as usize, state))
        | map!(tag!("$"), |_| RLEToken::EndLine)
        | map!(tag!("!"), |_| RLEToken::EndBlock)
    )
);

named!(rle_line<&[u8], Vec<RLEToken> >, many0!(rle_token));

#[cfg(test)]
#[test]
fn test_rle_line() {
    use self::RLEToken::*;
    use self::State::*;

    assert_parse!(b"bo$bbo$3o!" => rle_line,
        vec![Run(1, Dead), Run(1, Alive), EndLine, Run(1, Dead), Run(1, Dead),
            Run(1, Alive), EndLine, Run(3, Alive), EndBlock]
    );
}

// Parse one line known to be RLE metainformation.
fn parse_rle_meta(line: &str) -> Result<RLEMeta> {
    match rle_meta(line.as_bytes()) {
        IResult::Done(_, res) => Ok(res),
        _ => Err(ParseError),
    }
}

#[cfg(test)]
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