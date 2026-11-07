use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::Ordering;

use super::BUCKET;
use crate::{Error, Real};

/// The order of the points in the tree and the split axis of each inner node,
/// numbered as in a binary heap.
pub(super) fn arrange<Float: Real>(points: &[[Float; 3]]) -> Result<(Vec<u32>, Box<[u8]>), Error> {
    if points.is_empty() {
        return Err(Error::Empty);
    }
    if u32::try_from(points.len()).is_err() {
        return Err(Error::TooManyPoints {
            found: points.len(),
        });
    }
    if points
        .iter()
        .flatten()
        .any(|coordinate| !coordinate.finite())
    {
        return Err(Error::NotFinite);
    }
    let mut order: Vec<u32> = (0..).zip(points).map(|(index, _)| index).collect();
    let mut axes = alloc::vec![0; inner_nodes(points.len())].into_boxed_slice();
    split(&mut order, points, &mut axes, 0);
    Ok((order, axes))
}

/// A child holds at most half of its parent's points, so a level past
/// `BUCKET` points is the only one needing a slot.
fn inner_nodes(count: usize) -> usize {
    let mut levels = 0;
    while count >> levels > BUCKET {
        levels += 1;
    }
    (1 << levels) - 1
}

fn split<Float: Real>(order: &mut [u32], points: &[[Float; 3]], planes: &mut [u8], node: usize) {
    if order.len() <= BUCKET {
        return;
    }
    let axis = widest_axis(order, points);
    planes[node] = u8::from(axis > 0) + u8::from(axis > 1);
    let middle = order.len() / 2;
    order.select_nth_unstable_by(middle, |left, right| {
        let (left, right) = (points[*left as usize][axis], points[*right as usize][axis]);
        left.partial_cmp(&right).unwrap_or(Ordering::Equal)
    });
    let (lower, rest) = order.split_at_mut(middle);
    split(lower, points, planes, 2 * node + 1);
    split(&mut rest[1..], points, planes, 2 * node + 2);
}

fn widest_axis<Float: Real>(order: &[u32], points: &[[Float; 3]]) -> usize {
    let mut lowest = [Float::INFINITY; 3];
    let mut highest = [-Float::INFINITY; 3];
    for index in order {
        for (axis, coordinate) in points[*index as usize].into_iter().enumerate() {
            if coordinate < lowest[axis] {
                lowest[axis] = coordinate;
            }
            if coordinate > highest[axis] {
                highest[axis] = coordinate;
            }
        }
    }
    let mut widest = 0;
    for axis in 1..3 {
        if highest[axis] - lowest[axis] > highest[widest] - lowest[widest] {
            widest = axis;
        }
    }
    widest
}
