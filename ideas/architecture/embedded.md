# Embedded

**[guess]** The micro-robots of the vision run on microcontrollers or small
application processors: little memory, sometimes no operating system, often
no double-precision unit.

## What it would take

- **`no_std` with `alloc`**: the core free of the standard library, the
  thread pool behind a feature.
- **`f32`**: generic over the scalar, or a second build. Halves memory and
  doubles speed where `f64` is emulated.
- **No allocation in the loop**: buffers sized once, from the source and the
  target, reused each iteration.
- **A spatial index that fits**: the k-d tree crate must itself work without
  the standard library. **[to check]**

## [question] Now or later

Designing for it costs little now (keep I/O and threads out of the core,
avoid allocating per iteration) and much later. Building it is not urgent.
**[guess]** design for it, build it after the first release.
