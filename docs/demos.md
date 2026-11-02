# Demos

Both animations are computed from real runs, frame by frame; `just demo`
writes them again and prints the tables below.

## A — point clouds over time

![Demo A](figures/clouds.gif)

A machined bracket moves in front of a sensor, in six generated sequences
of 120 frames. The red particles are what the sensor returns; the cyan
points are the model, placed at the pose congru finds, frame after frame.
Each tile says which phase ran (follow from the last pose, or find again
from nothing) and the rotation error against the truth; the bar below
turns amber past 1° and red past 5°.

Every sequence is already hard: a few hundred points per frame, 5 mm of
noise on a 20 cm part, and 40 points of clutter. Each adds one trouble:

| Run | Added | Median error | Frames within 2° |
| --- | --- | --- | --- |
| Noise | nothing | 0.91° | 114 / 120 |
| Shake | the sensor wanders as in a hand, up to 6° and 2.5 cm over two seconds, and jolts every frame | 1.07° | 109 / 120 |
| Bend | the part flexes by up to 4 mm | 1.07° | 112 / 120 |
| Occluder | a ball crosses in front | 0.94° | 112 / 120 |
| Lens | radial distortion | 1.17° | 103 / 120 |
| Knocks | two abrupt turns of 70° and 60° | 0.95° | 113 / 120 |

For scale: aligning one such frame alone, from the true pose, already
leaves about 0.85° of error. What tracking adds is holding on: through the
ball, the jolts and the knocks, the pose never drifts away.

How it works: the follow phase aligns each frame onto the side of the model
facing the sensor, pulled toward the predicted pose, with a robust kernel
scaled by the noise it has learned; a frame it cannot explain is searched
again at once ([tracking](../ideas/method/tracking.md)).

## B — a simulated depth camera

![Demo B](figures/tracking.gif)

*Demo B: a simulated robot cell. A machined bracket turns on a turntable
under a depth camera. Each tile is what the camera returns, shaded by its
surface normals; the green outline is the model drawn at the pose congru
finds; the number is the rotation error against the truth.*

| Condition | Added to an imperfect sensor | Median error | Frames within 2° |
| --- | --- | --- | --- |
| Sensor | noise, blur, holes, flying pixels | 0.10° | 90 / 90 |
| Shake | a camera in a hand: slow wander and jolts | 0.52° | 90 / 90 |
| Lens | radial distortion of the depth image | 0.28° | 87 / 90 |
| Flex | the part flexes by up to 4 mm | 0.16° | 90 / 90 |
| Room | a wall, a bench, a box, a can and the turntable in view | 49.74° | 0 / 90 |
| Everything | all of it, twice the noise, three times the holes | 15.47° | 2 / 90 |

**[caution]** Measured in 0.1.17, without the crop from the truth and
without removing the support plane: the part alone is tracked, but a room
around it still breaks the search ([open](../ideas/open.md)). Computed
offline, frame by frame, by `just demo`; the pipeline (found by shape, scan
cropped, aligned onto the part of the model visible from the predicted pose)
is real-time by design ([the demo](../ideas/demo/README.md)).
