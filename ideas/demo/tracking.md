# Tracking

**[sketch]** The first frame has no pose: [multistart](../method/global-initialisation.md)
finds it. Every later frame starts from the previous pose, moved by the
last velocity, and runs point-to-plane ICP with a robust kernel against the
model, sampled once.

Each frame returns the pose, the residual and the
[observability](../method/observability.md) report. When the report says a
motion is free, the tracker keeps that part of the predicted pose instead
of trusting the data.

**[guess]** When the residual jumps or the inlier share drops, the track is
lost: multistart runs again, around the last good pose.
