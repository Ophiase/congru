use nalgebra::Unit;

use super::triangle::Face;
use super::{Closest, Surface};
use crate::{Error, Point, TriangleMesh, Vector};

const MOST_IN_A_LEAF: usize = 4;

/// A box around some faces, and either the faces or the two nodes under it.
#[derive(Debug)]
struct Node {
    low: Vector,
    high: Vector,
    /// The first face of a leaf, or the second child of an inner node, whose first
    /// child comes right after it.
    start: u32,
    faces: u32,
}

impl Node {
    fn squared_distance(&self, query: &Point) -> f64 {
        let below = self.low - query.coords;
        let above = query.coords - self.high;
        below.sup(&above).sup(&Vector::zeros()).norm_squared()
    }
}

/// The exact distance to a mesh, through a [bounding-volume hierarchy]. Its faces
/// are the mesh's, without those of no area.
///
/// [bounding-volume hierarchy]: https://en.wikipedia.org/wiki/Bounding_volume_hierarchy
#[derive(Debug)]
pub struct MeshIndex {
    faces: Vec<Face>,
    nodes: Vec<Node>,
}

impl MeshIndex {
    /// # Errors
    /// [`Error::Empty`] when no triangle has an area.
    pub fn new(mesh: &TriangleMesh) -> Result<Self, Error> {
        let points = mesh.vertices().points();
        let mut faces: Vec<Face> = mesh
            .triangles()
            .iter()
            .filter_map(|corners| Face::new(corners.map(|corner| points[corner as usize])))
            .collect();
        if faces.is_empty() {
            return Err(Error::Empty { what: "mesh" });
        }
        let mut nodes = Vec::with_capacity(faces.len());
        build(&mut faces, 0, &mut nodes);
        Ok(Self { faces, nodes })
    }

    fn visit(&self, query: &Point, node: usize, best: &mut (f64, Option<(Point, usize)>)) {
        let here = &self.nodes[node];
        if here.faces > 0 {
            let first = here.start as usize;
            for at in first..first + here.faces as usize {
                let point = self.faces[at].closest(query);
                let squared = (point - query).norm_squared();
                if squared < best.0 {
                    *best = (squared, Some((point, at)));
                }
            }
            return;
        }
        let (near, far) = (node + 1, here.start as usize);
        let (near_distance, far_distance) = (
            self.nodes[near].squared_distance(query),
            self.nodes[far].squared_distance(query),
        );
        let order = if near_distance <= far_distance {
            [(near, near_distance), (far, far_distance)]
        } else {
            [(far, far_distance), (near, near_distance)]
        };
        for (child, distance) in order {
            if distance < best.0 {
                self.visit(query, child, best);
            }
        }
    }
}

impl Surface for MeshIndex {
    fn closest(&self, query: &Point) -> Option<Closest> {
        let mut best = (f64::INFINITY, None);
        self.visit(query, 0, &mut best);
        let (point, face) = best.1?;
        let normal: Unit<Vector> = self.faces[face].normal;
        Some(Closest {
            point,
            normal,
            squared_distance: best.0,
        })
    }
}

/// Lays `faces` out as a tree appended to `nodes`, whose faces start at `offset`.
fn build(faces: &mut [Face], offset: usize, nodes: &mut Vec<Node>) {
    let (low, high) = bounds(faces);
    let here = nodes.len();
    nodes.push(Node {
        low,
        high,
        start: u32::try_from(offset).unwrap_or(u32::MAX),
        faces: u32::try_from(faces.len()).unwrap_or(u32::MAX),
    });
    if faces.len() <= MOST_IN_A_LEAF {
        return;
    }
    let widest = (high - low).imax();
    faces.sort_unstable_by(|left, right| left.centre()[widest].total_cmp(&right.centre()[widest]));
    let middle = faces.len() / 2;
    let (first, second) = faces.split_at_mut(middle);
    build(first, offset, nodes);
    nodes[here].start = u32::try_from(nodes.len()).unwrap_or(u32::MAX);
    nodes[here].faces = 0;
    build(second, offset + middle, nodes);
}

fn bounds(faces: &[Face]) -> (Vector, Vector) {
    faces.iter().flat_map(|face| face.corners).fold(
        (
            Vector::repeat(f64::INFINITY),
            Vector::repeat(f64::NEG_INFINITY),
        ),
        |(low, high), corner| (low.inf(&corner.coords), high.sup(&corner.coords)),
    )
}
