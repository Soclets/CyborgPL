//! SplitMix64: small, fast, and the same sequence on every machine.
//! One seed gives one program; nothing else feeds the generator.

#[derive(Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A child stream: the parent moves on by one draw, the child is independent.
    pub fn split(&mut self) -> Rng {
        Rng(self.next_u64() ^ 0x6A09_E667_F3BC_C909)
    }

    /// Uniform in [0, n). n must be > 0.
    pub fn below(&mut self, n: u64) -> u64 {
        // Lemire's multiply-shift; the tiny bias is irrelevant here and it is deterministic.
        ((self.next_u64() as u128 * n as u128) >> 64) as u64
    }

    /// Uniform in [lo, hi], both inclusive, over i128 (every value we need fits).
    pub fn range(&mut self, lo: i128, hi: i128) -> i128 {
        assert!(lo <= hi, "empty range {lo}..={hi}");
        let span = (hi - lo) as u128 + 1;
        if span > u64::MAX as u128 {
            let r = ((self.next_u64() as u128) << 64) | self.next_u64() as u128;
            return lo + (r % span) as i128;
        }
        lo + self.below(span as u64) as i128
    }

    pub fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }

    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len() as u64) as usize]
    }

    /// Index chosen by weight. At least one weight must be non-zero.
    pub fn weighted(&mut self, weights: &[u64]) -> usize {
        let total: u64 = weights.iter().sum();
        let mut r = self.below(total);
        for (i, w) in weights.iter().enumerate() {
            if r < *w {
                return i;
            }
            r -= w;
        }
        unreachable!()
    }
}
