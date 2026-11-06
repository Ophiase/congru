# Demos

Both animations are computed from real runs, frame by frame; `just demo`
writes them again and prints the tables below.

## A — point clouds over time

![Demo A](figures/clouds.gif)

A machined bracket moves in front of a sensor, in six generated sequences
of 120 frames.

- **Red particles**: what the sensor returns.
- **Blue points**: the model, at the pose congru finds, frame after frame.
- **Each tile**: which phase ran (follow, or find again) and the rotation
  error; the bar below turns amber past 1° and red past 5°.

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

A machined bracket turns on a turntable under a depth camera; the camera,
not the part, carries the shake. The first row has every effect of demo A,
the second the same in a room. Each row:

- **image**: the scene undegraded, where a colour camera would be;
- **depth**: the degraded depth, coloured from near (red) to far (blue),
  holes left empty;
- **points**: what the tracker was given after the crop;
- **green outline**: the model at the pose congru finds, the same on all
  three; the number is the rotation error.

| Condition | Added to an imperfect sensor | Median error | Frames within 2° |
| --- | --- | --- | --- |
| All effects | everything demo A certifies: shake, lens, flex, a can crossing, two knocks of 70° | 0.44° | 85 / 90 |
| All in a room | the same, with a wall, a bench, a box and the turntable in view | 1.24° | 75 / 90 |
| Sensor | about 5 mm of noise, blur, holes, flying pixels | 0.23° | 90 / 90 |
| Shake | a camera in a hand: slow wander and jolts | 0.21° | 90 / 90 |
| Lens | radial distortion of the depth image | 0.33° | 90 / 90 |
| Flex | the part flexes by up to 4 mm | 0.27° | 90 / 90 |
| Room | the room alone | 0.72° | 90 / 90 |

One effect at a time: [sensor](figures/tracking-sensor.gif),
[shake](figures/tracking-shake.gif), [lens](figures/tracking-lens.gif),
[flex](figures/tracking-flex.gif), [room](figures/tracking-room.gif).

Still frames of what goes right and wrong, with what each ray says:
[the cases](projection-cases.md).

**[caution]** Measured on one path and one sensor seed, with a noisy box standing
in for a detector.

On two other paths, the room gives 83 and 83 of 90, all in a room 82 and 60
([this step](answer-step.md), [holding on](hold-step.md), [the box](box-step.md),
[the model's shape](shape-crop-step.md)).

- no crop from the truth but that box, no support plane removed;
- computed offline by `just demo`. A detection in a room takes seconds on four
  threads: following is real-time by design, finding is not.

The realistic scenes have their own films: [scene films](scene-films.md), and
[what they measure](benchmark/scenes.md).
