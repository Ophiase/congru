# Holding on when the frame says nothing

**[sketch]** The author's idea: when the alignment falls sharply at one frame, the
object is probably not detectable there, so forecast from the last positions rather
than believe that frame. A [Kalman filter][kalman] in spirit.

## Two losses, not one

[What a depth frame says](../problem/visibility.md) keeps them apart, by the rays
at the forecast pose:

| Rays at the forecast | What happened | Answer |
| --- | --- | --- |
| Most stop before the model: hidden | Something passes in front | Coast on the forecast |
| Many go through the model | It is not there any more: a knock | Search again |

A forecast is not evidence. A person keeps a hidden part where it should be, and
looks again when the space it should fill turns out empty.

## The rule, per frame

1. **Forecast** from the last poses, at their velocity.
2. **Measure** from the forecast, along rays.
3. **Believed** if the frame confirms it about as well as it did recently: a drop
   is read against the recent level, since each view says a different amount.
4. **Not believed**: hidden coasts; through searches, and what is found must itself
   be believed, or the forecast goes on.
5. **Too long without belief**: the object is lost, and the answer says so.

## Measured

**[defined]** [The step](../../docs/hold-step.md): all in a room reaches 68, 74 and
57 of 90 on three paths, two never used to choose.

- **After a knock, the place holds and the turn does not**: trying every
  orientation where the forecast stands finds it again, once the part is visible.
- **Smooth only what is not**: the tracker filters its own pose, and filtering it
  twice took shake from 88 to 78. Poses along rays are smoothed here.

## [question]

The forecast depends on the object: a part on a table, a thrown ball. By default
the current trajectory and its noise; parametric later. Which objects first?

[kalman]: https://en.wikipedia.org/wiki/Kalman_filter
