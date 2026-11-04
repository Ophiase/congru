# Video

**[sketch]** One animated image per run, computed offline from real frames:

- **left**: the grey frame the camera saw, degraded, with the model's
  outline drawn at the estimated pose;
- **right**: the pose error over time, rotation and translation, with the
  moments the observability report flagged a free motion;
- **a strip of runs**, one per filter strength, so the reader sees where
  tracking holds and where it breaks.

Written as GIF from Rust, like the figures: no plotting stack, nothing drawn
by hand. The same frames, at full rate, show it runs in real time.

## [defined] Three layers side by side

The algorithm sees a point cloud, so a failure cannot be understood without
it. Every tracking GIF shows, for each frame, one row:

- the image;
- the depth, coloured by distance;
- the points the tracker was given, after the box and the crop;
- over all three, the same outline of the model at the pose found.
