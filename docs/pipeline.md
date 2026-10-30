# The pipeline, layer by layer

What happens between a camera frame and a pose, in the demos and the
realistic scenes. Each layer is its own function; the table says where it
lives and what is still missing.

```text
 depth frame ─► points ─► no support ─► region ─► thinned ─► tracker ─► pose
 (RGB-D: the    pinhole    RANSAC        crop      voxel     predict,    + trust
 depth only)    back-      plane         around    grid      follow,     + free
                projection removed       a guess             find        motions
```

| Layer | What it does | Where | Today |
| --- | --- | --- | --- |
| **1. Frame** | A depth image: the distance along the optical axis at every pixel. The colour image is not used | [congru-vision](../crates/congru-vision/src/lib.rs) | Simulated: scenes rendered by rasterising meshes, then degraded (noise growing with depth, holes, flying pixels, lens distortion, a hand-held camera) by [congru-sim](../crates/congru-sim/src/lib.rs) |
| **2. Points** | Every valid pixel back-projected through the pinhole camera into a 3D point | `back_project` | Done |
| **3. Support removed** | The largest plane wider than the target (a table, a road, a shelf board) found by RANSAC on 3,000 sampled points, its points dropped | `without_support` | Done; one plane only |
| **4. Region of interest** | Only the points within about three quarters of the target's size around where it is expected | `within` | **No detector yet.** The first frame's centre comes from a simulated detector (the truth, a few degrees and centimetres off); later frames use the tracker's prediction |
| **5. Thinned** | One point per voxel, a sixtieth of the target's size: a density independent of distance | `PointCloud::voxel_downsample` | Done |
| **6. Tracker** | Predict the pose from the last ones (alpha-beta filter); align the region onto the side of the model the camera would see there (point-to-plane ICP, Tukey kernel scaled by the learned noise, pulled toward the prediction); if the frame fails its health check, search all rotations | [congru-track](../crates/congru-track/src/lib.rs) | Done; see [tracking](../ideas/method/tracking.md) |
| **7. Answer** | The pose, whether it is trusted, which motions the frame left free | `Update`, `Observability` | Done |

## Which demo uses which layers

| Animation | Starts at | Why |
| --- | --- | --- |
| Demo A, point clouds | Layer 5 | Synthetic point clouds, no camera: the alignment alone |
| Demo B, the turntable | Layer 1 | One part, a known room, a crop from the prediction |
| Realistic scenes | Layer 1 | Ordinary scenes, the same layers, a detector's first guess |

## What milestone 2 adds

- **A real detector** at layer 4: a segmentation network or, without one,
  clustering (DBSCAN) of what is left after layer 3: the *mostly the
  object* rung of the [ladder](../ideas/problem/hypothesis-ladder.md).
- **Colour** at layer 1, to tell apart objects of the same shape.
- **Real recordings**, RGB-D and lidar, in place of the simulated frame.

## [caution] Known bug

On four realistic scenes (the bin, the car, the shelf, the street) the
tracker ends near 150° from a first guess only 6° off, which a local
method cannot do on its own: a convention between layers, to find first.
