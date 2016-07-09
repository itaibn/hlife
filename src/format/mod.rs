pub mod parse;
mod build_rle;
mod build_mc;
pub mod write;

use evolve::Hashlife;
use block::Block;

impl<'a> Hashlife<'a> {
    pub fn block_from_bytes(&self, bytes: &[u8]) -> Result<Block<'a>, ()> {
        use self::parse::{parse_file, ParseOut};
        use self::build_rle::block_from_rle;
        use self::build_mc::build_mc;
        use nom::IResult;

        // TODO: Do this with less copying.
        let mut with_newline = bytes.to_vec();
        with_newline.push(b'\n');

        if let IResult::Done(_, parse_out) =
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
}

// Test for parsing error found in ::evolve tests.
#[test]
fn test_block_from_bytes() {
    use block::Block;
    use evolve::Hashlife;

    Hashlife::with_new(|hl| {
        assert!(hl.block_from_bytes(b"bbo$boo$bbo!").is_ok());
        // From failed examples in `self::write::test::test_build_round_trip`
        assert_eq!(hl.block_from_bytes(b"$!"), Ok(Block::Leaf(0)));
        let longer_test = b"x = 2, y = 2, rule = B3/S23\nbb$bb!";
        assert_eq!(hl.block_from_bytes(longer_test), Ok(Block::Leaf(0)));
        // Test RLE lacking ending '!'
        assert_eq!(hl.block_from_bytes(b"3o"), Err(()));
        let double_header = b"x=2,y=2,rule=B3/S23\nx=2,y=2,rule=B3/S23\nbb$bb!";
        assert_eq!(hl.block_from_bytes(double_header), Err(()));

        // .mc
        assert_eq!(hl.block_from_bytes(b"x=8,y=8\nbo$2bo$3o2$3o$2bo$bo!"),
                   hl.block_from_bytes(b"[M2]\n.*$..*$***$$***$..*$.*$$\n"));
    });
}
