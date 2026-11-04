# Detection

How the first pose is found in a depth frame, and found again once the
tracker has let go.

## [defined] Nothing the sensor does not give

A headset or a robot has a depth frame, the camera's intrinsics and the
model of the object. It does not have the pose, a crop around it, or the
knowledge that the object stands on a plane. Until 0.1.15 the demos took
all three: the first crop came from the truth nudged by a few degrees, and
the largest plane was removed. Both are gone; the truth only scores.

## [defined] A 2D box is fair; the rest is not

A headset also has a colour camera, and a network that draws a box around
a known object is ordinary ([BOP] scores pose methods given such boxes).
The box gives neither depth nor orientation. Until a real detector runs
(milestone 2), it is simulated: the true silhouette's box, 40 % wider and
shifted by a tenth. It is the one input still taken from the truth, and
every page that shows a scene says so.

## [sketch] Point pair features, voted

[Point pair features][drost] (Drost et al., 2010): every pair of oriented
model points is described by its distance and three angles, and stored in
a table. In the frame, each sampled reference point pairs with its
neighbours within the model's size; every table hit votes for a model
point and a rotation about the normal, and the peak gives one pose per
reference. Local by construction: a pair spans at most one object. On the whole
frame, without a box, it failed on every scene: in the noise of the
simulated sensor, a flat face of the model matches the table or the road
better than the object does. [FPFH] matched across the whole frame was
worse still, and slower (42 s on the table).

## [sketch] Each candidate checked against the depth

Every voted pose is refined, rendered with the camera and compared with the
frame, pixel by pixel.

- A pixel *agrees* when the frame is at the model's depth.
- A pixel *violates* when the frame sees behind where the model would stand.
- The candidate with most agreement, net of violations, is kept past a
  threshold: a frame without the object answers *not found*.
- The check needs nothing but the depth and the model.

**[defined]** Its score is multiplied by the share of the model inside the image.
The room still fails, for other reasons ([evidence along rays](ray-evidence.md)).

## [caution]

Shape alone cannot tell two identical objects apart, nor turn a symmetric
one the right way: colour, or the motion over a few frames, is needed.

[BOP]: https://bop.felk.cvut.cz/
[drost]: https://doi.org/10.1109/CVPR.2010.5540108
[FPFH]: https://doi.org/10.1109/ROBOT.2009.5152473
