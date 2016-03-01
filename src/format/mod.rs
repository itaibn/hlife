pub mod parse;
pub mod build;

use evolve::Hashlife;
use block::Block;

impl<'a> Hashlife<'a> {
    pub fn block_from_bytes(&mut self, bytes: &[u8]) -> Option<Block<'a>> {
        use self::parse::parse_file;
        use nom::IResult;

        if let IResult::Done(_, tokens) = parse_file(bytes) {
            Some(self.block_cache().block_from_rle(&tokens))
        } else {
            None
        }
    }
}
