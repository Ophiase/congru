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
   be believed, or the forecast goes on. Every third frame on the forecast searches
   anyway: a wrong pose lying flat is neither hidden nor refuted.
5. **Too long without belief**: the object is lost, and the answer says so.

## Measured

**[defined]** [The step](../../docs/hold-step.md), [and the next](../../docs/answer-step.md).

- **After a knock, the place holds and the turn does not**: trying every
  orientation where the forecast stands finds it again, once the part is visible.
- **Never smooth an answer that can jolt**: see [what answers](what-answers.md).

## The forecast is a parameter

**[sketch]** The author's question, a first answer: `MotionModel` in `congru-track`,
one family, as in the [Singer model][singer] **[to check]**: the share of the
velocity left after each frame.

- **One** goes on for ever, the default.
- **Zero** stands where last seen, for a part on a table.
- **Between**, it dies away. The turn and the place have their gains apart.

## [question]

Which objects first? Every measure here runs the default: no scene file sets the
model yet, and only a test exercises the part at rest.

[singer]: https://doi.org/10.1109/TAES.1970.310128
[kalman]: https://en.wikipedia.org/wiki/Kalman_filter
