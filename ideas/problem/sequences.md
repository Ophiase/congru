# Sequences

**[sketch]** The observation model of [one scenario](observation-model.md),
run over time: a model moving in front of a sensor, frame after frame, with
the truth kept at every frame. The test sets of tracking are generated, so
every claim about it is measured.

## What a sequence can contain

| Event | What it does to the frames |
| --- | --- |
| **Smooth motion** | The pose drifts and turns a little each frame, in the sensor's space |
| **Noise** | Gaussian on every observed point, as in one scenario |
| **Self-occlusion** | Only the side facing the sensor is seen; it changes as the model turns |
| **Occluder** | A ball crosses between sensor and model: what it hides vanishes, its own surface appears |
| **Distortion** | A smooth warp of the observed points: a miscalibrated or rolling sensor |
| **Jump** | The pose changes abruptly between two frames: a knock, a dropped frame, a re-grasp |
| **Shake** | A small random jolt of rotation and position every frame: a vibrating mount |
| **Handheld** | A slow, wide wander of the sensor, several degrees and centimetres over a second or two: a camera in a child's hands; smooth noise, octaves of sines whose amplitude halves as their frequency doubles |
| **Bend** | A slow, smooth flexing of the part itself: real parts are never perfectly rigid |
| **Sparsity** | Only a share of the visible surface comes back, as from a cheap sensor |
| **Clutter** | Points scattered around the part every frame |

**[defined]** The baseline of every demo sequence is already hard: sparse
(a few hundred points), noisy (a few percent of the part's size), with
clutter. Each event is added on top of it.

Each event is off by default and has one strength, so a sequence turns one
up at a time, like the sweeps of [measuring success](measuring-success.md).

**[defined]** No biological model: sequences use the machined bracket of
the [demo](../demo/scene.md) and the analytic shapes.
