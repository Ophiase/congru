use congru::{NearestNeighbors, PointCloud, RigidTransform};

/// The mean distance from the model at `pose` to the model at `truth`, nearest
/// point to nearest point: [ADD-S], blind to what a symmetry leaves unchanged, as
/// a gear turned by one tooth.
///
/// [ADD-S]: https://arxiv.org/abs/1711.00199
pub(super) fn surface_error(
    samples: &PointCloud,
    pose: &RigidTransform,
    truth: &RigidTransform,
) -> anyhow::Result<f64> {
    let index = NearestNeighbors::new(&samples.transformed(truth))?;
    let total: f64 = samples
        .points()
        .iter()
        .map(|point| index.nearest(&pose.apply(point)).squared_distance.sqrt())
        .sum();
    Ok(total / congru_vision::pixel::coordinate(samples.count()))
}

#[cfg(test)]
mod tests {
    use congru::{Distance, TriangleMesh};
    use congru_sim::{bracket, gear};

    use super::*;

    fn off_by(mesh: &TriangleMesh, angle: f64) -> anyhow::Result<f64> {
        let samples = mesh.sample_surface(Distance::new(0.003)?)?.cloud().clone();
        let turned = RigidTransform::from_axis_angle(congru::Vector::z() * angle, [0.0; 3]);
        surface_error(&samples, &turned, &RigidTransform::identity())
    }

    #[test]
    fn a_turn_by_one_tooth_costs_little_where_a_part_turned_costs_much() -> anyhow::Result<()> {
        let tooth = std::f64::consts::TAU / 12.0;
        assert!(off_by(&gear()?, tooth)? < 0.002);
        assert!(off_by(&bracket()?, tooth)? > 0.005);
        Ok(())
    }
}
