use super::kabsch::kabsch;
use super::{Match, check_length};
use crate::geometry::count_as_float;
use crate::{Error, Point, RigidTransform, Vector};

/// A motion and a scale: `p ↦ scale · R p + t`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarFit {
    pub scale: f64,
    /// `R` and `t`, the scale left out.
    pub transform: RigidTransform,
    /// Root mean square distance left after the motion and the scale.
    pub rmse: f64,
}

/// The [similarity] minimising `Σ ‖c R sᵢ + t − tᵢ‖²` ([Umeyama]), each `sourceᵢ`
/// paired with `targetᵢ`: a wrong unit or a depth scale off by a few percent.
///
/// [similarity]: https://en.wikipedia.org/wiki/Similarity_(geometry)
/// [Umeyama]: https://doi.org/10.1109/34.88573
///
/// # Errors
/// [`Error::LengthMismatch`], [`Error::TooFewCorrespondences`] below three pairs,
/// [`Error::Degenerate`] for collinear points.
pub fn fit_similarity(source: &[Point], target: &[Point]) -> Result<SimilarFit, Error> {
    check_length("source and target", source.len(), target.len())?;
    let rotation = kabsch(&paired(source, target))?.rotation();
    let count = count_as_float(source.len());
    let mean = |points: &[Point]| -> Vector {
        points.iter().map(|point| point.coords).sum::<Vector>() / count
    };
    let (from, to) = (mean(source), mean(target));
    let turned = |point: &Point| rotation * (point.coords - from);
    let along: f64 = source
        .iter()
        .zip(target)
        .map(|(source, target)| turned(source).dot(&(target.coords - to)))
        .sum();
    let spread: f64 = source
        .iter()
        .map(|point| turned(point).norm_squared())
        .sum();
    if spread <= 0.0 {
        return Err(Error::Degenerate);
    }
    let scale = along / spread;
    let translation = to - scale * (rotation * from);
    let squared: f64 = source
        .iter()
        .zip(target)
        .map(|(source, target)| {
            (scale * (rotation * source.coords) + translation - target.coords).norm_squared()
        })
        .sum();
    Ok(SimilarFit {
        scale,
        transform: RigidTransform::from_parts(rotation, translation),
        rmse: (squared / count).sqrt(),
    })
}

fn paired(source: &[Point], target: &[Point]) -> Vec<Match> {
    source
        .iter()
        .zip(target)
        .map(|(source, target)| Match {
            source: *source,
            target: *target,
            normal: None,
            weight: 1.0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rand::rngs::ChaCha8Rng;
    use rand::{RngExt, SeedableRng};

    use super::*;

    fn drawn(random: &mut ChaCha8Rng, half: f64) -> Vector {
        Vector::from_fn(|_, _| half * (random.random::<f64>() * 2.0 - 1.0))
    }

    #[test]
    fn a_known_scale_and_motion_are_recovered_within_the_noise() -> Result<(), Error> {
        const NOISE: f64 = 0.001;
        const SPREAD: f64 = 0.1;
        const POINTS: usize = 400;
        let mut random = ChaCha8Rng::seed_from_u64(3);
        let source: Vec<Point> = (0..POINTS)
            .map(|_| Point::from(drawn(&mut random, SPREAD)))
            .collect();
        let motion = RigidTransform::from_axis_angle(Vector::new(0.7, -0.2, 1.1), [0.1, 0.0, -0.2]);
        for truth in [0.5, 0.98, 1.0, 1.05, 3.0] {
            let target: Vec<Point> = source
                .iter()
                .map(|point| {
                    let turned = motion.rotation() * point.coords;
                    Point::from(truth * turned + motion.translation() + drawn(&mut random, NOISE))
                })
                .collect();
            let fit = fit_similarity(&source, &target)?;
            let bound = NOISE / SPREAD;
            assert!(
                (fit.scale - truth).abs() < bound,
                "{} for {truth}",
                fit.scale
            );
            assert!(fit.transform.rotation_error_to(&motion) < bound / truth);
            assert!(fit.rmse < 2.0 * NOISE, "{}", fit.rmse);
        }
        Ok(())
    }

    #[test]
    fn collinear_points_are_refused() {
        let line: Vec<Point> = (0..5)
            .map(|step| Point::new(f64::from(step), 0.0, 0.0))
            .collect();
        assert!(fit_similarity(&line, &line).is_err());
    }

    #[test]
    fn a_length_mismatch_is_refused() {
        let points = [Point::origin(); 4];
        assert!(fit_similarity(&points[..3], &points).is_err());
    }
}
