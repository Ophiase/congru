use congru::{Error, Point, PointCloud, TriangleMesh};

use crate::noise::Fractal;

/// A square of side `size` around the origin, its height the fractal
/// noise: a floor, a field, a road, never perfectly flat.
///
/// # Errors
/// Below two samples a side.
pub fn ground(size: f64, samples: u32, relief: &Fractal) -> Result<TriangleMesh, Error> {
    let step = size / f64::from(samples.max(2) - 1);
    let vertices: Vec<Point> = (0..samples * samples)
        .map(|index| {
            let across = f64::from(index % samples) * step - size / 2.0;
            let along = f64::from(index / samples) * step - size / 2.0;
            Point::new(across, along, relief.height(across, along))
        })
        .collect();
    let corner = |across: u32, along: u32| along * samples + across;
    let triangles: Vec<[u32; 3]> = (0..samples - 1)
        .flat_map(|along| {
            (0..samples - 1).flat_map(move |across| {
                [
                    [
                        corner(across, along),
                        corner(across + 1, along),
                        corner(across, along + 1),
                    ],
                    [
                        corner(across + 1, along),
                        corner(across + 1, along + 1),
                        corner(across, along + 1),
                    ],
                ]
            })
        })
        .collect();
    TriangleMesh::new(PointCloud::new(vertices)?, triangles)
}
