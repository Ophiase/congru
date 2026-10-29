# Vision

**[sketch]** A registration brick small and sure enough to sit inside a
robot.

## For whom

- **Computer-vision engineers** who align a scan onto a model, a mesh onto a
  mesh, a depth frame onto the last one, and today glue Open3D or PCL into a
  pipeline they cannot ship.
- **Augmented reality and calibration**: a handful of marked points onto a
  CAD model, where the error must be bounded and explained
  ([origin.md](origin.md)).
- **Small robots**: arms, drones, inspection crawlers, the micro-robots to
  come, with a weak CPU, no GPU, sometimes no operating system
  ([embedded](architecture/embedded.md)).

## What it promises

1. **One answer per situation**: correspondences known, a rough pose known,
   nothing known ([variants](problem/variants.md)).
2. **Measured, not claimed**: every method comes with its breaking point
   under noise, outliers and overlap, on a generator with a known truth
   ([measuring-success](problem/measuring-success.md)).
3. **Typed**: a number with a meaning has a type; a misuse fails to compile
   or fails loudly, never silently ([types](architecture/types.md)).
4. **Everywhere**: one Rust core; Python, C and C++ as thin skins
   ([bindings](architecture/bindings.md)).
5. **Fast enough for a control loop**: the target is indexed once and
   reused frame after frame ([performance](architecture/performance.md)).

## What it is not

- **[defined]** Not a point-cloud toolkit: no viewer, no file zoo, no
  segmentation. Open3D and PCL do that; this does one thing.
- **[sketch]** Not non-rigid, not scale, at first: a rigid motion has six
  numbers, and six numbers can be certified. Similarity (with scale) is the
  first extension worth having.
