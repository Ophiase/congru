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

**Twelve scenes of fourteen hold more than nine frames in ten**, the two cars 60
and 54 of 72. Every film: [scene films](../scene-films.md).

- **Used to choose**: the crate, whose regression was fixed on it, the gear, whose
  bug it showed, and five more to test the changes.

They run the pipeline of [demo B](../answer-step.md): a noisy box in place of a
detector, poses along rays then a short alignment, the trajectory held when a frame
says nothing.

| Scene | Within 2 % of its size, of 72 | Within 2° |
| --- | --- | --- |
| Part on a table | **72** | 72 |
| Mug close up | **72** | 72 |
| Twin brackets | **72** | 72 |
| Bracket in a bin | **72** | 71 |
| Street corner | **72** | 70 |
| Bracket at a low angle | **72** | 69 |
| Bin of parts | **72** | 0: a gear, right up to some teeth |
| Gear on a bench | **72** | 0: a gear |
| Gear from above | **72** | 0: a gear, seen flat |
| Crate among crates | **72** | 0: a box, right up to a half turn |
| Shelf of mugs | **72** | 0: the handle hides |
| Bracket behind a crate | **66** | 64 |
| Car on a road | **54** | 54 |
| Car close by | 48 | 48 |

**[caution]**

- **The first column is [ADD-S]**, the distance to the true surface, blind to
  what a symmetry leaves unchanged. A rotation error fails a gear turned by one
  tooth, so both are shown.
- **The box comes from the truth**, until a real detector runs.
- **The two far scenes are given a lidar's noise** in their files: a stereo camera
  has 1 m of noise at 14 m, for a car 1.5 m high.
- **The breakdown above shows the first frame's search, not the follower.**
- **2 % of a car's size is 10 cm**: a bound relative to the object, not to a task.

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
