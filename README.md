# recalage

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-green.svg)](LICENSE)
[![Rust 1.89+](https://img.shields.io/badge/Rust-1.89%2B-orange.svg)](https://www.rust-lang.org/)
[![Justfile](https://img.shields.io/badge/just-available-brightgreen.svg)](https://just.systems/man/en/)
[![Status: research](https://img.shields.io/badge/status-research-blue.svg)](ideas/open.md)

Put a scan back where it belongs. Given two views of the same rigid object
(a depth frame and a CAD model, a few marks and the part they sit on, two
passes of a lidar), recalage finds the rotation and translation that bring
one onto the other, says how sure it is, and runs wherever the robot runs.

*Recalage* is the French word for registration.

![A machined part tracked through a degraded depth camera](docs/figures/tracking.gif)

*A simulated robot cell: a machined bracket turns on a turntable under a
depth camera. Each panel is what the camera returns, shaded by its surface
normals; the green outline is the part's model drawn at the pose recalage
found; the bar below is the rotation error (green under 1°, amber under 5°).
Top left: exact depth. Top right: noise and blur. Bottom left: holes and
flying pixels. Bottom right: all of it, with the turntable in view.*

| Condition | Median error | Frames within 2° |
| --- | --- | --- |
| Exact depth | 0.05° | 90 / 90 |
| Noise and blur | 0.48° | 86 / 90 |
| Holes and flying pixels | 0.22° | 81 / 90 |
| Everything, and the turntable | 0.71° | 90 / 90 |

Computed offline, frame by frame, by `just demo`; the pipeline (support
plane removed, scan cropped, aligned onto the part of the model visible
from the predicted pose) is real-time by design
([the demo](ideas/demo/README.md)).

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
residual lets that error through silently; recalage says, with every
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

![Iterative closest point, step by step](docs/figures/icp-steps.svg)

*Iterative closest point on a toy spiral. Red crosses are the model, blue
dots a noisy observation of it, moved. Each step pairs every dot with its
nearest cross, moves the dots by the motion that best fits those pairs, and
pairs again.*

What to do depends on what is known beforehand
([what we know](ideas/problem/what-we-know.md)).

| You know | recalage uses | Cost |
| --- | --- | --- |
| Which point goes with which | The closed form of Kabsch and Umeyama | One 3×3 SVD |
| A rough pose, within about 20° | ICP, point-to-point or point-to-plane | A few nearest-neighbour passes |
| Nothing at all | ICP from many rotations spread over SO(3) | A few dozen ICPs |

Without a rough pose, ICP slides into the nearest wrong answer. Starting it
from rotations spread evenly over all of SO(3) and keeping the one that
explains the most points finds the right one
([global initialisation](ideas/method/global-initialisation.md)):

![A local minimum, and the way out](docs/figures/local-minimum.svg)

Outliers pull a least-squares fit toward them. A robust kernel lowers the
weight of a pair as its distance grows, until outliers count for nothing
([robustness](ideas/method/robustness.md)):

![Least squares against a robust kernel](docs/figures/outliers.svg)

Every figure on this page is computed by the library from a seeded
scenario: `just figures` draws them again.

## What it is not

- **Not a toolkit.** No viewer, no file formats zoo, no segmentation:
  Open3D and PCL do that. recalage does one thing.
- **Not non-rigid.** Six numbers, a rotation and a translation; scale is the
  first extension worth having.
- **Not finished.** The core aligns and is tested against the truth; the
  bindings, the benchmark and real recordings are next.

## Status

| Part | State |
| --- | --- |
| Closed form, weighted, with the reflection fix | Done, tested |
| ICP: point-to-point, point-to-plane, robust kernels, trimming | Done, tested |
| Multistart over SO(3) (super-Fibonacci spirals) | Done, tested |
| Mesh targets, sampled with face normals | Done, tested |
| Observability: which motions the data leaves free | Done, tested |
| Simulated tracking demo: depth camera, degradations, projective tracking | Done, measured |
| Python (NumPy) and C/C++ bindings, command line | In progress ([bindings](ideas/architecture/bindings.md)) |
| Real recordings, RGB-D and lidar | Milestone 2 ([milestones](ideas/milestones.md)) |
| Benchmark of breaking points against Open3D and small_gicp | Planned ([benchmark](ideas/architecture/benchmark.md)) |

The questions still open are in [ideas/open.md](ideas/open.md).

## Try it

Rust 1.89+ and [just](https://just.systems):

```sh
just test      # every property, against the synthetic truth
just figures   # recompute the figures of this page
just demo      # run the tracking demo, rewrite the animation
```

```rust
use recalage::{Icp, PointCloud, RigidTransform, Target};

let target = Target::with_estimated_normals(PointCloud::from_xyz(&model)?, 10)?;
let source = PointCloud::from_xyz(&scan)?;
let alignment = Icp::default().align(&source, &target, &RigidTransform::identity())?;
println!("{:?}", alignment.transform.to_row_major());
```

## The repository

| Folder | What it is |
| --- | --- |
| [ideas/](ideas/README.md) | Every concept, one per file, with its status: start here |
| [crates/recalage/](crates/recalage/src/lib.rs) | The library: geometry, search, closed form, ICP, multistart |
| [crates/recalage-synth/](crates/recalage-synth/src/lib.rs) | The observation model: scenarios with a known answer |
| [crates/recalage-vision/](crates/recalage-vision/src/lib.rs) | Depth cameras: projection, rendering, depth to points, support removal |
| [crates/recalage-sim/](crates/recalage-sim/src/lib.rs) | The simulated cell: the part, its turntable, the sensor's flaws |
| [crates/recalage-demo/](crates/recalage-demo/src/main.rs) | The tracker and the animation above |
| [crates/recalage-bench/](crates/recalage-bench/src/main.rs) | The figures of this page, computed from real runs |

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
