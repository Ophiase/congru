# Degradations

**[defined]** What real depth sensors and real scenes do, each with one
strength, so the video turns one up at a time on top of an already
imperfect sensor.

## The sensor every condition starts from

A consumer depth camera at half a metre: Gaussian noise growing with the
square of the depth (1 cm at one metre), a one-pixel blur, 4 % of pixels
lost in round holes, and flying pixels mixing both sides of a depth edge.

## What each condition adds

| Condition | What it imitates |
| --- | --- |
| **Shake** | A camera in a hand: a slow wander of up to 4° and 2 cm over two seconds, and a jolt of up to 1° every frame on top |
| **Lens** | Radial distortion of the depth image, `k = 0.08` over the half-width |
| **Flex** | The part itself flexes by up to 4 mm, with no rigid part to first order |
| **Room** | A wall, a bench, a box, a can and the turntable all in view: the depth is never clean around the part |
| **Everything** | All of the above, with twice the noise and three times the holes |

## [caution]

The first frame's crop comes from a detector, simulated by the truth moved
by about 3 cm: without it, a room full of surfaces leaves the global search
nothing to tell the part from. Dropping that help is the *mostly the
object* rung of the [ladder](../problem/hypothesis-ladder.md).
