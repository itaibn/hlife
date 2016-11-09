/// Module for the leaf data structure in the block tree.
///
/// Leaf is the leaf data structure in a block. It is a type synonym for an integer type and stores
/// a `LEAF_SIZE x LEAF_SIZE` block of cells as a bit vector. LEAF_SIZE is always a power of 2, and
/// is equal to `2^LG_LEAF_SIZE`.  For each bit, 1 represents alive and 0 represents dead.  The bit
/// representing the (x, y) coordinate is the `(y*LEAF_Y_SHIFT + x*LEAF_X_SHIFT)`th least
/// significant bit. `LEAF_MASK` is a mask with all valid bits of the leaf set, and
/// `QUARTER_LEAF_MASK` is a mask for the top left (LEAF_SIZE/2) x (LEAF_SIZE/2) subblock.
///
/// Currently there are two configurations for Leaf: LEAF_SIZE is 2 by default, but can be set to 4
/// with the 4x4_leaf feature. With 4x4_leaf progressing a pattern is significantly more efficient,
/// but not all features work with it yet.

#[cfg(feature = "4x4_leaf")]
pub use self::leaf_4x4::{Leaf, LG_LEAF_SIZE, LEAF_MASK, QUARTER_LEAF_MASK};
#[cfg(not(feature = "4x4_leaf"))]
pub use self::leaf_2x2::{Leaf, LG_LEAF_SIZE, LEAF_MASK, QUARTER_LEAF_MASK};

/// Side length LEAF_SIZE
pub const LEAF_SIZE: usize = 1 << LG_LEAF_SIZE;
pub const LEAF_Y_SHIFT: usize = 4;
pub const LEAF_X_SHIFT: usize = 1;

#[cfg(not(feature = "4x4_leaf"))]
mod leaf_2x2 {
    // 01
    // 45
    pub type Leaf = u8;

    pub const LG_LEAF_SIZE: usize = 1;
    pub const LEAF_MASK: Leaf = 0x33;

    // For global::encase
    pub const QUARTER_LEAF_MASK: Leaf = 0x01;
}

#[cfg(feature = "4x4_leaf")]
mod leaf_4x4 {
    pub type Leaf = u16;

    pub const LG_LEAF_SIZE: usize = 2;
    pub const LEAF_MASK: Leaf = 0xffff;

    pub const QUARTER_LEAF_MASK: Leaf = 0x33;
}
