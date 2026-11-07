//! `align` is watched by a counting allocator, not trusted.

#![expect(
    clippy::unwrap_used,
    reason = "a broken fixture should fail the test loudly"
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use congru_embedded::{Icp, KdTree, Metric, Pose, Real, Workspace};
use congru_synth::shapes;

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

struct Counting;

#[expect(unsafe_code, reason = "a global allocator can only be written so")]
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|count| count.set(count.get() + 1));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

fn align_allocates_nothing<Float: Real>(metric: Metric) {
    let model = shapes::wavy_patch(30).unwrap();
    let points: Vec<[Float; 3]> = model
        .points()
        .iter()
        .map(|point| [point.x, point.y, point.z].map(Float::of))
        .collect();
    let mut target = KdTree::new(&points).unwrap();
    target.estimate_normals::<10>().unwrap();
    let mut workspace = Workspace::new(points.len());
    let icp = Icp {
        metric,
        ..Icp::default()
    };
    let before = ALLOCATIONS.with(Cell::get);
    let found = icp.align(&points, &target, &Pose::identity(), &mut workspace);
    let after = ALLOCATIONS.with(Cell::get);
    assert!(found.unwrap().iterations >= 1);
    assert_eq!(after, before);
}

#[test]
fn the_loop_never_allocates() {
    align_allocates_nothing::<f32>(Metric::PointToPlane);
    align_allocates_nothing::<f32>(Metric::PointToPoint);
    align_allocates_nothing::<f64>(Metric::PointToPlane);
    align_allocates_nothing::<f64>(Metric::PointToPoint);
}
