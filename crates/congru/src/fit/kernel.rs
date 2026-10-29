use crate::Distance;

/// How much a correspondence counts given its residual `r`, as an
/// iteratively reweighted least-squares weight. The scale `c` is the
/// residual past which a pair starts to count as an outlier.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Kernel {
    /// Plain least squares: every pair counts fully. Fast, not robust.
    #[default]
    Quadratic,
    /// Quadratic below `c`, linear beyond: mild robustness, always convex.
    Huber(Distance),
    /// `1 / (1 + (r/c)²)`: heavy tails, keeps far pairs a little.
    Cauchy(Distance),
    /// Geman–McClure: `(c² / (c² + r²))²`, stronger than Cauchy.
    GemanMcClure(Distance),
    /// Tukey's biweight: pairs beyond `c` count zero.
    Tukey(Distance),
}

impl Kernel {
    #[must_use]
    pub fn weight(self, residual: f64) -> f64 {
        let residual = residual.abs();
        match self {
            Self::Quadratic => 1.0,
            Self::Huber(scale) => {
                if residual <= scale.get() {
                    1.0
                } else {
                    scale.get() / residual
                }
            }
            Self::Cauchy(scale) => 1.0 / (1.0 + (residual / scale.get()).powi(2)),
            Self::GemanMcClure(scale) => {
                let squared_scale = scale.get().powi(2);
                (squared_scale / (squared_scale + residual.powi(2))).powi(2)
            }
            Self::Tukey(scale) => {
                if residual >= scale.get() {
                    0.0
                } else {
                    (1.0 - (residual / scale.get()).powi(2)).powi(2)
                }
            }
        }
    }
}
