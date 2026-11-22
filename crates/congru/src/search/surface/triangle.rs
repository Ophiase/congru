use crate::{Point, Vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Face {
    pub(super) corners: [Point; 3],
    pub(super) normal: nalgebra::Unit<Vector>,
}

impl Face {
    /// `None` for a triangle with no area, which has no normal.
    pub(super) fn new(corners: [Point; 3]) -> Option<Self> {
        let cross = (corners[1] - corners[0]).cross(&(corners[2] - corners[0]));
        nalgebra::Unit::try_new(cross, f64::MIN_POSITIVE.sqrt())
            .map(|normal| Self { corners, normal })
    }

    pub(super) fn centre(&self) -> Point {
        Point::from(
            (self.corners[0].coords + self.corners[1].coords + self.corners[2].coords) / 3.0,
        )
    }

    /// The point of the triangle nearest `query`, by which of its regions holds the
    /// projection ([Ericson], 5.1.5).
    ///
    /// [Ericson]: https://realtimecollisiondetection.net/
    pub(super) fn closest(&self, query: &Point) -> Point {
        let [first, second, third] = self.corners;
        let (along, across) = (second - first, third - first);
        let from_first = query - first;
        let (d1, d2) = (along.dot(&from_first), across.dot(&from_first));
        if d1 <= 0.0 && d2 <= 0.0 {
            return first;
        }
        let from_second = query - second;
        let (d3, d4) = (along.dot(&from_second), across.dot(&from_second));
        if d3 >= 0.0 && d4 <= d3 {
            return second;
        }
        let on_first_edge = d1 * d4 - d3 * d2;
        if on_first_edge <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
            return first + along * (d1 / (d1 - d3));
        }
        let from_third = query - third;
        let (d5, d6) = (along.dot(&from_third), across.dot(&from_third));
        if d6 >= 0.0 && d5 <= d6 {
            return third;
        }
        let on_second_edge = d5 * d2 - d1 * d6;
        if on_second_edge <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
            return first + across * (d2 / (d2 - d6));
        }
        let on_third_edge = d3 * d6 - d5 * d4;
        if on_third_edge <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
            let share = (d4 - d3) / ((d4 - d3) + (d5 - d6));
            return second + (third - second) * share;
        }
        let total = on_third_edge + on_second_edge + on_first_edge;
        first + along * (on_second_edge / total) + across * (on_first_edge / total)
    }
}
