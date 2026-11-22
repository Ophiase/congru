use std::f64::consts::PI;
use std::num::NonZeroUsize;

use nalgebra::{SVector, Unit};

use crate::geometry::count_as_float;
use crate::search::map;
use crate::{NearestNeighbors, Point, PointCloud, Vector};

const BINS: usize = 11;
/// Three angles, eleven bins each.
pub(crate) const LENGTH: usize = 3 * BINS;

pub(crate) type Descriptor = SVector<f64, LENGTH>;

/// Fast point feature histograms (Rusu et al., 2009): for each point, the
/// angles between its normal, a neighbour's normal and the segment joining
/// them, binned, then averaged with the neighbours' own histograms. Two
/// points with the same neighbourhood shape get close descriptors, whatever
/// the pose.
pub(crate) fn describe(
    cloud: &PointCloud,
    normals: &[Unit<Vector>],
    neighbors: NonZeroUsize,
) -> Vec<Descriptor> {
    let Ok(index) = NearestNeighbors::new(cloud) else {
        return Vec::new();
    };
    let around: Vec<Vec<(usize, f64)>> = map(cloud.points(), |point| {
        index
            .k_nearest(point, neighbors)
            .into_iter()
            .skip(1)
            .map(|neighbor| (neighbor.index, neighbor.squared_distance.sqrt()))
            .collect()
    });
    let simple: Vec<Descriptor> = (0..cloud.count())
        .map(|centre| simple_histogram(cloud.points(), normals, centre, &around[centre]))
        .collect();
    (0..cloud.count())
        .map(|centre| {
            let spread = around[centre]
                .iter()
                .filter(|(_, distance)| *distance > 0.0)
                .fold(Descriptor::zeros(), |sum, (neighbor, distance)| {
                    sum + simple[*neighbor] / *distance
                });
            let count = count_as_float(around[centre].len().max(1));
            normalised(&(simple[centre] + spread / count))
        })
        .collect()
}

fn simple_histogram(
    points: &[Point],
    normals: &[Unit<Vector>],
    centre: usize,
    around: &[(usize, f64)],
) -> Descriptor {
    let mut histogram = Descriptor::zeros();
    for (neighbor, distance) in around.iter().filter(|(_, distance)| *distance > 0.0) {
        let along = (points[*neighbor] - points[centre]) / *distance;
        let first = normals[centre].into_inner();
        let Some(second_axis) = Unit::try_new(first.cross(&along), f64::EPSILON) else {
            continue;
        };
        let third_axis = first.cross(&second_axis);
        let other = normals[*neighbor].into_inner();
        let alpha = second_axis.dot(&other);
        let phi = first.dot(&along);
        let theta = third_axis.dot(&other).atan2(first.dot(&other));
        histogram[bin(alpha, -1.0, 1.0)] += 1.0;
        histogram[BINS + bin(phi, -1.0, 1.0)] += 1.0;
        histogram[2 * BINS + bin(theta, -PI, PI)] += 1.0;
    }
    histogram
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clamped to the bins before the cast"
)]
fn bin(value: f64, lowest: f64, highest: f64) -> usize {
    let share = ((value - lowest) / (highest - lowest)).clamp(0.0, 1.0);
    ((share * count_as_float(BINS)).floor() as usize).min(BINS - 1)
}

fn normalised(descriptor: &Descriptor) -> Descriptor {
    let total = descriptor.sum();
    if total > 0.0 {
        descriptor * (100.0 / total)
    } else {
        *descriptor
    }
}
