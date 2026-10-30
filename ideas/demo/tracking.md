# Tracking

**[defined]** As built in `crates/congru-demo`, and measured on the
simulated turntable.

1. **First frame**: no pose. The model is searched in the frame by its
   shape ([detection](../method/detection.md)), the scan cropped around
   what was found.
2. **Every later frame**: the pose is predicted from the last one and the
   last velocity; the scan is cropped around the predicted part.
3. **Only the visible model**: the model is rendered at the predicted pose,
   and the scan is aligned onto what that view shows, with normals fitted
   to it. Hidden faces (the back of a plate, the underside of the base)
   can no longer attract points.
4. **Lost**: when the alignment fails or keeps less than 80 % of the scan,
   the prediction is dropped and the next frame starts again from
   multistart.

## What each step bought

**[defined]** Measured over 90 frames, four sensing conditions:

| Change | Effect |
| --- | --- |
| Damped point-to-plane step ([observability](../method/observability.md)) | No more failure when one flat face fills the view |
| Velocity reset on implausible jumps, 12 iterations per frame | No more runaway prediction |
| Support removed only when wider than the part | The part's own faces are kept |
| Alignment onto the visible model | 90 of 90 frames within 2° on exact depth, from about 50 |

**[caution]** The velocity prediction is all the tracker knows of motion;
a part that is dropped, or occluded for long, is found again only by
multistart.
