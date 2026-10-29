# Open questions

What the author settles before the code goes further. Each links to the
file that argues it.

1. **Which variant first?** The marker case (few marks, rough pose, a
   mesh) or the scan case (thousands of points, no pose)?
   [variants](problem/variants.md)
2. **Is aggregation a contribution?** The 2025 idea of collapsing the
   resampled observations back to one per model point, before aligning.
   [aggregation](method/aggregation.md)
3. **Global initialisation**: start with multistart over rotations (simple,
   no descriptors), or go straight to features with a certifiable solver?
   [global-initialisation](method/global-initialisation.md)
4. **Mesh targets**: sample the mesh, or exact point-to-triangle distance
   through a bounding-volume hierarchy?
   [mesh-targets](method/mesh-targets.md)
5. **Precision**: `f64` everywhere, or generic over `f32` for embedded
   targets? [embedded](architecture/embedded.md)
6. **Frames in the types**: is `Transform<From, To>` worth its weight in the
   bindings? [types](architecture/types.md)
7. **The name**. [naming](naming.md)
8. **A paper?** The benchmark against the observation model, with breaking
   points per method, may be the publishable part
   ([benchmark](architecture/benchmark.md)).
