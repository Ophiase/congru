# Tracking

**[sketch]** Aligning frame after frame is a simpler problem than aligning
once: the last pose is known, and motion is smooth most of the time. It
becomes the hard problem again only after a jump. So tracking has two
phases.

## Follow, and find

| Phase | When | What runs | Cost |
| --- | --- | --- | --- |
| **Follow** | The last frame was trusted | Predict the pose, then a few local ICP iterations from it | A few nearest-neighbour passes |
| **Find** | First frame, or the last one failed its health check | [Multistart](global-initialisation.md) over the whole model | A few dozen ICPs |

**Health check**: a followed frame is trusted when it keeps most of the scan
(inlier share) and its residual stays near the noise. Otherwise the next
frame is found again. A jump costs one expensive frame, not the track.

## The filter

**[sketch]** The prediction is an alpha-beta filter on rigid motions, the
steady-state Kalman filter of a constant-velocity model: the pose and the
velocity are corrected toward each measurement by fixed gains.

**[sketch]** The [observability](observability.md) report sets the gain per
direction: along a motion the frame leaves free, the measurement is not
believed and the prediction is kept. This is what a full Kalman filter
would do with the measurement's covariance, at a fraction of the
machinery.

**[question]** A full Kalman filter on SE(3), with the point-to-plane
Hessian as the measurement information, is the principled version; worth it
once the benchmark shows where the alpha-beta filter falls short.
