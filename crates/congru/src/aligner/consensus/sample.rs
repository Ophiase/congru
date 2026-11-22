/// A small, fast, deterministic generator (SplitMix64): the same seed gives
/// the same consensus on every platform, without a dependency.
#[derive(Debug, Clone)]
pub(crate) struct Draws(u64);

impl Draws {
    pub(crate) const fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub(crate) fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        mixed ^ (mixed >> 31)
    }

    /// An index below `count`, which must not be zero.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "the remainder is below a usize"
    )]
    pub(crate) fn below(&mut self, count: usize) -> usize {
        (self.next() % count as u64) as usize
    }
}
