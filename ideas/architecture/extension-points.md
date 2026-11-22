# Extension points

**[sketch]** congru should be a brick others build on without its author:
a few seams where a user plugs their own piece, no abstraction for its own
sake. The tracker already has one, the `ModelView` trait.

| Seam | What a user would plug in | Status |
| --- | --- | --- |
| A target trait: closest point, normal, distance for a query | A mesh BVH, a signed distance field, a voxel grid, a GPU index | **[question]** the kd-tree cloud becomes one implementation; costs a generic on `Icp::align` |
| A kernel trait: `weight(residual)` | Welsch, a learned weight, an adaptive kernel | **[sketch]** the enum would implement it and stay the default |
| A public single ICP step: pair, weigh, solve | Coarse to fine, IMU fusion, a custom stopping rule | **[sketch]** the loop in `icp/run.rs` is already three calls |
| `#[non_exhaustive]` on public enums and configuration structs | Growth without breaking users | **[caution]** forbids struct literals outside the crate, which the skins and demos use; to decide before 1.0 |
| Plain exchange types at the boundaries (`[f64; 3]`, slices) | No forced nalgebra | **[sketch]** `PointCloud::from_xyz` and `to_row_major` already exist |
| A minimal C ABI | Unity plugins, embedded boards | **[sketch]** unstable until the author declares it stable, at 1.0 at the earliest |

**[defined]** Adopted so far: `ModelView` in the tracker, and `Prior` as an
optional part of `Icp` rather than a fork of the loop. The rest waits for a
user who needs it, so the seam is shaped by a real case.
