use crate::{ types::Vec3, point_cloud::PointCloud, random_utils::* };
use rand::{ prelude::*, rng };

/// Generate a point cloud iteratively by random walk
pub fn iterative_random_walk(
    initial_points: Vec<Vec3>,
    num_iterations: usize,
    epsilon: f64,
    points_per_iteration: usize
) -> PointCloud {
    let mut rng = rng();
    let mut points = initial_points;

    for _ in 0..num_iterations {
        let mut new_points = Vec::new();

        for _ in 0..points_per_iteration {
            if !points.is_empty() {
                let base_idx = rng.random_range(0..points.len());
                let base_point = points[base_idx];

                let new_point = base_point + random_sphere_point(epsilon, &mut rng);
                new_points.push(new_point);
            }
        }

        points.extend(new_points);
    }

    PointCloud::from_points(points)
}

/// Generate a grid-based point cloud
pub fn grid(spacing: f64, dimensions: (usize, usize, usize)) -> PointCloud {
    let mut points = Vec::new();
    let (nx, ny, nz) = dimensions;

    for i in 0..nx {
        for j in 0..ny {
            for k in 0..nz {
                let point = Vec3::new(
                    (i as f64) * spacing,
                    (j as f64) * spacing,
                    (k as f64) * spacing
                );
                points.push(point);
            }
        }
    }

    PointCloud::from_points(points)
}

/// Generate random points in a box
pub fn random_box(min: Vec3, max: Vec3, num_points: usize) -> PointCloud {
    let mut rng = rng();
    let mut points = Vec::with_capacity(num_points);

    for _ in 0..num_points {
        let point = Vec3::new(
            rng.random_range(min.x..max.x),
            rng.random_range(min.y..max.y),
            rng.random_range(min.z..max.z)
        );
        points.push(point);
    }

    PointCloud::from_points(points)
}

/// Generate random points on a sphere surface
pub fn sphere_surface(center: Vec3, radius: f64, num_points: usize) -> PointCloud {
    let mut rng = rng();
    let mut points = Vec::with_capacity(num_points);

    for _ in 0..num_points {
        let point = center + random_sphere_surface_point(radius, &mut rng);
        points.push(point);
    }

    PointCloud::from_points(points)
}

/// Generate random points inside a sphere
pub fn sphere_volume(center: Vec3, radius: f64, num_points: usize) -> PointCloud {
    let mut rng = rng();
    let mut points = Vec::with_capacity(num_points);

    for _ in 0..num_points {
        let point = center + random_sphere_point(radius, &mut rng);
        points.push(point);
    }

    PointCloud::from_points(points)
}
