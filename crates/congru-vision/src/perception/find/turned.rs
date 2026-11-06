use std::f64::consts::TAU;

use congru::{RigidTransform, Vector};
use nalgebra::{Quaternion, UnitQuaternion};

use super::pairs::Oriented;
use super::search::{FIRST_ITERATIONS, shared};
use super::{Finder, Found};
use crate::{Camera, DepthImage, Observed};

/// Orientations tried at a known place: rays bring back from about 30°, so a few
/// hundred cover them all.
const ORIENTATIONS: u32 = 144;
/// The best orientations refined before they are compared.
const TURNED_KEPT: usize = 12;
/// The fractions of the plastic number's powers, a low-discrepancy sequence.
const SPREAD_FIRST: f64 = 0.754_877_666_2;
const SPREAD_SECOND: f64 = 0.569_840_290_9;
/// The first moves of a polish, in radians and as a share of the model's size,
/// and how they shrink when none helps.
const POLISH_TURN: f64 = 0.02;
const POLISH_SHIFT_SHARE: f64 = 1.0 / 60.0;
const POLISH_SHRINK: f64 = 0.5;
const POLISH_ROUNDS: u32 = 6;

impl Finder {
    /// Orientations spread over all of them, the model's centre at `place`.
    fn orientations_at(&self, place: Vector) -> Vec<RigidTransform> {
        (0..ORIENTATIONS)
            .map(|index| {
                let turn = spread_turn(index);
                let shift = place - turn * self.points_centre.coords;
                RigidTransform::from_parts(turn, [shift.x, shift.y, shift.z])
            })
            .collect()
    }

    /// Orientations at the middle of `frame`'s points: a frame cut to a region of
    /// interest says where the object is, not how it is turned, and a small or
    /// half-hidden object gives votes too few pairs.
    pub(super) fn orientations_in(&self, frame: &[Oriented]) -> Vec<RigidTransform> {
        let middle = |pick: fn(&Oriented) -> f64| {
            let mut values: Vec<f64> = frame.iter().map(pick).collect();
            values.sort_by(f64::total_cmp);
            values.get(values.len() / 2).copied().unwrap_or(0.0)
        };
        self.orientations_at(Vector::new(
            middle(|oriented| oriented.point.x),
            middle(|oriented| oriented.point.y),
            middle(|oriented| oriented.point.z),
        ))
    }

    /// The object found again where `forecast` puts it, turned any way: a knock
    /// loses how it is turned, not where it stands. Needs no votes, so it holds
    /// when the object is half hidden and its pairs are few. The last choice reads
    /// what the answer will be believed by, not an amount a larger view would win.
    #[must_use]
    pub fn turned_again(
        &self,
        depth: &DepthImage,
        camera: &Camera,
        forecast: &RigidTransform,
    ) -> Option<Found> {
        let tight = self.observed_at(depth, camera, forecast);
        let tolerance = tight.tolerance();
        let place = forecast.apply(&self.points_centre).coords;
        let mut judged = shared(&self.orientations_at(place), |pose| {
            let brought = self.refined_by(depth, camera, pose, FIRST_ITERATIONS);
            vec![
                (self.score(&tight, camera, pose, tolerance), *pose),
                (self.score(&tight, camera, &brought, tolerance), brought),
            ]
        });
        judged.sort_by(|left, right| right.0.total_cmp(&left.0));
        let best: Vec<(f64, RigidTransform)> = judged.iter().take(TURNED_KEPT).copied().collect();
        let refined = shared(&best, |(value, pose)| {
            let refined = self.polished(&tight, camera, &self.refined(depth, camera, pose));
            vec![
                (*value, *pose),
                (self.score(&tight, camera, &refined, tolerance), refined),
            ]
        });
        let (_, pose) = refined
            .into_iter()
            .map(|(_, pose)| (self.confirmed(&tight, camera, &pose), pose))
            .max_by(|left, right| left.0.total_cmp(&right.0))?;
        Some(Found {
            pose,
            agreement: self.confirmed(&tight, camera, &pose),
        })
    }

    /// `start` improved by small moves the rays support, the moves shrinking
    /// when none helps: rays through the model pin what hits leave free.
    #[must_use]
    pub fn polished(
        &self,
        seen: &Observed,
        camera: &Camera,
        start: &RigidTransform,
    ) -> RigidTransform {
        let tolerance = seen.tolerance();
        let (mut pose, mut best) = (*start, self.score(seen, camera, start, tolerance));
        let (mut turn, mut shift) = (POLISH_TURN, self.diameter * POLISH_SHIFT_SHARE);
        for _ in 0..POLISH_ROUNDS {
            let mut better = false;
            for axis in [Vector::x(), Vector::y(), Vector::z()] {
                for sign in [1.0, -1.0] {
                    let moved = axis * sign;
                    let centre = pose.apply(&self.points_centre);
                    let spin = RigidTransform::from_axis_angle(Vector::zeros(), -centre.coords)
                        .then(&RigidTransform::from_axis_angle(moved * turn, [0.0; 3]))
                        .then(&RigidTransform::from_axis_angle(
                            Vector::zeros(),
                            centre.coords,
                        ));
                    let slide = moved * shift;
                    let slid = RigidTransform::from_axis_angle(
                        Vector::zeros(),
                        [slide.x, slide.y, slide.z],
                    );
                    for trial in [pose.then(&spin), pose.then(&slid)] {
                        let value = self.score(seen, camera, &trial, tolerance);
                        if value > best {
                            (pose, best, better) = (trial, value, true);
                        }
                    }
                }
            }
            if !better {
                turn *= POLISH_SHRINK;
                shift *= POLISH_SHRINK;
            }
        }
        pose
    }
}

/// A turn among [`ORIENTATIONS`] spread evenly: [Shoemake's] uniform draw, fed a
/// low-discrepancy sequence instead of chance.
///
/// [Shoemake's]: https://doi.org/10.1016/B978-0-08-050755-2.50036-1
fn spread_turn(index: u32) -> UnitQuaternion<f64> {
    let share = f64::from(index);
    let first = (share + 0.5) / f64::from(ORIENTATIONS);
    let second = (share * SPREAD_FIRST).fract() * TAU;
    let third = (share * SPREAD_SECOND).fract() * TAU;
    let (low, high) = ((1.0 - first).sqrt(), first.sqrt());
    UnitQuaternion::from_quaternion(Quaternion::new(
        high * third.cos(),
        low * second.sin(),
        low * second.cos(),
        high * third.sin(),
    ))
}
