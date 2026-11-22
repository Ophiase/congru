# Perception

**[sketch]** From a depth image to the source cloud of the tracker:

1. **Back-project** each valid pixel through the camera intrinsics.
2. **Crop** to a box around the predicted pose, so the table and the
   clutter mostly stay out.
3. **Downsample** on a voxel grid: fewer points, same geometry, and a
   density that no longer depends on the distance to the camera.

Each step is a function of its own, usable on a real sensor's frames
([M2](../milestones.md)).
