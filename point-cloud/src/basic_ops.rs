use crate::{ types::{ Vec3 }, point_cloud::PointCloud };

impl PointCloud {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            metadata: None,
        }
    }

    /// Create a point cloud from a vector of points
    pub fn from_points(points: Vec<Vec3>) -> Self {
        Self {
            points,
            metadata: None,
        }
    }

    /// Add a single point to the cloud
    pub fn add_point(&mut self, point: Vec3) {
        self.points.push(point);
    }

    /// Add multiple points to the cloud
    pub fn add_points(&mut self, points: Vec<Vec3>) {
        self.points.extend(points);
    }

    /// Get the number of points
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if the point cloud is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl Default for PointCloud {
    fn default() -> Self {
        Self::new()
    }
}
