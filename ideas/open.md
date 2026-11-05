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

## Next, from the author's review of 0.1.1

**[defined]** Presentation is not done; these come before any new method.

1. **Colours**: the dark background with vivid particles reads harsh. Move
   the animations and figures to a light pastel palette, one shared theme.
2. **Demo B**:
   - the outline still drifts above the part on some frames: find out why
     (render offset, or a real pose error the tables hide) and fix it;
   - the background must be a real one, noisy at every scale: fractal
     noise, a sum of Perlin octaves of growing frequency and shrinking
     amplitude, on the wall and the bench;
   - the camera shake must be visible and tested, not only a jolt of the
     pose.
   - **[defined]** In a room, the region kept is centred on a wrong pose
     found on the turntable, so the part itself is never given to the
     tracker, while the depth shows it plainly: detection is what fails.
3. **Scenes**: the gallery shows too many simple synthetic variations.
   What matters is a few ordinary scenes, found free online with a clear
   licence, with one object to cut out of them; and the tooling that makes
   adding such a scene one command (fetch a single file, check its sum,
   record it in [catalogue/sources.yaml](../catalogue/sources.yaml)).

## Next, from the author's review of demo B

**[defined]** Demo A is solved; what fails in a room is not missing data. The
frame says more than points ([visibility](problem/visibility.md)).

**[sketch]** On a branch, proposals for 1, 2 and 5, measured, for the author to
keep or not: [what a pose is worth](method/pose-information.md). Questions 3 and 4
stay open, and [step 7 of the ladder](problem/cloud-ladder.md) shows why they matter.

**[question]** Far scenes: the simulated sensor has about 1 m of noise at 14 m,
close to a car's height, so even the truth scores zero. Give them a lidar's noise?

**[question]** A new one: the author's intuition is a person reads surfaces, not
points. Segments work in 2D without any start; faces help in 3D but not yet in a
room. Should the model become faces and edges everywhere, ahead of points?

1. **[question]** Keep the detector as measured: every vote judged, refined along
   rays, share in frame? Room stays at 0 / 90, and it costs time
   ([evidence along rays](method/ray-evidence.md)).
2. **[question]** Move the tracker to rays with a budget of K points, and to three
   verdicts: supported, hidden, refuted. It breaks a right start in one update.
3. **[question]** Is a physical prior allowed, that a part neither sinks nor floats?
   The sensor alone leaves it open.
4. **[question]** Should the answer be a set of poses when several survive?
5. **[question]** How to detect in clutter without a 2D box? Without one, the room
   still gives none, then 160° off.
