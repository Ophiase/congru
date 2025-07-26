use crate::{types::Vec3, point_cloud::PointCloud};

impl PointCloud {
    /// Calculate the centroid of the point cloud
    pub fn centroid(&self) -> Vec3 {
        if self.points.is_empty() {
            return Vec3::zeros();
        }
        
        let sum = self.points.iter().fold(Vec3::zeros(), |acc, p| acc + p);
        sum / self.points.len() as f64
    }

    /// Calculate the bounding box (min, max)
    pub fn bounding_box(&self) -> Option<(Vec3, Vec3)> {
        if self.points.is_empty() {
            return None;
        }

        let first = self.points[0];
        let mut min = first;
        let mut max = first;

        for point in &self.points[1..] {
            min = Vec3::new(
                min.x.min(point.x),
                min.y.min(point.y),
                min.z.min(point.z),
            );
            max = Vec3::new(
                max.x.max(point.x),
                max.y.max(point.y),
                max.z.max(point.z),
            );
        }

        Some((min, max))
    }

    /// Calculate minimum pairwise distance
    pub fn min_pairwise_distance(&self) -> Option<f64> {
        if self.points.len() < 2 {
            return None;
        }

        let mut min_dist = f64::INFINITY;
        for i in 0..self.points.len() {
            for j in (i + 1)..self.points.len() {
                let dist = (self.points[i] - self.points[j]).norm();
                min_dist = min_dist.min(dist);
            }
        }

        Some(min_dist)
    }
}
