# Demos

Both animations are computed from real runs, frame by frame; `just demo`
writes them again and prints the tables below.

## A — point clouds over time

![Demo A](figures/clouds.gif)

*Demo A: point clouds only. A machined bracket moves in front of a sensor;
the blue points are what the sensor returns, noisy and partly hidden; the
red points are the model, placed at the pose congru finds, frame after
frame. The text says which phase ran (follow from the last pose, or find
again from nothing) and the rotation error against the truth. Top left:
noise. Top right: a ball crossing in front. Bottom left: lens distortion.
Bottom right: two knocks, after which the model is found again.*

| Run | Median error | Frames within 2° |
| --- | --- | --- |
| Noise | 0.72° | 120 / 120 |
| A ball crossing in front | 0.19° | 98 / 120 |
| Lens distortion | 0.63° | 120 / 120 |
| Two knocks | 0.11° | 118 / 120 |

## B — a simulated depth camera

![Demo B](figures/tracking.gif)

*Demo B: a simulated robot cell. A machined bracket turns on a turntable under a
depth camera. Each panel is what the camera returns, shaded by its surface
normals; the green outline is the part's model drawn at the pose congru
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
([the demo](../ideas/demo/README.md)).
