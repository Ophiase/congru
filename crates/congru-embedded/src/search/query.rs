use super::{BUCKET, KdTree};
use crate::{Real, Vector3};

/// Deep enough for 2³² points: each level pushes at most one far child.
const STACK_DEPTH: usize = 32;

/// A point of the tree, by its index in [`KdTree::points`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Nearest<Float> {
    pub index: usize,
    pub squared_distance: Float,
}

/// Decides which points a walk through the tree keeps.
pub(super) trait Collector<Float> {
    /// The squared distance past which a point no longer interests it.
    fn reach(&self) -> Float;
    fn offer(&mut self, index: usize, squared_distance: Float);
}

struct Closest<Float> {
    found: Option<usize>,
    reach: Float,
}

impl<Float: Real> Collector<Float> for Closest<Float> {
    #[inline]
    fn reach(&self) -> Float {
        self.reach
    }

    #[inline]
    fn offer(&mut self, index: usize, squared_distance: Float) {
        if squared_distance < self.reach {
            self.reach = squared_distance;
            self.found = Some(index);
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Frame<Float> {
    start: u32,
    end: u32,
    node: u32,
    plane: Float,
}

impl<Float: Real> KdTree<Float> {
    /// The stack a query reserves for the branches left to visit.
    #[must_use]
    pub const fn query_stack_bytes() -> usize {
        size_of::<[Frame<Float>; STACK_DEPTH]>()
    }

    /// The closest point at a squared distance strictly below `squared_limit`.
    /// A tight limit also makes the search faster, by pruning from the start.
    #[must_use]
    pub fn nearest_within(
        &self,
        query: Vector3<Float>,
        squared_limit: Float,
    ) -> Option<Nearest<Float>> {
        let mut closest = Closest {
            found: None,
            reach: squared_limit,
        };
        self.walk(query, &mut closest);
        closest.found.map(|index| Nearest {
            index,
            squared_distance: closest.reach,
        })
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "the tree refuses 2^32 points at its making"
    )]
    pub(super) fn walk(&self, query: Vector3<Float>, collector: &mut impl Collector<Float>) {
        let mut stack = [Frame::<Float>::default(); STACK_DEPTH];
        stack[0].end = self.points.len() as u32;
        let mut depth = 1;
        while depth > 0 {
            depth -= 1;
            let Frame {
                mut start,
                mut end,
                mut node,
                plane,
            } = stack[depth];
            if plane >= collector.reach() {
                continue;
            }
            while (end - start) as usize > BUCKET {
                let middle = start + (end - start) / 2;
                let pivot = self.points[middle as usize];
                collector.offer(middle as usize, (pivot - query).squared_norm());
                let axis = usize::from(self.axes[node as usize]);
                let along = query[axis] - pivot[axis];
                let (left, right) = (2 * node + 1, 2 * node + 2);
                let lower = along <= Float::ZERO;
                stack[depth] = Frame {
                    start: if lower { middle + 1 } else { start },
                    end: if lower { end } else { middle },
                    node: if lower { right } else { left },
                    plane: along * along,
                };
                depth += 1;
                (start, end, node) = if lower {
                    (start, middle, left)
                } else {
                    (middle + 1, end, right)
                };
            }
            for index in start as usize..end as usize {
                collector.offer(index, (self.points[index] - query).squared_norm());
            }
        }
    }
}
