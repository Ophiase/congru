# Scenes

## Realistic scenes

![One frame, step by step](../figures/breakdown.gif)

*How one frame is processed: depth, the 2D detector's box (simulated),
the model found by shape, region, voxels, model tracked
([the pipeline](../pipeline.md)).*

Ordinary scenes written as short TOML files in
[scenes/](../../scenes/), rendered by a simulated depth camera; the
target is tinted. Adding one is writing one file.

![The realistic scenes, filmed by a camera in a hand](../figures/scenes.gif)

*Each scene filmed by a wandering camera while its target moves; the green
outline is where the tracker puts the target, the number its rotation
error. **[caution] Every scene fails today: 0 of 72 frames within 2°.**
Until 0.1.15 the first pose came from the truth nudged by a few degrees
and the supporting plane was removed; both were shortcuts a headset does
not have, and are gone. The target is now found by its shape inside a
simulated 2D box: on a clean frame that works (a test checks it), in the
simulated sensor's noise it does not yet, and the car and the street
corner, far away, are not found at all. `just demo` films them again.*

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
