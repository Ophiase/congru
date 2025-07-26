use crate::types::Vec3;
use serde::{ Serialize, Deserialize };

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloud {
    pub points: Vec<Vec3>,
    pub metadata: Option<PointCloudMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloudMetadata {
    pub name: String,
    pub timestamp: Option<String>,
    pub source: Option<String>,
}
