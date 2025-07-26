use rand::prelude::*;

pub struct PointCloudSampler {
    rng: StdRng,
}

impl PointCloudSampler {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    /// Creates a new `PointCloudSampler` with an independent seed derived from the current sampler.
    /// Useful for generating multiple independent samplers for parallel or distinct sampling tasks.
    /// Each forked sampler has its own state, ensuring reproducible and independent sampling.
    ///
    /// # Example
    /// let mut original = PointCloudSampler::new();
    /// let mut fork1 = original.fork();
    /// let mut fork2 = original.fork();
    /// // `original.rng()`, `fork1.rng()`, and `fork2.rng()` are independent,
    /// // but all are deterministically derived from the original seed.
    pub fn fork(&mut self) -> Self {
        // build a full 256‑bit seed for the new StdRng
        let mut seed = <StdRng as SeedableRng>::Seed::default();
        // fill it from our current RNG, advancing its state
        self.rng.fill_bytes(&mut seed);
        // create a new sampler from that seed
        PointCloudSampler {
            rng: StdRng::from_seed(seed),
        }
    }
}

impl Default for PointCloudSampler {
    fn default() -> Self {
        Self::new()
    }
}
