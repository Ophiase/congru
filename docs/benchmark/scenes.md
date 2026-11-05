# Scenes

## Realistic scenes

![One frame, step by step](../figures/breakdown.gif)

*How one frame is processed: depth, the 2D detector's box (simulated),
the model found by shape, region, voxels, model tracked
([the pipeline](../pipeline.md)).*

Ordinary scenes written as short TOML files in
[scenes/](../../scenes/), rendered by a simulated depth camera; the
target is tinted. Adding one is writing one file.

Each scene, frame by frame: [the films](../scene-films.md).

**Thirteen scenes of fourteen hold at least in part**, none used to choose
anything. Every film: [scene films](../scene-films.md).

They run the pipeline of [demo B](../hold-step.md): a noisy box in place of a
detector, poses along rays, the trajectory held when a frame says nothing.

| Scene | Within 2 % of its size, of 72 | Within 2° |
| --- | --- | --- |
| Part on a table | **72** | 69 |
| Bracket in a bin | **72** | 64 |
| Mug close up | **72** | 72 |
| Bin of parts | **71** | 0: a gear, right up to some teeth |
| Twin brackets | **71** | 72 |
| Bracket at a low angle | **68** | 38 |
| Gear on a bench | **63** | 19: a gear |
| Bracket behind a crate | **60** | 20 |
| Street corner | **57** | 42 |
| Gear from above | 47 | 0: a gear, seen flat |
| Shelf of mugs | 45 | 0: the handle hides |
| Car on a road | 42 | 39 |
| Car close by | 39 | 28 |
| Crate among crates | **0** | 0: a regression, was 72 |

**[caution]**

- **The first column is [ADD-S]**, the distance to the true surface, blind to
  what a symmetry leaves unchanged. A rotation error fails a gear turned by one
  tooth, so both are shown.
- **The box comes from the truth**, until a real detector runs.
- **The two far scenes are given a lidar's noise** in their files: a stereo camera
  has 1 m of noise at 14 m, for a car 1.5 m high.
- **The breakdown above still shows the previous pipeline.**

[ADD-S]: https://arxiv.org/abs/1711.00199

![The realistic scenes](scenes/realistic.png)

| Scene, left to right | What it tests |
| --- | --- |
| bin of parts | A gear on top of a jumble of brackets in an open bin, seen from above: the picking problem. |
| car on a road | A car twelve metres ahead on a bumpy road, another behind it, trees and houses along the side. |
| gear on a bench | A gear lying among a bracket, a mug and crates on a workbench: twelve teeth, twelve near-answers. |
| part on a table | A machined bracket on a table, a mug and a crate beside it, a wall behind. |
| shelf of mugs | A row of mugs on a shelf, one of them the target, all alike but for their handles. |
| street corner | A car turning at a corner twenty metres away, half hidden by a parked one: far, sparse, partly seen. |

## Simple situations

Bare point clouds, every situation of the protocol:
[their own page](scenes-simple.md).
