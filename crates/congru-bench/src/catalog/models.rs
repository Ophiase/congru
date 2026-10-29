use congru::{Distance, Error, OrientedCloud, TriangleMesh};
use congru_synth::shapes;

/// Target samples per model size, for the model the methods align onto.
const TARGET_SAMPLES_PER_SPAN: f64 = 60.0;
/// At most this many observed points, as a sensor's frame after a crop.
const OBSERVED_POINTS: usize = 1500;
/// Observed samples per model size, before any corruption.
const OBSERVED_SAMPLES_PER_SPAN: f64 = 40.0;

/// A model of the benchmark: its surface, densely sampled for the methods
/// to align onto, more coarsely for the sensor to observe, and its size.
#[derive(Debug, Clone)]
pub struct Model {
    pub name: &'static str,
    pub summary: &'static str,
    pub target: OrientedCloud,
    pub observed: OrientedCloud,
    /// Diagonal of the bounding box: every distance of the benchmark is a
    /// share of it.
    pub span: f64,
}

impl Model {
    pub fn sampled(
        name: &'static str,
        summary: &'static str,
        mesh: &TriangleMesh,
    ) -> Result<Self, Error> {
        let span = diagonal(mesh);
        Ok(Self {
            name,
            summary,
            target: mesh.sample_surface(Distance::new(span / TARGET_SAMPLES_PER_SPAN)?)?,
            observed: at_most(
                &mesh.sample_surface(Distance::new(span / OBSERVED_SAMPLES_PER_SPAN)?)?,
                OBSERVED_POINTS,
            )?,
            span,
        })
    }
}

/// The four geometries the benchmark covers.
///
/// # Errors
/// Never in practice; the type is the mesh constructors'.
pub fn models() -> Result<Vec<Model>, Error> {
    Ok(vec![
        Model::sampled(
            "bracket",
            "machined part: flat faces, thin plates, a boss and a pin",
            &congru_sim::bracket()?,
        )?,
        Model::sampled(
            "blob",
            "smooth closed surface, nearly round: few features to lock on",
            &shapes::lumpy_sphere_mesh(48)?,
        )?,
        Model::sampled(
            "patch",
            "open curved surface, seen from one side, as a depth camera sees",
            &shapes::wavy_patch_mesh(60)?,
        )?,
        Model::sampled(
            "gear",
            "twelve teeth: the same after a twelfth of a turn, a trap for every method",
            &congru_sim::gear()?,
        )?,
    ])
}

fn diagonal(mesh: &TriangleMesh) -> f64 {
    let points = mesh.vertices().points();
    let (lowest, highest) = points.iter().fold(
        (points[0].coords, points[0].coords),
        |(lowest, highest), point| (lowest.inf(&point.coords), highest.sup(&point.coords)),
    );
    (highest - lowest).norm()
}

fn at_most(cloud: &OrientedCloud, count: usize) -> Result<OrientedCloud, Error> {
    let stride = cloud.cloud().count().div_ceil(count).max(1);
    let points: Vec<_> = cloud
        .cloud()
        .points()
        .iter()
        .step_by(stride)
        .copied()
        .collect();
    let normals: Vec<_> = cloud
        .normals()
        .iter()
        .step_by(stride)
        .map(|normal| normal.into_inner())
        .collect();
    OrientedCloud::new(congru::PointCloud::new(points)?, &normals)
}
