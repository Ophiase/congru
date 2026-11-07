use crate::Real;

/// How much a pair counts given its residual, as an iteratively reweighted
/// least-squares weight. The scale is the residual past which a pair starts
/// to count as an outlier, in coordinate units.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Kernel<Float> {
    #[default]
    Quadratic,
    /// Quadratic below the scale, linear beyond: mild, always convex.
    Huber(Float),
    /// `1 / (1 + (r/c)²)`: heavy tails, keeps far pairs a little.
    Cauchy(Float),
    /// `(c² / (c² + r²))²`, stronger than Cauchy.
    GemanMcClure(Float),
    /// Biweight: pairs beyond the scale count zero.
    Tukey(Float),
}

impl<Float: Real> Kernel<Float> {
    #[must_use]
    #[inline]
    pub fn weight(self, residual: Float) -> Float {
        let residual = residual.magnitude();
        let one = Float::ONE;
        match self {
            Self::Quadratic => one,
            Self::Huber(scale) if residual <= scale => one,
            Self::Huber(scale) => scale / residual,
            Self::Cauchy(scale) => {
                let ratio = residual / scale;
                one / (one + ratio * ratio)
            }
            Self::GemanMcClure(scale) => {
                let square = scale * scale;
                let share = square / (square + residual * residual);
                share * share
            }
            Self::Tukey(scale) if residual >= scale => Float::ZERO,
            Self::Tukey(scale) => {
                let ratio = residual / scale;
                let share = one - ratio * ratio;
                share * share
            }
        }
    }

    pub(crate) fn scale_is_valid(self) -> bool {
        match self {
            Self::Quadratic => true,
            Self::Huber(scale)
            | Self::Cauchy(scale)
            | Self::GemanMcClure(scale)
            | Self::Tukey(scale) => scale > Float::ZERO && scale.finite(),
        }
    }
}
