# Embedded

**[defined]** `f32` and `no_std` exist: `crates/congru-embedded`, a second
crate beside the core, measured in
[its page](../../docs/benchmark/embedded.md).

**[guess]** The micro-robots of the vision run on microcontrollers or small
application processors: little memory, sometimes no operating system, often
no double-precision unit.

## What was decided

- **[defined]** A second crate, not a generalised core. The core's public
  types are `f64`, its numbers are published, and `nalgebra` and `kiddo`
  would come along. A skin may not hold an algorithm; this crate is no skin.
- **[caution]** So ICP now exists twice. A method added to the core does not
  reach a board by itself; the comparison on the page is what keeps the two
  honest.
- **[defined]** Generic over a small `Real` trait, for `f32` and `f64`, with
  `libm` and `thiserror` as the only dependencies.
- **[defined]** `alloc` once: the k-d tree and a `Workspace` are made up
  front, then [`Icp::align`] allocates nothing. A counting allocator watches
  it in a test.

## What it holds

- **[Kabsch] with weights and a [similarity]**, solved by [Horn]'s
  quaternion: a Jacobi eigenvalue solve of 4 × 4, so no SVD, no reflection
  case, and the scale comes from the same eigenvalue.
- **Point-to-point and point-to-plane ICP**, with a gate and the core's robust
  kernels. Trimming and the prior are left out.
- **A compact [k-d tree]**: the points themselves, reordered, one byte per
  inner node, leaves of eight, a fixed stack for a query.
- **Normals** from a fixed count of nearest points, on the stack.

## What `f32` asked for

- **[defined]** Centroids in two passes: rough, then corrected by the small
  differences. A plain sum of many coordinates loses the digits they need.
- **[defined]** Rotations are unit quaternions, renormalised at each
  composition: a chained matrix would drift.
- **[defined]** Point-to-plane levers are divided by their radius, so the
  normal equations are balanced in any unit.
- **[defined]** A stopping tolerance is never finer than `16 ε`, or `f32`
  spends its budget on rounding.
- **[defined]** A rotation whose eigenvalue gap is below `√ε` is refused:
  rounding alone would turn it about the data's line.
- **[caution]** `sinf` of `libm` computes in `f64`, which pulled soft doubles
  into `f32` images. Sine and cosine are a series here; the measuring recipe
  counts the helpers left, and must read zero.

## What is verified, and how

- **[defined]** Tests against the known truth in both scalars: rounding alone
  must leave a few tens of `ε` times the coordinates, noise what its variance
  allows.
- **[defined]** `just embedded-build` builds for `thumbv7em-none-eabihf` and
  links a bare-metal image, using a target others may lack, so it stays out of
  `just check`. On a host, `#![no_std]` already refuses a stray `std`.
- **[defined]** `just embedded-measure` writes the page: flash of the images,
  heap of the data, time and accuracy against the core.

## Left out

- **[caution]** Nothing has run on a board or in an emulator: no cycle count,
  no stack high-water mark beyond the query's own frame.
- **[question]** Which board first. The hardware `vsqrt` is not used: `libm`'s
  `sqrtf` is software, a small cost per iteration, unmeasured on the target.
- **[sketch]** Global search, marks and tracking are not here. A robot starts
  from a pose it already holds.

[`Icp::align`]: ../../crates/congru-embedded/src/icp/run.rs
[Kabsch]: https://en.wikipedia.org/wiki/Kabsch_algorithm
[similarity]: https://doi.org/10.1109/34.88573
[Horn]: https://doi.org/10.1364/JOSAA.4.000629
[k-d tree]: https://en.wikipedia.org/wiki/K-d_tree
