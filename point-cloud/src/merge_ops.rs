use crate::{ types::Vec3, point_cloud::PointCloud };

impl PointCloud {
    /// Merge with another point cloud
    pub fn merge(&mut self, other: &PointCloud) {
        self.points.extend_from_slice(&other.points);
    }

    /// Create a new point cloud by merging multiple clouds
    pub fn merge_multiple(clouds: &[PointCloud]) -> Self {
        let total_points: usize = clouds
            .iter()
            .map(|c| c.len())
            .sum();
        let mut points = Vec::with_capacity(total_points);

        for cloud in clouds {
            points.extend_from_slice(&cloud.points);
        }

        Self::from_points(points)
    }

    /// Remove duplicate points within a tolerance
    pub fn deduplicate(&mut self, tolerance: f64) {
        let mut unique_points = Vec::new();

        for point in &self.points {
            let is_duplicate = unique_points
                .iter()
                .any(|existing: &Vec3| { (point - existing).norm() < tolerance });

            if !is_duplicate {
                unique_points.push(*point);
            }
        }

        self.points = unique_points;
    }
}
