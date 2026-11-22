# The ICP family

**[defined]** Iterative closest point: pair each source point with its
nearest target point, solve for the motion as if those pairs were right,
move, repeat. It converges to the nearest local minimum, so it needs a pose
within its basin.

## The two metrics worth having

- **Point-to-point** (Besl–McKay): each step is the closed form on the
  current pairs. No normals; slow on smooth surfaces, which slide.
- **Point-to-plane** (Chen–Medioni): the residual is the distance to the
  tangent plane of the matched point, linearised in the rotation; a 6×6
  system per step. **[to check]** converges in far fewer iterations on
  surfaces, the usual default.

**[question]** Generalised ICP (plane-to-plane) is the next step in
accuracy; is it worth a third metric, or a later extension?

## The loop, made explicit

1. **Match**: nearest neighbour through a k-d tree built once on the target.
2. **Reject**: drop pairs past a distance; keep only the closest share
   (trimming, for partial overlap).
3. **Weigh**: a robust kernel of the residual ([robustness](robustness.md)).
4. **Solve**: closed form or the 6×6 system.
5. **Stop**: when a step moves less than a tolerance, or the budget ends.

**[sketch]** Each stage is a choice with a name and a type, so a user reads
the configuration as a sentence and the tests can vary one stage at a time.

## [caution]

- The residual is not a measure of success: a wrong minimum can fit well.
- Normals estimated from a noisy scan have an arbitrary sign; the
  point-to-plane residual must not depend on it.
