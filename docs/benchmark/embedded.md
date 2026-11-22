# Embedded

Written by `just embedded-measure`. Code is measured on images linked for
`thumbv7em-none-eabihf`, time and accuracy on this x86_64 machine, one thread.

- **Not run on a board**: this recipe links for the target and stops there,
  so nothing here is a cycle count. Where `f64` is emulated, its ratio to
  `f32` will be far worse than on this machine.
- **Same frames**: a bracket as in [the comparison](comparison.md), 16 seeds,
  a gate of 0.03 m, at most 50 iterations; Cauchy of scale 0.005 m.

## Code

One image: every fit, the tree, normals, both metrics, a panic that halts
and a bump allocator. Optimised for size with full link-time optimisation.

| Scalar | Flash, bytes | Helpers for soft `f64` |
| --- | ---: | ---: |
| `f32` | 19812 | 0 |
| `f64` | 35680 | 11 |

## Data

The whole heap of an alignment, read from the structures. The caller also
holds the source, 12 bytes a point in `f32`. A query adds a fixed stack.

| Scene | Target points | Source points | Tree `f32` | Tree `f64` | Buffer `f32` | Buffer `f64` |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| as in the comparison | 10840 | 4564 | 262207 | 522367 | 36512 | 73024 |
| on a board's budget | 948 | 550 | 22879 | 45631 | 4400 | 8800 |

Stack of one query: 512 bytes in `f32`.

## Time per iteration

All frames of a setup: total time over total iterations, in microseconds.

| Scene | Setup | `congru` | `f64` here | `f32` here | `f64` over `f32` |
| --- | --- | ---: | ---: | ---: | ---: |
| as in the comparison | point-to-plane | 6876 | 3913 | 3556 | 1.10 |
| as in the comparison | point-to-plane, Cauchy | 7012 | 4212 | 3818 | 1.10 |
| as in the comparison | point-to-point | 5537 | 3295 | 2990 | 1.10 |
| on a board's budget | point-to-plane | 386 | 221 | 207 | 1.07 |
| on a board's budget | point-to-plane, Cauchy | 250 | 194 | 176 | 1.10 |

Building the tree and estimating normals, `f32`, once:

| Target points | Tree, ms | Normals of 10 neighbours, ms |
| ---: | ---: | ---: |
| 10840 | 1.4 | 16.6 |
| 948 | 0.1 | 1.2 |

## Accuracy, as in the comparison

Median error over the seeds, then how many came within 1° and
5 mm of the truth.

| Setup | Situation | `congru` °, mm, near | `f64` °, mm, near | `f32` °, mm, near |
| --- | --- | ---: | ---: | ---: |
| point-to-plane | a guess 10° off | 0.045, 0.05, 16/16 | 0.044, 0.06, 16/16 | 0.039, 0.07, 16/16 |
| point-to-plane | 5 mm of noise | 0.484, 0.60, 16/16 | 0.497, 0.58, 16/16 | 0.509, 0.57, 16/16 |
| point-to-plane | 30 % strays | 0.123, 0.11, 16/16 | 0.126, 0.12, 16/16 | 0.131, 0.12, 16/16 |
| point-to-plane | half of it seen | 0.106, 0.13, 16/16 | 0.104, 0.13, 16/16 | 0.104, 0.13, 16/16 |
| point-to-plane | all of them, 20° off | 1.340, 2.93, 3/16 | 1.227, 2.46, 5/16 | 1.227, 2.72, 5/16 |
| point-to-plane, Cauchy | a guess 10° off | 0.049, 0.06, 16/16 | 0.048, 0.06, 16/16 | 0.044, 0.07, 16/16 |
| point-to-plane, Cauchy | 5 mm of noise | 0.953, 1.33, 9/16 | 0.950, 1.29, 10/16 | 0.971, 1.24, 9/16 |
| point-to-plane, Cauchy | 30 % strays | 0.055, 0.07, 16/16 | 0.053, 0.07, 16/16 | 0.053, 0.08, 16/16 |
| point-to-plane, Cauchy | half of it seen | 0.104, 0.13, 16/16 | 0.104, 0.13, 16/16 | 0.104, 0.13, 16/16 |
| point-to-plane, Cauchy | all of them, 20° off | 1.719, 3.38, 2/16 | 1.859, 3.27, 2/16 | 1.859, 3.27, 2/16 |
| point-to-point | a guess 10° off | 0.050, 0.09, 16/16 | 0.050, 0.09, 16/16 | 0.050, 0.09, 16/16 |
| point-to-point | 5 mm of noise | 0.474, 0.54, 16/16 | 0.474, 0.54, 16/16 | 0.474, 0.54, 16/16 |
| point-to-point | 30 % strays | 0.114, 0.17, 16/16 | 0.114, 0.17, 16/16 | 0.114, 0.17, 16/16 |
| point-to-point | half of it seen | 0.127, 0.17, 16/16 | 0.127, 0.17, 16/16 | 0.127, 0.17, 16/16 |
| point-to-point | all of them, 20° off | 1.858, 3.40, 4/16 | 1.858, 3.40, 4/16 | 1.859, 3.40, 4/16 |

## Accuracy, on a board's budget

Median error over the seeds, then how many came within 1° and
5 mm of the truth.

| Setup | Situation | `congru` °, mm, near | `f64` °, mm, near | `f32` °, mm, near |
| --- | --- | ---: | ---: | ---: |
| point-to-plane | a guess 10° off | 0.172, 0.22, 16/16 | 0.150, 0.21, 16/16 | 0.151, 0.18, 16/16 |
| point-to-plane | 5 mm of noise | 1.743, 2.08, 2/16 | 1.956, 2.41, 1/16 | 1.944, 2.28, 0/16 |
| point-to-plane | 30 % strays | 0.542, 0.36, 16/16 | 0.525, 0.36, 16/16 | 0.500, 0.33, 16/16 |
| point-to-plane | half of it seen | 0.297, 0.43, 16/16 | 0.343, 0.43, 16/16 | 0.343, 0.43, 16/16 |
| point-to-plane | all of them, 20° off | 3.931, 4.12, 1/16 | 3.872, 4.51, 1/16 | 3.980, 4.51, 1/16 |
| point-to-plane, Cauchy | a guess 10° off | 0.181, 0.23, 16/16 | 0.164, 0.22, 16/16 | 0.158, 0.20, 16/16 |
| point-to-plane, Cauchy | 5 mm of noise | 2.632, 3.21, 0/16 | 2.723, 3.31, 0/16 | 2.862, 3.49, 0/16 |
| point-to-plane, Cauchy | 30 % strays | 0.205, 0.22, 14/16 | 0.185, 0.23, 14/16 | 0.195, 0.21, 14/16 |
| point-to-plane, Cauchy | half of it seen | 0.344, 0.44, 16/16 | 0.360, 0.42, 16/16 | 0.360, 0.42, 16/16 |
| point-to-plane, Cauchy | all of them, 20° off | 3.990, 6.21, 1/16 | 4.237, 5.88, 1/16 | 4.237, 6.22, 1/16 |
