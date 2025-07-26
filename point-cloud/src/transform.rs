use crate::{ types::{ Vec3, Mat3 }, point_cloud::PointCloud };

impl PointCloud {
    /// Apply a rigid transformation (rotation + translation)
    pub fn transform(&mut self, rotation: &Mat3, translation: &Vec3) {
        for point in &mut self.points {
            *point = rotation * *point + translation;
        }
    }

    /// Create a transformed copy
    pub fn transformed(&self, rotation: &Mat3, translation: &Vec3) -> Self {
        let transformed_points: Vec<Vec3> = self.points
            .iter()
            .map(|p| rotation * p + translation)
            .collect();

        Self {
            points: transformed_points,
            metadata: self.metadata.clone(),
        }
    }
}
