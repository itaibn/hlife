pub mod parse;
mod build_rle;
mod build_mc;
pub mod write;

use ::Hashlife;
use block::Block;

impl<'a> Hashlife<'a> {
    pub fn raw_block_from_bytes(&self, bytes: &[u8]) -> Result<Block<'a>, ()> {
        use self::parse::{parse_file, ParseOut};
        use self::build_rle::block_from_rle;
        use self::build_mc::build_mc;
        use nom::IResult;

        // TODO: Do this with less copying.
        let mut with_newline = bytes.to_vec();
        with_newline.push(b'\n');

        if let IResult::Done(b"", parse_out) =
                parse_file(&with_newline) {
            match parse_out {
                ParseOut::RLE(tokens) => block_from_rle(self, &tokens),
                ParseOut::MC(lines) => build_mc(self, &lines),
                ParseOut::Fail => Err(()),
            }
        } else {
            Err(())
        }
    }

    /// Simpler API for generating a block from a string; useful for quickly
    /// generating a specific block, as in testing.
    pub fn raw_rle(&self, pat: &'static str) -> Block<'a> {
        self.raw_block_from_bytes(pat.as_bytes()).unwrap()
    }
}

// Test for parsing error found in ::evolve tests and others.
#[test]
fn test_block_from_bytes() {
    use block::Block;
    use ::Hashlife;

    Hashlife::with_new(|hl| {
        assert!(hl.raw_block_from_bytes(b"bbo$boo$bbo!").is_ok());
        // From failed examples in `self::write::test::test_build_round_trip`
        assert_eq!(hl.raw_block_from_bytes(b"$!"), Ok(Block::Leaf(0)));
        let longer_test = b"x = 2, y = 2, rule = B3/S23\nbb$bb!";
        assert_eq!(hl.raw_block_from_bytes(longer_test), Ok(Block::Leaf(0)));
        // Test RLE lacking ending '!'
        assert_eq!(hl.raw_block_from_bytes(b"3o"), Err(()));
        let double_header = b"x=2,y=2,rule=B3/S23\nx=2,y=2,rule=B3/S23\nbb$bb!";
        assert_eq!(hl.raw_block_from_bytes(double_header), Err(()));

        // .mc
        assert_eq!(hl.raw_block_from_bytes(
            b"x=8,y=8,rule=B3/S23\nbo$2bo$3o2$3o$2bo$bo!"),
                   hl.raw_block_from_bytes(b"[M2]\n.*$..*$***$$***$..*$.*$$\n"));
        assert_eq!(
            hl.raw_block_from_bytes(b"[M2]\n.*$..*$***$$$$$$\n4 1 1 0 1"),
            hl.raw_block_from_bytes(
            b"x=16,y=16,rule=B3/S23\nbo7bo$2bo7bo$3o5b3o6$9bo$10bo$8b3o!"));
    });
}

// From failure in write::test::test_round_trip
#[test]
fn test_empty_rle() {
    Hashlife::with_new(|hl| {
        hl.raw_block_from_bytes(b"!\n").unwrap();
    });
}
