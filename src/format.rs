#![allow(dead_code)]

//use std::str::Chars;
use std::io::{self, Read};
use std::result;
use std::str;

//use nom::space;
use nom::*;

/*
define_macro! opt_try {
    ( $e:expr ) => {match $e {
        Some(x) => x,
        None => {return None;},
    }}
}
*/

#[derive(Debug, PartialEq, Eq)]
struct ParseError;
type Result<T> = result::Result<T, ParseError>;

#[derive(Debug, PartialEq, Eq)]
enum LineParse {
    RLEMeta(RLEMeta),
}

// TODO: Replace u64 by bignums
#[derive(Debug, PartialEq, Eq)]
struct RLEMeta{
    x: u64,
    y: u64,
    //TODO
    //rule: ...
}

fn fst_char(s: &str) -> Option<char> {s.chars().next()}

fn parse_u64(s: &str) -> Result<(u64, &str)> {
    let mut chars = s.chars();
    let mut res = 0;
    let mut any = false;
    loop {
        match chars.clone().next().and_then(|c| c.to_digit(10)) {
            Some(d) => {res = 10*res + d as u64; any = true;},
            None => {break;},
        }
        chars.next();
    }
    if !any {
        Err(ParseError)
    } else {
        Ok((res, chars.as_str()))
    }
}

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

named!(rle_meta<&[u8], RLEMeta>,
    chain!(
        many0!(space) ~
        tag!("x") ~
        many0!(space) ~
        tag!("=") ~
        many0!(space) ~
        x: digit ~
        many0!(space) ~
        tag!(",") ~
        many0!(space) ~
        tag!("y") ~
        many0!(space) ~
        tag!("=") ~
        many0!(space) ~
        y: digit ~
        many0!(space) ~
        tag!(",") ~
        many0!(space) ~
        tag!("rule") ~
        many0!(space) ~
        tag!("=") ~
        many0!(space) ~
        // Replace with rule grammar
        many0!(not_line_ending)
        ,
        || {RLEMeta {
            x: u64::from_str_radix(str::from_utf8(x).unwrap(), 10).unwrap(),
            y: u64::from_str_radix(str::from_utf8(y).unwrap(), 10).unwrap()}}
    )
);

// Parse one line known to be RLE metainformation.
fn parse_rle_meta(line: &str) -> Result<RLEMeta> {
    match rle_meta(line.as_bytes()) {
        IResult::Done(_, res) => Ok(res),
        _ => Err(ParseError),
    }
/*
    let mut rest = line;
    if !rest.starts_with("x") {
        return Err(ParseError);
    }
    rest = rest[1..].trim_left();
    if !rest.starts_with("=") {
        return Err(ParseError);
    }
    rest = rest[1..].trim_left();
    let (x, rest_) = try!(parse_u64(rest));
    rest = rest_.trim_left();
    if !rest.starts_with(",") {
        return Err(ParseError);
    }
    rest = rest[1..].trim_left();
    if !rest.starts_with("y") {
        return Err(ParseError);
    }
    rest = rest[1..].trim_left();
    if !rest.starts_with("=") {
        return Err(ParseError);
    }
    rest = rest[1..].trim_left();
    let (y, rest_) = try!(parse_u64(rest));
    rest = rest_.trim_left();
    if !rest.starts_with(",") {
        return Err(ParseError);
    }
    rest = rest[1..].trim_left();
    if !rest.starts_with("rule") {
        return Err(ParseError);
    }
    rest = rest[4..].trim_left();
    if !rest.starts_with("=") {
        return Err(ParseError);
    }
    rest = &rest[1..];
    // Don't parse rule for now
    Ok(RLEMeta {x: x, y: y})
*/
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
}
