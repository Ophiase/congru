use std::f64::consts::TAU;

use congru::{Error, Point, PointCloud, TriangleMesh};

/// Closed solids gathered into one mesh. They may overlap: a depth buffer
/// only keeps the nearest surface, so no boolean union is needed.
#[derive(Debug, Default)]
pub struct Solids {
    vertices: Vec<Point>,
    triangles: Vec<[u32; 3]>,
}

impl Solids {
    pub fn cuboid(&mut self, lowest: [f64; 3], highest: [f64; 3]) -> &mut Self {
        let corner = |index: u32| {
            let pick = |axis: usize| {
                if index >> axis & 1 == 1 {
                    highest[axis]
                } else {
                    lowest[axis]
                }
            };
            Point::new(pick(0), pick(1), pick(2))
        };
        let quads = [
            [0, 2, 3, 1],
            [4, 5, 7, 6],
            [0, 1, 5, 4],
            [2, 6, 7, 3],
            [0, 4, 6, 2],
            [1, 3, 7, 5],
        ];
        let start = self.push_vertices((0..8).map(corner));
        for [first, second, third, fourth] in quads {
            self.triangles
                .push([start + first, start + second, start + third]);
            self.triangles
                .push([start + first, start + third, start + fourth]);
        }
        self
    }

    /// A cylinder along z from `base` up `height`, its round side cut into
    /// `segments` flat faces.
    pub fn cylinder(
        &mut self,
        base: [f64; 3],
        radius: f64,
        height: f64,
        segments: u32,
    ) -> &mut Self {
        let ring = |level: f64| {
            (0..segments).map(move |segment| {
                let angle = TAU * f64::from(segment) / f64::from(segments);
                Point::new(
                    base[0] + radius * angle.cos(),
                    base[1] + radius * angle.sin(),
                    base[2] + level,
                )
            })
        };
        let bottom = self.push_vertices(ring(0.0));
        let top = self.push_vertices(ring(height));
        let centres = self.push_vertices([
            Point::from(base),
            Point::new(base[0], base[1], base[2] + height),
        ]);
        for segment in 0..segments {
            let next = (segment + 1) % segments;
            self.triangles
                .push([bottom + segment, bottom + next, top + next]);
            self.triangles
                .push([bottom + segment, top + next, top + segment]);
            self.triangles
                .push([centres, bottom + next, bottom + segment]);
            self.triangles
                .push([centres + 1, top + segment, top + next]);
        }
        self
    }

    /// A triangular prism: the triangle `profile` in the xz plane, swept
    /// along y from `from` to `to`.
    pub fn prism(&mut self, profile: [[f64; 2]; 3], from: f64, to: f64) -> &mut Self {
        let corners = [from, to]
            .into_iter()
            .flat_map(|along| profile.map(|[across, up]| Point::new(across, along, up)));
        let start = self.push_vertices(corners);
        let faces = [
            [0, 1, 2],
            [3, 5, 4],
            [0, 3, 4],
            [0, 4, 1],
            [1, 4, 5],
            [1, 5, 2],
            [2, 5, 3],
            [2, 3, 0],
        ];
        self.triangles
            .extend(faces.map(|face| face.map(|corner| start + corner)));
        self
    }

    /// A prism: the closed outline `profile` in the xy plane, star-shaped
    /// about the origin, swept along z from `from` to `to`.
    pub fn extrusion(&mut self, profile: &[[f64; 2]], from: f64, to: f64) -> &mut Self {
        let count = u32::try_from(profile.len()).unwrap_or(0);
        let bottom = self.push_vertices(
            profile
                .iter()
                .map(|[across, along]| Point::new(*across, *along, from)),
        );
        let top = self.push_vertices(
            profile
                .iter()
                .map(|[across, along]| Point::new(*across, *along, to)),
        );
        let centres = self.push_vertices([Point::new(0.0, 0.0, from), Point::new(0.0, 0.0, to)]);
        for corner in 0..count {
            let next = (corner + 1) % count;
            self.triangles
                .push([bottom + corner, bottom + next, top + next]);
            self.triangles
                .push([bottom + corner, top + next, top + corner]);
            self.triangles
                .push([centres, bottom + next, bottom + corner]);
            self.triangles.push([centres + 1, top + corner, top + next]);
        }
        self
    }

    /// # Errors
    /// [`Error::Empty`] when no solid was added.
    pub fn build(&self) -> Result<TriangleMesh, Error> {
        TriangleMesh::new(
            PointCloud::new(self.vertices.clone())?,
            self.triangles.clone(),
        )
    }

    fn push_vertices(&mut self, vertices: impl IntoIterator<Item = Point>) -> u32 {
        let start = u32::try_from(self.vertices.len()).unwrap_or(u32::MAX);
        self.vertices.extend(vertices);
        start
    }
}
