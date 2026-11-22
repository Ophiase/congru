# What answers a frame

**[sketch]** Three ways to say where the object is, each good at something else.

| Pose | Good at | Bad at |
| --- | --- | --- |
| Aligned on points, [ICP][icp] | Precision: many points | The place: clutter pulls it |
| Along rays | The place: clutter cannot pull it | Precision: a verdict per pixel |
| Forecast of the filter | Going on when a frame says nothing | Any jolt: it lags |

## The rules

- **The answer is what the frame says**, not a filter's output. The tracker's pose
  lagged behind each jolt, and its filter ran under the hold's.
- **In clutter, rays first, then a short alignment.** Rays bring the pose close;
  an ICP with a gate of a twenty-fifth of the size then gives the precision,
  kept only if the frame confirms it at least as well.
- **The filter is for the forecast.** [The motion model](hold-through-loss.md) sets
  it; along rays alone, it also smooths the answer.
- **The place answers faster than the turn.** A depth pins the place far better.
  A gain of 0.8 and 0.5 on the place, 0.6 and 0.2 on the turn; the turn is
  fragile either way: 0.7 and 0.3 lost a whole path, 11 of 90.

## [caution]

**A gate is not a cut**, as far as I can tell; the author refused a scan cut to
the model's silhouette: a hypothesis choosing the data that confirm it.

- **The short alignment keeps every point of the box**; the gate only says which
  pairs pull, as every ICP does.
- **The frame still judges**, on all its rays, not on those pairs.
- **[question]** A wrong start is not saved by it, but the author may refuse it
  too.

[icp]: https://en.wikipedia.org/wiki/Iterative_closest_point
