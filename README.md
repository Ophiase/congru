# recalage

*Working name, see [naming](ideas/naming.md).*

Rigid registration you can ship: align a scan, or a handful of marked
points, onto a point cloud or a mesh, robust to noise, outliers and partial
views. One Rust core, called from Python, C and C++.

> **Status: research, pre-alpha.** The direction is written in
> [ideas/](ideas/README.md); the questions it still waits on are in
> [ideas/open.md](ideas/open.md). The code in `crates/` is a draft and may
> be rewritten once they are answered.

## Why

Aligning two views of a rigid object is solved in papers and unsolved in
practice: teams rewrite ICP, tune it on their own data, and cannot put it on
the robot. This project began with augmented-reality guidance for surgery,
where **a dozen noisy marks** on a bone must land on a **preoperative mesh**
from a **rough first pose**, the hardest corner of the problem
([origin](ideas/origin.md)).

## The problem

A model is observed through Gaussian resampling, scattered outliers, a
partial view, then an unknown rigid motion; the task is to recover the
motion ([observation model](ideas/problem/observation-model.md),
[hypotheses](ideas/problem/hypotheses.md),
[variants](ideas/problem/variants.md)). The same model generates test data
with a known truth, so every method is **measured** against it
([measuring success](ideas/problem/measuring-success.md)).

## The plan

The entry points follow what the caller already knows
([what we know](ideas/problem/what-we-know.md)):

| You know | Method | Idea |
| --- | --- | --- |
| Which point goes with which | Weighted Kabsch–Umeyama | [closed form](ideas/method/closed-form.md) |
| A rough pose | ICP, robust kernels, trimming | [ICP family](ideas/method/icp-family.md) |
| Nothing | ICP from rotations spread over SO(3) | [initialisation](ideas/method/global-initialisation.md) |

Around them:

- **Typed**: a non-empty finite cloud, a proper rigid motion, a positive
  distance; a misuse fails at the boundary
  ([types](ideas/architecture/types.md)).
- **Everywhere**: NumPy in, NumPy out; a C ABI with a header-only C++
  wrapper; `no_std` and `f32` designed for, built later
  ([bindings](ideas/architecture/bindings.md),
  [embedded](ideas/architecture/embedded.md)).
- **Fast where it counts**: the target indexed once and reused, queries in
  parallel ([performance](ideas/architecture/performance.md)).
- **A benchmark** of breaking points under each corruption, against Open3D
  and small_gicp ([benchmark](ideas/architecture/benchmark.md)).

## Layout

| Path | What |
| --- | --- |
| [ideas/](ideas/README.md) | The project's thinking, one concept per file |
| `crates/recalage` | The core (draft) |
| `crates/recalage-synth` | The observation model, for tests (draft) |
| [AGENTS.md](AGENTS.md) | How people and agents work here |

## Development

[just](https://just.systems) and stable Rust 1.89 or newer:

```sh
just check    # format, pedantic clippy, tests, markdown
just guards   # the repository's checks over the changed files
```

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[Apache-2.0](LICENSE).
