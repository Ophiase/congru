# Congru - Efficient point cloud alignment

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-green.svg)](LICENSE)
[![Rust 1.89+](https://img.shields.io/badge/Rust-1.89%2B-orange.svg)](https://www.rust-lang.org/)
[![Justfile](https://img.shields.io/badge/just-available-brightgreen.svg)](https://just.systems/man/en/)
[![Status: research](https://img.shields.io/badge/status-research-blue.svg)](ideas/open.md)

Put a scan back where it belongs. Given two views of the same rigid object
(a depth frame and a CAD model, a few marks and the part they sit on, two
passes of a lidar), congru finds the rotation and translation that bring
one onto the other, says how sure it is, and runs wherever the robot runs.

<p align="center">
  <img src="docs/figures/clouds.gif" alt="A noisy point cloud in motion, and the model holding on to it" width="49%">
  <img src="docs/figures/tracking.gif" alt="A machined part tracked through a simulated depth camera" width="49%">
</p>

<p align="center">
  <a href="docs/demos.md">The demos</a> ·
  <a href="docs/pipeline.md">The pipeline, layer by layer</a> ·
  <a href="docs/benchmark/protocol.md">How methods are tested</a> ·
  <a href="docs/benchmark/scenes.md">The scenes they are tested on</a> ·
  <a href="docs/benchmark/results.md">The results, method by method</a>
</p>

## Why this project exists

Registration is in every pipeline that looks at the 3D world: a robot
grasping a part, a drone mapping a building, a headset laying a model over
the real object. It is solved in papers and unsolved in practice: each team
rewrites ICP, tunes it on its own data, and finds out in the field where it
breaks.

The hardest corner is the one I met first: a camera finds a few marks on an
object, and they must land on its model from a rough first guess. The
difficulty is not speed. It is this:

![On the surface is not in the right place](docs/figures/marks-on-mesh.svg)

*Twelve noisy marks on a smooth, rounded patch of a model. ICP brings
every mark onto the surface, and the residual is tiny, yet the pose is far
off: on a rounded surface, a dozen points can slide without leaving it. The
same marks, numbered so each one knows its place, give the pose at once.*

A small residual is not a correct answer. A library that only returns the
residual lets that error through silently; congru says, with every
alignment, which motions the data left free: here, three turns about axes
as far away as the curvature of the surface
([observability](ideas/method/observability.md)).

## The problem, in three pictures

The project starts from a model of what a sensor does to an object
([observation model](ideas/problem/observation-model.md)).

![Resampling, outliers, rigid motion](docs/figures/problem.svg)

1. **Resampling.** Each point of the model (red) is seen a random number of
   times $N_i$, each time with Gaussian noise (blue). Some points are never
   seen.
2. **Outliers.** Points that belong to nothing (green) fall around the
   object: background, a hand, a reflection.
3. **Rigid motion.** Everything is turned and shifted by an unknown
   rotation $R$ and translation $t$. The task: find them.

Real sensors add one more corruption: they see only part of the object. The
same model, run forward with a chosen seed, generates test data with a known
answer, so every method here is **measured** against the truth, not
eyeballed ([measuring success](ideas/problem/measuring-success.md)).

## How it is solved

Rigid registration is a family of methods, not one algorithm; which one wins
depends on what is known beforehand and on what the data does
([the landscape](ideas/method/landscape.md)).

| Family | Needs | Methods | In congru |
| --- | --- | --- | --- |
| Correspondences | Which point goes with which | [Kabsch], [Umeyama], [RANSAC], [TEASER++] | Closed form; RANSAC inside `Consensus` |
| Local, no correspondences | A guess within about 20° | [ICP] point-to-point and [point-to-plane], [robust kernels], trimmed, annealed, adaptive; [NDT], [CPD] | The ICP family |
| Global | Nothing | Multistart over SO(3), [FPFH] with RANSAC, [Go-ICP] | `Search`, `Guarded`, `Consensus` |
| Tracking | The last frames | Any of the above inside a motion filter ([alpha-beta], [Kalman]) | Two-phase tracker |
| Learned | Training data | [PointNet], [Deep Closest Point], [GeoTransformer] | Out of the core |

[Kabsch]: https://en.wikipedia.org/wiki/Kabsch_algorithm
[Umeyama]: https://doi.org/10.1109/34.88573
[RANSAC]: https://en.wikipedia.org/wiki/Random_sample_consensus
[TEASER++]: https://arxiv.org/abs/2001.07715
[ICP]: https://en.wikipedia.org/wiki/Iterative_closest_point
[point-to-plane]: https://doi.org/10.1016/0262-8856(92)90066-C
[robust kernels]: https://en.wikipedia.org/wiki/M-estimator
[NDT]: https://en.wikipedia.org/wiki/Normal_distributions_transform
[CPD]: https://arxiv.org/abs/0905.2635
[FPFH]: https://doi.org/10.1109/ROBOT.2009.5152473
[Go-ICP]: https://arxiv.org/abs/1605.03344
[alpha-beta]: https://en.wikipedia.org/wiki/Alpha_beta_filter
[Kalman]: https://en.wikipedia.org/wiki/Kalman_filter
[PointNet]: https://arxiv.org/abs/1612.00593
[Deep Closest Point]: https://arxiv.org/abs/1905.03304
[GeoTransformer]: https://arxiv.org/abs/2202.06688

Every method is one [`Aligner`](ideas/architecture/contract.md) and goes
through the same [benchmark](docs/benchmark/protocol.md): three families of
models, thirteen situations that each break one hypothesis, six tracking
sequences, many seeds. A default is a default because a table says so, and
[the results](docs/benchmark/results.md) say where each method wins and
where it breaks.

<details>
<summary>The ideas, in pictures</summary>

![Iterative closest point, step by step](docs/figures/icp-steps.svg)

*The local family on a toy spiral: pair each observed point (blue) with its
nearest model point (red), move by the best fit of those pairs, repeat.*

![A local minimum, and the way out](docs/figures/local-minimum.svg)

*Without a guess, a local method slides into the nearest wrong answer; a
global search starts it from rotations spread over all of SO(3).*

![Least squares against a robust kernel](docs/figures/outliers.svg)

*Outliers pull least squares toward them; a robust kernel lowers the weight
of a pair as its distance grows.*

</details>

Every figure on this page is computed by the library from a seeded
scenario: `just figures` draws them again.

## Hypotheses, and dropping them

Every hypothesis buys speed. congru states each one, is excellent under
all of them first, then offers the next rung for when one fails; the
[benchmark](docs/benchmark/protocol.md) has a situation that breaks each.

| congru starts from | Which buys | The next rung |
| --- | --- | --- |
| A **rigid** object | Six numbers, closed forms | Bending modes, then kinematic chains |
| Points **mostly on the object** (a mask or crop upstream) | Least squares, mild kernels | Robust kernels and trimming, then consensus over an outlier majority |
| An object **mostly visible** | Reliable global shape | A motion prior, then local features |
| A **guess** within about 20° | Fast, accurate local ICP | Multistart over all rotations |
| A **calibrated** sensor | Points where the sensor says | Robust kernels, then distortion as an unknown |
| **Enough** points | Reliable normals | Point-to-point, closed form, and a report of what is left free |

The full ladder: [ideas/problem/hypothesis-ladder.md](ideas/problem/hypothesis-ladder.md).
congru is not a toolkit: no viewer, no file format zoo, no segmentation;
Open3D and PCL do that.

## Status

| Part | State |
| --- | --- |
| Closed form, weighted, with the reflection fix | Done, tested |
| ICP: point-to-point, point-to-plane, robust kernels, trimming | Done, tested |
| Multistart over SO(3) (super-Fibonacci spirals) | Done, tested |
| Mesh targets, sampled with face normals | Done, tested |
| Observability: which motions the data leaves free | Done, tested |
| Simulated tracking demo: depth camera, degradations, projective tracking | Done, measured |
| Python bindings, NumPy in and out | Done ([bindings](docs/bindings.md)) |
| C ABI and C++ header (unstable), command line printing JSON | Done ([bindings](docs/bindings.md)) |
| Real recordings, RGB-D and lidar | Milestone 2 ([milestones](ideas/milestones.md)) |
| One contract for every method: the `Aligner` trait | Done ([contract](ideas/architecture/contract.md)) |
| Benchmark: 7 one-shot methods, 3 trackers, 3 models, 19 situations | Done ([protocol](docs/benchmark/protocol.md), [results](docs/benchmark/results.md)) |
| Comparison with Open3D and small_gicp | Planned |

The questions still open are in [ideas/open.md](ideas/open.md).

## Try it

Rust 1.89+ and [just](https://just.systems):

```sh
just test      # every property, against the synthetic truth
just figures   # recompute the figures of this page
just demo      # run both demos, rewrite the animations, print the tables
just bench     # every method on every situation: docs/benchmark/results.md
```

```rust
use congru::{Icp, PointCloud, RigidTransform, Target};

let target = Target::with_estimated_normals(PointCloud::from_xyz(&model)?, 10)?;
let source = PointCloud::from_xyz(&scan)?;
let alignment = Icp::default().align(&source, &target, &RigidTransform::identity())?;
println!("{:?}", alignment.transform.to_row_major());
```

## The repository

| Folder | What it is |
| --- | --- |
| [ideas/](ideas/README.md) | Every concept, one per file, with its status: start here |
| [crates/congru/](crates/congru/src/lib.rs) | The library: geometry, search, closed form, ICP, multistart |
| [crates/congru-synth/](crates/congru-synth/src/lib.rs) | The observation model: scenarios with a known answer |
| [crates/congru-vision/](crates/congru-vision/src/lib.rs) | Depth cameras: projection, rendering, depth to points, support removal |
| [crates/congru-sim/](crates/congru-sim/src/lib.rs) | The simulated cell: the part, its turntable, the sensor's flaws |
| [crates/congru-track/](crates/congru-track/src/lib.rs) | Tracking: follow from frame to frame, find again after a jump |
| [crates/congru-bench/](crates/congru-bench/src/main.rs) | The benchmark: every method, every situation, many seeds |
| [crates/congru-python/](crates/congru-python/README.md) | The Python bindings, NumPy in and out |
| [crates/congru-demo/](crates/congru-demo/src/main.rs) | The two animations above |
| [crates/congru-figures/](crates/congru-figures/src/main.rs) | The figures of this page, computed from real runs |

Agents working on this code read [AGENTS.md](AGENTS.md).

## Standing on

- **Besl and McKay** (1992), and **Chen and Medioni** (1992): ICP,
  point-to-point and point-to-plane.
- **Kabsch** (1976) and **Umeyama** (1991): the closed form.
- **Fitzpatrick, West and Maurer** (1998): what fiducial noise does to the
  error where it matters, the reference for marker-based alignment.
- **Yang, Shi and Carlone** (2021), TEASER++: registration that certifies
  its own answer, the horizon.
- **Alexa** (2022): super-Fibonacci spirals, even samples of rotations.

The full list, with what each brings: [ideas/prior-art.md](ideas/prior-art.md).

## Contributors

- **Ophiase**: author; the observation model and its variants date from 2025.
- **Harmostes**: the AI agent working with Ophiase. In Greek, the
  *harmostēs* is the one who fits things together.

## Licence

[Apache 2.0](LICENSE). Milestones are in [CHANGELOG.md](CHANGELOG.md); how
to contribute in [CONTRIBUTING.md](CONTRIBUTING.md), and to report a flaw in
[SECURITY.md](SECURITY.md).
