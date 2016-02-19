#![allow(dead_code)]

//use std::str::Chars;
use std::io::{self, Read};
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

#[derive(Debug, PartialEq, Eq)]
struct ParseError;
type Result<T> = result::Result<T, ParseError>;

#[derive(Debug, PartialEq, Eq)]
enum LineParse {
    RLEMeta(RLEMeta),
}

// TODO: Replace u64 by bignums
#[derive(Debug, PartialEq, Eq)]
struct RLEMeta {
    x: u64,
    y: u64,
    //TODO
    //rule: ...
}

#[derive(Debug, PartialEq, Eq)]
enum RLEToken {
    Run(usize, State),
    EndLine,
    EndBlock,
}

#[derive(Debug, PartialEq, Eq)]
enum State {Dead, Alive}

fn parse_rle_line(line: &str) -> LineParse {
    let mut rest = line;
    rest = rest.trim_left();
    match fst_char(rest) {
        Some('x') =>
            LineParse::RLEMeta(parse_rle_meta(rest).expect("FIXME: better error\
            handling")),
        _ => unimplemented!(),
    }
}

fn fst_char(s: &str) -> Option<char> {s.chars().next()}

fn digits_to_u64(x: &[u8]) -> u64 {
    u64::from_str_radix(str::from_utf8(x).unwrap(), 10).unwrap()
}

named!(uint<&[u8], u64>,
    map_res!(
        digit,
        // `unwrap` should never panic since `digit` only accepts ASCII
        // characters.
        |x| u64::from_str(str::from_utf8(x).unwrap())
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
fn test_parse_rle_meta_0() {
    assert_eq!(parse_rle_meta(" x = 3 , y = 8 , rule = ?"),
        Ok(RLEMeta {x: 3, y: 8}));
}

#[cfg(test)]
#[test]
fn test_parse_rle_meta_1() {
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
}

#[cfg(test)]
#[test]
fn test_parse_rle_meta_2() {
    assert_eq!(parse_rle_meta("x=3,y=8,rule=B3/S23"),
        Ok(RLEMeta{x:3,y:8}));
    assert_eq!(parse_rle_meta("x=33,y=27421,rule=B3/S23"),
        Ok(RLEMeta{x:33,y:27421}));
}
