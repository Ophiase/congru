# Step: what answers a frame

The current step of demo B and of the scenes, which now run one pipeline, the
`Follower` of `congru-track`. Same noisy box as [before](hold-step.md), the only
input from the truth. The rules: [what answers](../ideas/method/what-answers.md).

## Demo B, frames within 2° of 90

Before is 0.1.23; the two last paths were never used to choose anything.

| Condition | 0.75 turns, chosen on | 0.55 turns, apart | 1.3 turns, apart | Median, first path |
| --- | --- | --- | --- | --- |
| Room | 81 → **90** | 77 → **83** | 82 → **83** | 0.99° → 0.72° |
| All in a room | 68 → **78** | 71 → **81** | 54 → **61** | 1.43° → 1.17° |
| All effects | 79 → **87** | 81 → **84** | 69 → **84** | 0.72° → 0.42° |
| Shake | 88 → **90** | 88 → **90** | 88 → **90** | 0.58° → 0.21° |
| Sensor, lens, flex | 90 → 90 | 90 → 90 | 90 → 90 | |

All in a room takes about 40 s instead of 130 s over 90 frames, four threads.

## What each change bought

All in a room, then the room, on the first path; all effects and shake do not run
the rays, and moved only with the second line.

| Change | All in a room | Room | All effects | Shake |
| --- | --- | --- | --- | --- |
| 0.1.23 | 68 | 81 | 79 | 88 |
| The place corrected faster than the turn, a lower bar for a search in a box | 74 | 81 | 82 | 88 |
| The answer is what the frame says, not the tracker's filter | 74 | 81 | 87 | 90 |
| A short alignment after the rays | 79 | 86 | 87 | 90 |
| One pipeline, its sampling a share of the size | 78 | 90 | 87 | 90 |

The tracker no longer runs behind the rays: the same answers, a quarter of the time.

## The scenes

[Their films](scene-films.md), and [the table](benchmark/scenes.md). Every scene but
two now holds all or nearly all of its frames, from a median around 1.4 % of the
size to around 0.7 %.

## [caution] What fails

- **All effects at 0.55 turns still under its 89 before the hold**: 84, the
  frames after each knock.
- **A knock in a room costs up to ten frames**: at 1.3 turns, the search finds a
  flat wrong pose that confirms 0.84, and is believed until frame 70.
- **The first frames with the can in front are off**: 2 to 4°, answered at once
  rather than not at all, which is a choice: a right pose confirms as little as
  a wrong one there.
- **The two nearest cars miss their first frames**: 60 and 54 of 72; nothing is
  found at the start.
- **Without a box, all in a room is 0 of 90**: see [the box](../ideas/method/box-as-test.md).
- **Thresholds came from one path**, and the gains are fragile: on the turn, 0.7
  and 0.3 fell to 11 of 90 at 1.3 turns; each number here is one sensor seed,
  and a run can move by a few frames with a small change of sampling. The 90 of
  the room moved from 86 that way.

![All effects, then all in a room](figures/tracking.gif)
