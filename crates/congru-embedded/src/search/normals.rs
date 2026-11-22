use alloc::vec::Vec;

use super::KdTree;
use super::query::Collector;
use crate::linear::symmetric_eigen;
use crate::{Error, Real, Vector3};

/// The `COUNT` closest points so far, nearest first, on the stack.
struct Patch<Float, const COUNT: usize> {
    found: [(usize, Float); COUNT],
    held: usize,
}

impl<Float: Real, const COUNT: usize> Collector<Float> for Patch<Float, COUNT> {
    fn reach(&self) -> Float {
        if self.held < COUNT {
            Float::INFINITY
        } else {
            self.found[COUNT - 1].1
        }
    }

    fn offer(&mut self, index: usize, squared_distance: Float) {
        if squared_distance >= self.reach() {
            return;
        }
        let mut slot = self.held.min(COUNT - 1);
        while slot > 0 && self.found[slot - 1].1 > squared_distance {
            self.found[slot] = self.found[slot - 1];
            slot -= 1;
        }
        self.found[slot] = (index, squared_distance);
        self.held = (self.held + 1).min(COUNT);
    }
}

impl<Float: Real> KdTree<Float> {
    /// Replaces the normals by those of the plane fitted to each point's
    /// `COUNT` nearest points, itself included: the eigenvector of their
    /// covariance with the smallest eigenvalue. Their sign is arbitrary,
    /// which point-to-plane does not mind.
    ///
    /// # Errors
    /// [`Error::InvalidParameter`] unless `3 <= COUNT <= points`.
    pub fn estimate_normals<const COUNT: usize>(&mut self) -> Result<(), Error> {
        if COUNT < 3 || COUNT > self.points.len() {
            return Err(Error::InvalidParameter {
                name: "normal neighbours",
            });
        }
        let normals: Vec<Vector3<Float>> = self
            .points
            .iter()
            .map(|point| {
                let mut patch = Patch {
                    found: [(0, Float::ZERO); COUNT],
                    held: 0,
                };
                self.walk(*point, &mut patch);
                plane_normal(&self.points, *point, &patch.found)
            })
            .collect();
        self.normals = Some(normals.into_boxed_slice());
        Ok(())
    }
}

fn plane_normal<Float: Real>(
    points: &[Vector3<Float>],
    centre: Vector3<Float>,
    patch: &[(usize, Float)],
) -> Vector3<Float> {
    let mut mean = Vector3::ZERO;
    for (index, _) in patch {
        mean += points[*index] - centre;
    }
    let mean = mean * (Float::ONE / Float::of_count(patch.len()));
    let mut covariance = [[Float::ZERO; 3]; 3];
    for (index, _) in patch {
        let offset = points[*index] - centre - mean;
        for (sum, row) in covariance.iter_mut().zip(offset.outer(offset)) {
            for (entry, value) in sum.iter_mut().zip(row) {
                *entry += value;
            }
        }
    }
    let eigen = symmetric_eigen(covariance);
    let mut smallest = 0;
    for index in 1..3 {
        if eigen.values[index] < eigen.values[smallest] {
            smallest = index;
        }
    }
    let normal = Vector3(eigen.column(smallest));
    normal * (Float::ONE / normal.norm())
}
