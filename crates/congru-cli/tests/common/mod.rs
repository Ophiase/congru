use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use congru::{PointCloud, RigidTransform, TriangleMesh};
use serde_json::Value;

/// A folder of its own under the system's temporary folder, removed when
/// dropped.
pub struct Scratch(pub PathBuf);

impl Scratch {
    pub fn new(test: &str) -> Self {
        let folder = std::env::temp_dir().join(format!("congru-cli-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        Self(folder)
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn write_xyz(path: &Path, cloud: &PointCloud) {
    let mut text = String::new();
    for point in cloud.points() {
        writeln!(text, "{} {} {}", point.x, point.y, point.z).unwrap();
    }
    std::fs::write(path, text).unwrap();
}

pub fn write_ply(path: &Path, mesh: &TriangleMesh) {
    let mut text = format!(
        "ply\nformat ascii 1.0\nelement vertex {}\nproperty double x\n\
         property double y\nproperty double z\nelement face {}\n\
         property list uchar int vertex_indices\nend_header\n",
        mesh.vertices().count(),
        mesh.triangles().len()
    );
    for point in mesh.vertices().points() {
        writeln!(text, "{} {} {}", point.x, point.y, point.z).unwrap();
    }
    for [first, second, third] in mesh.triangles() {
        writeln!(text, "3 {first} {second} {third}").unwrap();
    }
    std::fs::write(path, text).unwrap();
}

/// Runs the command; its exit status, and every line of its output parsed.
pub fn congru(arguments: &[&str]) -> (bool, Vec<Value>) {
    let output = Command::new(env!("CARGO_BIN_EXE_congru"))
        .args(arguments)
        .output()
        .unwrap();
    let lines = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (output.status.success(), lines)
}

pub fn transform(value: &Value) -> RigidTransform {
    let entries: Vec<f64> = value
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry.as_f64().unwrap())
        .collect();
    RigidTransform::from_row_major(&entries.try_into().unwrap()).unwrap()
}

pub fn row_major(transform: &RigidTransform) -> String {
    transform
        .to_row_major()
        .iter()
        .map(f64::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}
