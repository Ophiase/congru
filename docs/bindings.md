# Bindings

One Rust core, a thin skin per language. A skin converts arrays, calls the
core and translates its errors; it never adds a default or a rule of its
own ([bindings](../ideas/architecture/bindings.md)).

| Language | State | Where |
| --- | --- | --- |
| Rust | The library itself | [crates/congru](../crates/congru/src/lib.rs) |
| Python | Working, tested | [crates/congru-python](../crates/congru-python/README.md) |
| C | Working, **unstable** until 1.0 | [congru.h](../crates/congru-capi/include/congru.h) |
| C++ | Working, header-only, **unstable** | [congru.hpp](../crates/congru-capi/include/congru.hpp) |
| Command line | Working, JSON on stdout | [congru-cli](../crates/congru-cli/src/main.rs) |

## Conventions shared by every language

| Thing | Layout |
| --- | --- |
| Points | `n × 3` doubles, row after row: `x0 y0 z0 x1 y1 z1 …` |
| Triangles | `m × 3` unsigned indices into the vertices |
| Transform | `4 × 4` doubles, row-major, homogeneous, last row `0 0 0 1` |
| Direction | An alignment's transform takes the **source** (the scan) onto the **target** (the model); an `Aligner`'s pose takes the **model** into the **sensor's** frame, its inverse |
| Units | Those of the points; angles in radians |
| Errors | One error type per language, carrying the core's message |

## Rust

```rust
use congru::{Icp, PointCloud, RigidTransform, Target};

let target = Target::with_estimated_normals(PointCloud::from_xyz(&model)?, 10)?;
let alignment = Icp::default().align(&PointCloud::from_xyz(&scan)?, &target, &RigidTransform::identity())?;
let row_major: [f64; 16] = alignment.transform.to_row_major();
```

The methods of the benchmark implement the
[`Aligner`](../ideas/architecture/contract.md) contract (`Local` holds an `Icp`,
`Search` a `Multistart`), so a caller can hold any of them as `Box<dyn Aligner>`.

## Python

```python
import numpy as np
import congru

target = congru.Target(model, neighbors=12)            # (n, 3) float64
alignment = congru.Icp().align(scan, target)           # (4, 4) row-major
alignment.observability                                # what was left free
```

| Python | Rust |
| --- | --- |
| `fit_rigid(source, target, weights=None)` | `fit_rigid` |
| `Target(points, *, normals=None, neighbors=None)`, `Target.from_mesh` | `Target` |
| `Icp(*, metric, kernel, max_correspondence_distance, overlap, max_iterations)` | `Icp` |
| `Multistart(*, starts, icp)` | `Multistart` |
| `Certifier(*, truncation, reach, margin, ...).run(source, target, seeds)` | `Certifier` |
| `fit_similarity(source, target)` | `fit_similarity` |
| `Surface.exact(v, t)`, `Surface.field(v, t, voxel, band)`, `.align(source, icp, initial)` | `MeshIndex`, `DistanceField`, `Icp::align_on` |
| `Metric.POINT_TO_POINT`, `Metric.POINT_TO_PLANE` | `Metric` |
| `Kernel.quadratic()`, `.huber(s)`, `.cauchy(s)`, `.geman_mcclure(s)`, `.tukey(s)` | `Kernel` |
| `RegistrationError(ValueError)` | `Error` |

Arrays already C-contiguous `float64` are read without a copy; others are
converted. The interpreter lock is released while a target is built and
while an alignment runs. The tracker and the `Aligner` contract are not
exposed yet.

## C and C++

The C ABI is a static or dynamic library, `congru_capi`, and one header.
Names, structs and values may change in any release until 1.0.

```cpp
#include "congru.hpp"

const congru::target target(model_xyz);           // std::span<const double>
const congru_alignment found = target.align(scan_xyz);
// found.transform: 16 doubles, row-major; found.converged, found.rmse
```

`just capi-example` builds the library and runs
[examples/align.cpp](../crates/congru-capi/examples/align.cpp) with g++.
Every pointer is checked, a panic never crosses the boundary, and a failure
returns a `congru_status` with its detail in `congru_last_error_message`.

## Command line

`congru align`, `congru track`, `congru describe` and `congru methods` read
`.xyz` and `.ply` files and print JSON, for scripts and agents.
