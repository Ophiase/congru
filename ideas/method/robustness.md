# Robustness

**[sketch]** Three tools against what the observation model throws at ICP,
each for a different corruption.

| Corruption | Tool | Parameter |
| --- | --- | --- |
| Outliers far from the surface | Distance threshold | A distance |
| Outliers near the surface, heavy noise | Robust kernel (IRLS) | A scale, about $3\sigma$ |
| The source sees only part of the target | Trimming | The overlap share |

## Kernels

The weight of a pair given its residual $r$ and a scale $c$: quadratic
(always 1), Huber, Cauchy, Geman–McClure, Tukey, from the mildest to the
harshest. **[to check]** Babin et al. (2019) found Cauchy and Geman–McClure
the safest defaults for registration.

## [question] The scale

Every kernel needs $c$, and a wrong $c$ breaks it. Options:

- the caller gives it, from the sensor's noise (simple, honest);
- it is estimated each iteration from the median residual (robust, adds a
  hidden behaviour);
- adaptive kernels (Chebrolu et al. 2021), which also choose the shape.

**[guess]** Start with the first, measure the second on the generator.
