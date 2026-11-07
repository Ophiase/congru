use congru::{Distance, DistanceField, MeshIndex, PointCloud, Surface, TriangleMesh};

use super::icp::{AlignmentOut, IcpOptions, congru_icp_options_default, icp, translated};
use crate::boundary::{Failure, cloud, guard, transform, write};
use crate::status::Status;

/// `congru_surface` in the header: a mesh held for alignment, opaque to C, freed
/// by [`congru_surface_free`].
#[derive(Debug)]
pub enum SurfaceHandle {
    Exact(MeshIndex),
    Field(DistanceField),
}

/// # Safety
/// `vertices` points at `3 × vertex_count` readable doubles, `triangles` at
/// `3 × triangle_count` readable indices.
unsafe fn mesh(
    vertices: *const f64,
    vertex_count: usize,
    triangles: *const u32,
    triangle_count: usize,
) -> Result<TriangleMesh, Failure> {
    let points: PointCloud = unsafe { cloud(vertices, vertex_count, "vertices") }?;
    if triangles.is_null() {
        return Err(Failure::null("triangles"));
    }
    let length = triangle_count
        .checked_mul(3)
        .ok_or_else(|| Failure::new(Status::InvalidInput, "triangles: the count overflows"))?;
    let indices = unsafe { std::slice::from_raw_parts(triangles, length) };
    let faces: Vec<[u32; 3]> = indices.as_chunks::<3>().0.to_vec();
    Ok(TriangleMesh::new(points, faces)?)
}

/// Holds a mesh for exact distances, by a tree of boxes: the most accurate, the
/// slowest to query.
///
/// # Safety
/// As for the vertices and triangles above; `out` points at a writable pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_surface_new_mesh(
    vertices: *const f64,
    vertex_count: usize,
    triangles: *const u32,
    triangle_count: usize,
    out: *mut *mut SurfaceHandle,
) -> Status {
    guard(|| {
        let mesh = unsafe { mesh(vertices, vertex_count, triangles, triangle_count) }?;
        let handle = SurfaceHandle::Exact(MeshIndex::new(&mesh)?);
        unsafe { write(out, Box::into_raw(Box::new(handle)), "out") }
    })
}

/// Holds a mesh as a signed distance field of cubes of side `voxel`, truthful within
/// `band` of the surface: the fastest to query, once built. `band` must exceed the
/// correspondence distance of the alignments.
///
/// # Safety
/// As [`congru_surface_new_mesh`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_surface_new_field(
    vertices: *const f64,
    vertex_count: usize,
    triangles: *const u32,
    triangle_count: usize,
    voxel: f64,
    band: f64,
    out: *mut *mut SurfaceHandle,
) -> Status {
    guard(|| {
        let mesh = unsafe { mesh(vertices, vertex_count, triangles, triangle_count) }?;
        let field = DistanceField::from_mesh(&mesh, Distance::new(voxel)?, Distance::new(band)?)?;
        let handle = SurfaceHandle::Field(field);
        unsafe { write(out, Box::into_raw(Box::new(handle)), "out") }
    })
}

/// Frees a surface; null is ignored.
///
/// # Safety
/// `surface` is null or a handle not freed yet.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_surface_free(surface: *mut SurfaceHandle) {
    if !surface.is_null() {
        drop(unsafe { Box::from_raw(surface) });
    }
}

/// As `congru_icp_align`, onto the surface itself.
///
/// # Safety
/// `surface` is a live handle; the rest as `congru_icp_align`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn congru_surface_align(
    surface: *const SurfaceHandle,
    xyz: *const f64,
    count: usize,
    initial: *const f64,
    options: *const IcpOptions,
    out: *mut AlignmentOut,
) -> Status {
    guard(|| {
        let surface = unsafe { surface.as_ref() }.ok_or_else(|| Failure::null("surface"))?;
        let source = unsafe { cloud(xyz, count, "xyz") }?;
        let initial = unsafe { transform(initial, "initial") }?;
        let options = unsafe { options.as_ref() }
            .copied()
            .unwrap_or_else(|| congru_icp_options_default());
        let icp = icp(&options)?;
        let alignment = match surface {
            SurfaceHandle::Exact(index) => aligned(&icp, &source, index, &initial),
            SurfaceHandle::Field(field) => aligned(&icp, &source, field, &initial),
        }?;
        unsafe { write(out, translated(&alignment), "out") }
    })
}

fn aligned(
    icp: &congru::Icp,
    source: &PointCloud,
    surface: &impl Surface,
    initial: &congru::RigidTransform,
) -> Result<congru::Alignment, congru::Error> {
    icp.align_on(source, surface, initial)
}
